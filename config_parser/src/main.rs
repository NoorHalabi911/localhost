mod config;
mod parser;
mod server;
mod request;
mod router;

fn main() {
    println!("🔧 Starting config parser...");

    match parser::parse_config_file("example.conf") {
        Ok(config) => {
            println!("✅ Config loaded.\n");
            server::start(&config);
        }
        Err(err) => {
            eprintln!("❌ Failed to parse config: {}", err);
        }
    }
}
