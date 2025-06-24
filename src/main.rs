use std::{fs, net::TcpListener};
mod serverConfig;

use serde::Deserialize;
use serverConfig::ServerConfig;
use std::collections::HashMap;
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
    let mut listeners: HashMap<String, TcpListener> = HashMap::new();

    for address in &server.server_address {
        let bind_address = format!("{}:{}", address.ip, address.port);

        let listener = TcpListener::bind(&bind_address)
            .expect(&format!("Failed to bind to address: {}", bind_address));
        listener
            .set_nonblocking(true)
            .expect("failed to set non-blocking");
        println!("Listening on: {}", bind_address);

        listeners.insert(bind_address, listener);
    }
}
