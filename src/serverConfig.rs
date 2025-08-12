use mio::net::TcpStream;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
/// Top-level server configuration loaded from `src/config.json`.
///
/// Why: Drives listeners, routing, and error pages at runtime.
/// How: Deserialized by `serde_json` in `main.rs::json_parser`.
pub struct ServerConfig {
    pub server_name: String,
    pub server_address: Vec<ServerAddress>, //ip and Port
    pub max_body_size: usize,               // in bytes
    pub router: Vec<RouterConfig>,
    pub error_msg: HashMap<u16, String>, // status code and page path
}
#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
/// Per-route configuration.
///
/// Why: Controls method restrictions, roots, index files, CGI, listing, and redirects.
/// How: The router matches the longest `path` prefix and applies these settings.
pub struct RouterConfig {
    pub path: String,
    pub methods: Vec<String>, // GET, POST, etc.
    pub root: String,
    pub index: Option<String>, // default file to serve
    pub cgi: Option<(String, String)>,
    pub directory_listing: Option<bool>, // enable/disable directory listing for this route
    pub redirection: Option<RedirectionConfig>, // optional redirection
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
/// Redirection policy for a route.
///
/// Why: Support 301/302/307/308 redirects in config.
/// How: When a route has `redirection`, `handle_request` returns a redirect response.
pub struct RedirectionConfig {
    pub target: String,
    pub status: Option<u16>, // 301 or 302, default to 302
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
/// IP/Port pair for listeners.
///
/// Why: A server can bind multiple addresses.
/// How: Each entry is bound to a `TcpListener` at startup.
pub struct ServerAddress {
    pub ip: String,
    pub port: u16,
}
/// Per-connection state used by the mio event loop.
///
/// Why: Track read/write buffers, readiness, and activity for timeouts.
/// How: Stored in a `Token -> Connection` map inside the event loop.
pub struct Connection {
    pub stream: TcpStream,
    pub read_buffer: Vec<u8>,
    pub write_buffer: Vec<u8>,
    pub is_writing: bool,
    pub last_active: Instant,
}
