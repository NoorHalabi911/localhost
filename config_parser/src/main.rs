mod config;
mod parser;
mod server;

fn main() {
    let config = parser::parse_config_file("example.conf")
        .expect("Failed to load config");
    server::start(&config);
}
