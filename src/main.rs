use cgi::run_cgi_script;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use serde::Deserialize;
use serverConfig::ServerConfig;
use static_file::{FileResponse, build_http_response, read_static_file};
use std::collections::HashMap;
use std::env;
use std::io;
use std::io::{Read, Write};
use std::{fs, time::Duration};
use upload_handler::{UploadResult, build_upload_response, handle_file_upload};

use crate::serverConfig::Connection;
mod cgi;
mod serverConfig;
mod static_file;
mod upload_handler;

fn main() {
    let servers = json_parser();
    for server in &servers {
        listener_socket(&server);
    }
}
// fn test_cgi()-> Result<(), Box<dyn Error>> {
// let mut wd = env::current_dir().unwrap();
// let python_file_path = wd.join("py.py");
// println!("{}", python_file_path.display());
// let body = "name=Rust";
// let path_info = wd.to_string_lossy();
// ///mnt/c/Users/admar/Desktop/lh/py.py
// match run_cgi_script(python_file_path.to_str().unwrap(), body, &path_info) {
//     Ok(output) => println!("{}", output),
//     Err(e) => eprintln!("Error: {}", e),
// }
// Ok(())
// }

fn json_parser() -> Vec<ServerConfig> {
    let mut wd = env::current_dir().unwrap();
    println!("wd {}", wd.display());
    let file = fs::read_to_string("config.json").expect("file don't exist");
    println!("fu=ile {}", file);
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
const SERVER: Token = Token(0);

fn listener_socket(server: &ServerConfig) -> std::io::Result<()> {
    let mut listeners: HashMap<Token, TcpListener> = HashMap::new();

    for address in &server.server_address {
        let mut next_token = listeners.len();
        let bind_address = format!("{}:{}", address.ip, address.port);
        let socket_addr: std::net::SocketAddr =
            bind_address.parse().expect("Invalid socket address");

        let mut listener =
            TcpListener::bind(socket_addr).expect(&format!("Failed to bind to {}", bind_address));
        println!("Listening on {}", bind_address);

        let token = Token(next_token);
        println!("listenr tokken {:?}", token);

        listeners.insert(token, listener);
    }

    run_mio_server(listeners)
}
pub fn run_mio_server(mut listeners: HashMap<Token, TcpListener>) -> std::io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(2048);
    let mut buffer = [0; 2048];

    let mut clients: HashMap<Token, Connection> = HashMap::new();
    let mut next_token = listeners.len() + 1;

    // Register all listening sockets
    for (token, listener) in listeners.iter_mut() {
        poll.registry()
            .register(listener, *token, Interest::READABLE)?;
    }

    println!("Starting mio event loop...");
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(10)))?;

        for event in events.iter() {
            let token = event.token();

            if listeners.contains_key(&token) {
                let listener = listeners.get_mut(&token).unwrap();

                loop {
                    match listener.accept() {
                        Ok((mut stream, addr)) => {
                            println!("New connection from {:?}", addr);
                            // mio::net::TcpStream is non-blocking by default
                            let client_token = Token(next_token);
                            println!("client token {:?}", client_token);
                            next_token += 1;

                            poll.registry().register(
                                &mut stream,
                                client_token,
                                Interest::READABLE,
                            )?;
                            clients.insert(
                                client_token,
                                Connection {
                                    stream,
                                    read_buffer: Vec::new(),
                                    write_buffer: Vec::new(),
                                    is_writing: false,
                                },
                            );
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            eprintln!("error at wouldBlock {}", e);
                            break;
                        }
                        Err(e) => {
                            eprintln!("Accept failed: {}", e);
                            break;
                        }
                    }
                }
                continue;
            }
            if let Some(conn) = clients.get_mut(&token) {
                println!("we are at clients.get_mut");
                let mut temp_buf = [0; 2048];
                match conn.stream.read(&mut temp_buf) {
                    // if the client disconects remove it by token
                    Ok(0) => {
                        println!("Client {:?} disconnected", token);
                        clients.remove(&token);
                        continue;
                    }
                    Ok(n) => {
                        conn.read_buffer.extend_from_slice(&temp_buf[..n]);

                        if let Some(pos) =
                            conn.read_buffer.windows(4).position(|w| w == b"\r\n\r\n")
                        {
                            let request = String::from_utf8_lossy(&conn.read_buffer[..pos]);
                            let path = request
                                .lines()
                                .next()
                                .and_then(|line| line.split_whitespace().nth(1))
                                .unwrap_or("/");

                            let file = if path == "/" {
                                handle_path("def")
                            } else {
                                handle_path(path.trim_start_matches('/'))
                            };

                            conn.write_buffer = match file {
                            Ok(content) => format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                                    content.len(),
                                    content
                                ).into_bytes(),
                            Err(_) => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec(),
                        };
                            conn.is_writing = true;
                            conn.read_buffer.clear(); // clear the memory client after handling

                            poll.registry().reregister(
                                &mut conn.stream,
                                token,
                                Interest::WRITABLE,
                            )?;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        println!("not ready to read yet");
                    }
                    Err(e) => {
                        println!("Failed to read from client {:?}: {}", token, e);
                        clients.remove(&token); // and error from the client we remove
                        continue;
                    }
                }
                if event.is_writable() && conn.is_writing {
                    match conn.stream.write(&conn.write_buffer) {
                        Ok(n) => {
                            conn.write_buffer.drain(..n);

                            if conn.write_buffer.is_empty() {
                                conn.is_writing = false;
                                // Properly shutdown the stream before removing
                                let _ = conn.stream.shutdown(std::net::Shutdown::Both);
                                poll.registry().deregister(&mut conn.stream)?;
                                clients.remove(&token);
                                println!("After write");
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // Wait until ready again
                        }
                        Err(e) => {
                            println!("Write error to {:?}: {}", token, e);
                            clients.remove(&token);
                            continue;
                        }
                    }
                }
            }
        }
    }
}

fn handle_path(path: &str) -> io::Result<String> {
    let mut wd = env::current_dir().unwrap();
    let file_name = format!("{}.html", path);
    let html_path = wd.join("html").join(file_name);
    println!("html path {}", html_path.display());
    fs::read_to_string(html_path)
}
