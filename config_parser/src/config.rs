use std::collections::HashMap;

#[derive(Debug, Clone)] // 👈 Add Clone here
pub struct ServerConfig {
    pub address: String,
    pub ports: Vec<u16>,
    pub server_name: Option<String>,
    pub client_max_body_size: usize,
    pub error_pages: HashMap<u16, String>,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone)] // 👈 Add Clone here too
pub struct Route {
    pub path: String,
    pub methods: Vec<String>,
    pub redirection: Option<String>,
    pub root: Option<String>,
    pub default_file: Option<String>,
    pub directory_listing: Option<bool>,
    pub cgi: Option<HashMap<String, String>>,
}

