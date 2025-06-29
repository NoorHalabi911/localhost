use std::io::Read;
use std::{
    fs,
    net::{TcpListener, TcpStream},
};
mod serverConfig;

use serde::Deserialize;
use serverConfig::ServerConfig;
use std::collections::HashMap;

mod static_file;
use static_file::{FileResponse, build_http_response, read_static_file};
mod upload_handler;
use upload_handler::{build_upload_response, handle_file_upload};

use libc;
use std::os::unix::io::{AsRawFd, RawFd}; // Not available on Windows

fn main() {
    // مثال يدوي: كأنك استقبلت طلب من المتصفح

    // 1. طلب GET لملف ثابت
    let method = "GET";
    let path = "/index.html";

    if method == "GET" {
        let result = read_static_file(path);
        let response = build_http_response(result);
        println!("--- GET Response ---");
        println!("{}", String::from_utf8_lossy(&response));
    }

    // 2. طلب POST لرفع ملف
    let method = "POST";
    let content_type = "multipart/form-data; boundary=----XYZ";
    let fake_body = fs::read("tests/upload_example_body.txt").unwrap_or_default();

    if method == "POST" {
        let result = handle_file_upload(&fake_body, content_type);
        let response = build_upload_response(result);
        println!("--- POST Response ---");
        println!("{}", String::from_utf8_lossy(&response));
    }

    let servers = json_parser();
    for server in &servers {
        listener_socket(&server);
    }
}

fn json_parser() -> Vec<ServerConfig> {
    let file = fs::read_to_string("src/config.json").expect("file don't exist");

    let servers: Vec<ServerConfig> =
        serde_json::from_str(&file).expect("JSON was not well-formatted");

    //for printing the servers configs
    // for server in servers {
    //     println!("Server Name: {:#?}", server);
    // }
    servers
}
fn listener_socket(server: &ServerConfig) {
    let mut listeners: HashMap<RawFd, TcpListener> = HashMap::new();

    for address in &server.server_address {
        let bind_address = format!("{}:{}", address.ip, address.port);

        let listener = TcpListener::bind(&bind_address)
            .expect(&format!("Failed to bind to address: {}", bind_address));
        listener
            .set_nonblocking(true)
            .expect("failed to set non-blocking");
        println!("Listening on: {}", bind_address);

        listeners.insert(listener.as_raw_fd(), listener);
    }
    run_epoll(listeners)
}

fn run_epoll(mut listeners: HashMap<RawFd, TcpListener>) {
    const MAX_EVENTS: usize = 1024;
    let epoll_fd = unsafe { libc::epoll_create1(0) };
    if epoll_fd == -1 {
        panic!("Failed to create epoll");
    }
    let mut clients: HashMap<RawFd, TcpStream> = HashMap::new();

    // Register all listening sockets
    for fd in listeners.keys() {
        let mut ev = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: *fd as u64,
        };

        let res = unsafe { libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, *fd, &mut ev) };
        if res == -1 {
            panic!("Failed to add fd {} to epoll", fd);
        }
    }

    let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; MAX_EVENTS];

    println!("Starting epoll loop...");
    loop {
        // number of file descriptors that are ready
        let nfds =
            unsafe { libc::epoll_wait(epoll_fd, events.as_mut_ptr(), MAX_EVENTS as i32, 1000) };
        if nfds < 0 {
            eprintln!("epoll_wait failed");
            break;
        }

        for i in 0..nfds as usize {
            let event = events[i];
            let fd = event.u64 as RawFd;

            //we check if the event is from a listener or a client

            if let Some(listener) = listeners.get(&fd) {
                // this for the listener
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("New connection from {:?}", addr);
                        stream.set_nonblocking(true).unwrap();
                        let client_fd = stream.as_raw_fd();

                        let mut ev = libc::epoll_event {
                            events: libc::EPOLLIN as u32,
                            u64: client_fd as u64,
                        };
                        // register the new client socket with epoll
                        let res = unsafe {
                            libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, client_fd, &mut ev)
                        };
                        if res == -1 {
                            eprintln!("Failed to add client fd {} to epoll", client_fd);
                            continue;
                        }
                        clients.insert(client_fd, stream);
                    }
                    Err(e) => {
                        eprintln!("accept() failed: {}", e);
                    }
                }

                //reading from the client
            } else if let Some(mut stream) = clients.remove(&fd) {
                let mut buffer = [0u8; 2048]; // Buffer size of 2048 bytes
                match stream.read(&mut buffer) {
                    Ok(0) => {
                        // Client disconnected
                        println!("Client {} disconnected", fd);
                        unsafe {
                            libc::epoll_ctl(
                                epoll_fd,
                                libc::EPOLL_CTL_DEL,
                                fd,
                                std::ptr::null_mut(),
                            );
                        }
                        continue;
                    }
                    Ok(n) => {
                        println!("Read {} bytes from client {}", n, fd);
                    }
                    Err(e) => {
                        eprintln!("Failed to read from client {}: {}", fd, e);
                        unsafe {
                            libc::epoll_ctl(
                                epoll_fd,
                                libc::EPOLL_CTL_DEL,
                                fd,
                                std::ptr::null_mut(),
                            );
                        }
                        continue;
                    }
                }
            }

            {
                println!("Client FD {} is ready for read/write", fd);
            }
        }
    }
}
