use std::fs;
mod serverConfig;

use serverConfig::ServerConfig;
fn main() {}

fn json_parser() {
    let file = fs::read_to_string("config.json").expect("file don't exist");

    let servers: Vec<ServerConfig> =
        serde_json::from_str(&file).expect("JSON was not well-formatted");
}
