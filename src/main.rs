use cgi::run_cgi_script; // Not available on Windows
use libc;
use serde::Deserialize;
use serverConfig::ServerConfig;
use static_file::{FileResponse, build_http_response, read_static_file};
use std::collections::HashMap;
use std::env;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::{
    fs,
    net::{TcpListener, TcpStream},
};
use upload_handler::{UploadResult, build_upload_response, handle_file_upload}; // Not available on Windows
mod cgi;
mod serverConfig;
mod static_file;
mod upload_handler;

fn main() {
    // let method = "GET";
    // let path = "/index.html";

    // if method == "GET" {
    //     let result = read_static_file(path);
    //     let response = build_http_response(result);
    //     println!("--- GET Response ---");
    //     println!("{}", String::from_utf8_lossy(&response));
    // }

    // // 2. طلب POST لرفع ملف
    // let method = "POST";
    // let content_type = "multipart/form-data; boundary=----XYZ";
    // let fake_body = fs::read("tests/upload_example_body.txt").unwrap_or_default();

    // if method == "POST" {
    //     let result = handle_file_upload(&fake_body, content_type);
    //     let response = build_upload_response(result);
    //     println!("--- POST Response ---");
    //     println!("{}", String::from_utf8_lossy(&response));
    // }
    let servers = json_parser();
    for server in &servers {
        listener_socket(&server);
    }
}

fn json_parser() -> Vec<ServerConfig> {
    let file = fs::read_to_string("src/config.json").expect("file don't exist");

    let servers: Vec<ServerConfig> =
        serde_json::from_str(&file).expect("JSON was not well-formatted");

    // let servers: Vec<ServerConfig> =
    //     serde_yaml::from_str(&file).expect("yaml was not well-formatted");

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
    println!("im in epoll");
    const MAX_EVENTS: usize = 1024;
    let mut buffer = [0; 1024];
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
                        let request = String::from_utf8_lossy(&buffer[..n]);
                        let path = request
                            .lines()
                            .next()
                            .and_then(|line| line.split_whitespace().nth(1))
                            .unwrap_or("/");

                        let mut file: Result<String, io::Error>;
                        if path == "/" {
                            file = handle_path("def");
                        } else {
                            let file_needed = path.trim_start_matches('/');
                            file = handle_path(file_needed);
                        }
                        match file {
                            Ok(content) => {
                                let resp: String = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                                    content.len(),
                                    content
                                );
                                let _ = stream.write_all(resp.as_bytes());
                            }
                            Err(_) => {
                                let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                                let _ = stream.write_all(resp.as_bytes());
                            }
                        }

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
fn handle_path(path: &str) -> io::Result<String> {
    let mut wd = env::current_dir().unwrap();
    let file_name = format!("{}.html", path);
    let html_path = wd.join("html").join(file_name);
    fs::read_to_string(html_path)
}
