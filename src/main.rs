use std::{fs, net::TcpListener};
mod serverConfig;

use serde::Deserialize;
use serverConfig::ServerConfig;
use std::collections::HashMap;

use libc;
use std::os::unix::io::{AsRawFd, RawFd}; // Not available on Windows

fn main() {
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
        let nfds =
            unsafe { libc::epoll_wait(epoll_fd, events.as_mut_ptr(), MAX_EVENTS as i32, 1000) };

        if nfds < 0 {
            eprintln!("epoll_wait failed");
            break;
        }

        for i in 0..nfds as usize {
            let event = events[i];
            let fd = event.u64 as RawFd;

            if let Some(listener) = listeners.get(&fd) {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("New connection from {:?}", addr);
                        stream.set_nonblocking(true).unwrap();
                        let client_fd = stream.as_raw_fd();

                        let mut ev = libc::epoll_event {
                            events: libc::EPOLLIN as u32,
                            u64: client_fd as u64,
                        };

                        unsafe {
                            libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, client_fd, &mut ev);
                        }

                        // TODO: Store client stream somewhere (e.g., client map)
                    }
                    Err(e) => {
                        eprintln!("accept() failed: {}", e);
                    }
                }
            } else {
                println!("Client FD {} is ready for read/write", fd);
                // TODO: Read client data (next phase)
            }
        }
    }
}
