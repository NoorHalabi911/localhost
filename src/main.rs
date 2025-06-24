use std::fs;
mod serverConfig;

use serde::Deserialize;
use serverConfig::ServerConfig;
fn main() {
    json_parser();
}

fn json_parser() {
    let file = fs::read_to_string("src/config.json").expect("file don't exist");

    let servers: Vec<ServerConfig> =
        serde_json::from_str(&file).expect("JSON was not well-formatted");

    for server in servers {
        println!("Server Name: {:#?}", server);
    }
}
