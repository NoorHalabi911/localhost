use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub server_name: String,
    pub server_address: Vec<(String, u16)>, //ip and Port
    pub max_body_size: usize,               // in bytes
    pub router: Vec<RouterConfig>,
    pub error_page: HashMap<u16, String>, // status code and page path
}
#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
struct RouterConfig {
    pub path: String,
    pub methods: Vec<String>, // GET, POST, etc.
    pub root: String,
    pub index: Option<String>, // default file to serve
    pub cgi: Option<(String, String)>,
}
