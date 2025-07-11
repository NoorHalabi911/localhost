use cgi::run_cgi_script;
use mio::{Events, Interest, Poll, Token};
use serde::Deserialize;
use serverConfig::ServerConfig;
use static_file::{FileResponse, build_http_response, read_static_file};
use std::collections::HashMap;
use std::env;
use std::io;
use std::io::{Read, Write};
// use std::os::unix::io::{AsRawFd, RawFd};
use mio::net::{TcpListener, TcpStream};
use std::{fs, time::Duration};
use upload_handler::{UploadResult, build_upload_response, handle_file_upload};
mod session_manager;
use session_manager::SessionManager;

use crate::serverConfig::Connection;
mod cgi;
mod serverConfig;
mod static_file;
mod upload_handler;

fn main() {
    let servers = json_parser();
    let mut session_manager = SessionManager::new(); // create a new session manager
    for server in &servers {
        listener_socket(&server, &mut session_manager);
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
    let config_path = wd.join("src/config.json");
    println!("wd {}", wd.display());
    let file = fs::read_to_string(config_path).expect("file don't exist");
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

fn listener_socket(
    server: &ServerConfig,
    session_manager: &mut SessionManager,
) -> std::io::Result<()> {
    let mut listeners: HashMap<Token, TcpListener> = HashMap::new();

    for address in &server.server_address {
        let mut next_token = listeners.len();
        let bind_address = format!("{}:{}", address.ip, address.port);
        let socket_addr: std::net::SocketAddr =
            bind_address.parse().expect("Invalid socket address");

        // opens a tcp socket and it become passive
        // binding is like : I want to listen for connections on this IP:PORT
        let mut listener =
            TcpListener::bind(socket_addr).expect(&format!("Failed to bind to {}", bind_address));
        println!("Listening on {}", bind_address);

        let token = Token(next_token);
        println!("listenr tokken {:?}", token);

        listeners.insert(token, listener);
    }

    run_mio_server(listeners, session_manager)
}
pub fn run_mio_server(
    mut listeners: HashMap<Token, TcpListener>,
    session_manager: &mut SessionManager,
) -> std::io::Result<()> {
    let mut poll = Poll::new()?; //this is an event loop to watch socket's 
    let mut events = Events::with_capacity(2048);

    let mut clients: HashMap<Token, Connection> = HashMap::new();
    let mut next_token = listeners.len() + 1;

    // Register all listening sockets
    for (token, listener) in listeners.iter_mut() {
        poll.registry()
            .register(listener, *token, Interest::READABLE)?;
        // (*) is de refrence we copy the value and give it the ownership of the copy
    }

    println!("Starting mio event loop...");
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(10)))?;
        //checks every 10ms if any socket is ready for action

        for event in events.iter() {
            //
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
                            // it means there are no more clients waiting
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
                        // we get n bytes from the client we write them in a growable Vec
                        //becuse you might not get the full Requset in one go
                        conn.read_buffer.extend_from_slice(&temp_buf[..n]);

                        if let Some(pos) =
                            // we look for the ending sequence of the HTTP headers to know if we recive the full request
                            conn.read_buffer.windows(4).position(|w| w == b"\r\n\r\n")
                        {
                            //this happene if the requset is full
                            //to get the path bassed on the request path
                            let request = String::from_utf8_lossy(&conn.read_buffer[..pos]);
                            let path = request
                                .lines()
                                .next()
                                .and_then(|line| line.split_whitespace().nth(1))
                                .unwrap_or("/");

                            // 🪪 extract Cookie header (if exists)
                            let cookie_header = request
                                .lines()
                                .find(|line| line.starts_with("Cookie:"))
                                .map(|line| line.trim_start_matches("Cookie:").trim());

                            // 🪪 get or create session
                            let session = session_manager.get_or_create_session(cookie_header);

                            // 🪪 prepare Set-Cookie header (only if session is new)
                            let set_cookie_header = format!(
                                "Set-Cookie: session_id={}; Path=/; HttpOnly\r\n",
                                session.id
                            );
                            // fetch the file needed based of the path
                            let file = if path == "/" {
                                handle_path("def")
                            } else {
                                handle_path(path.trim_start_matches('/'))
                            };

                            conn.write_buffer = match file {
                            Ok(content) => format!(
                                    "HTTP/1.1 200 OK\r\n{}Content-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                                    set_cookie_header,
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
                // this cheaks if the socket is writable

                /*so we write the respone to the client buffer
                then when it's ready we write it in the straem and when it's
                finished we clean the buffer and close the connection */
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
                        } /*why we do this
                          only react when socket are ready
                          never block waiting on slow clients
                          */
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
