use crate::request::Request;
use crate::config::ServerConfig;
use crate::router::{route_request, RouteResult};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

pub fn start(config: &ServerConfig) {
    println!("🚀 Launching server...");

    let mut listeners = Vec::new();

    for port in &config.ports {
        let addr = format!("{}:{}", config.address, port);
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                listener
                    .set_nonblocking(true)
                    .expect("❌ Could not make listener non-blocking");
                println!("✅ Listening on {}", addr);
                listeners.push(listener);
            }
            Err(e) => {
                eprintln!("❌ Failed to bind {}: {}", addr, e);
            }
        }
    }

    println!("💡 Waiting for clients...\n");

    loop {
        for listener in &listeners {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!("👋 New client from {}", addr);
                    let config_clone = config.clone();
                    thread::spawn(move || {
                        handle_client(stream, &config_clone);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    eprintln!("❌ Accept error: {}", e);
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn handle_client(mut stream: TcpStream, config: &ServerConfig) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(bytes) if bytes > 0 => {
            let request_text = String::from_utf8_lossy(&buffer[..bytes]);
            println!("📥 Raw Request:\n{}", request_text);

            if let Some(request) = Request::from_raw(&request_text) {
                println!("✅ Parsed Request: {:#?}", request);

                match route_request(config, &request) {
                    RouteResult::Ok(content) => {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
                            content
                        );
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    RouteResult::Redirect(location) => {
                        let response = format!(
                            "HTTP/1.1 302 Found\r\nLocation: {}\r\n\r\n",
                            location
                        );
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                    RouteResult::MethodNotAllowed => {
                        stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n").unwrap();
                    }
                    RouteResult::Forbidden => {
                        stream.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").unwrap();
                    }
                    RouteResult::NotFound => {
                        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                    }
                }
            } else {
                stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").unwrap();
            }
        }
        Ok(_) => println!("⚠️ Empty request"),
        Err(e) => eprintln!("❌ Read error: {}", e),
    }
}
