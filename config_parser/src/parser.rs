use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use crate::config::{Route, ServerConfig};

pub fn parse_config_file(path: &str) -> Result<ServerConfig, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open config: {}", e))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let mut address = String::new();
    let mut ports = Vec::new();
    let mut server_name = None;
    let mut max_body_size = 0;
    let mut error_pages = HashMap::new();
    let mut routes = Vec::new();

    let mut in_error_pages = false;
    let mut current_route: Option<Route> = None;
    let mut in_cgi_block = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            in_error_pages = false;
            in_cgi_block = false;
            continue;
        }

        if trimmed.starts_with("route ") && trimmed.ends_with(':') {
            if let Some(route) = current_route.take() {
                routes.push(route);
            }

            let path = trimmed.trim_start_matches("route").trim_end_matches(':').trim().to_string();
            current_route = Some(Route {
                path,
                methods: Vec::new(),
                redirection: None,
                root: None,
                default_file: None,
                directory_listing: None,
                cgi: None,
            });
            continue;
        }

        if let Some(route) = current_route.as_mut() {
            if trimmed.starts_with("cgi:") {
                in_cgi_block = true;
                route.cgi = Some(HashMap::new());
                continue;
            }

            if in_cgi_block {
                if let Some((ext, bin)) = trimmed.split_once(':') {
                    if let Some(cgi_map) = route.cgi.as_mut() {
                        cgi_map.insert(ext.trim().to_string(), bin.trim().to_string());
                    }
                }
                continue;
            }

            if let Some((key, value)) = trimmed.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "methods" => {
                        route.methods = value.split(',').map(|m| m.trim().to_string()).collect();
                    }
                    "redirection" => route.redirection = Some(value.to_string()),
                    "root" => route.root = Some(value.to_string()),
                    "default_file" => route.default_file = Some(value.to_string()),
                    "directory_listing" => {
                        let enabled = value.eq_ignore_ascii_case("on");
                        route.directory_listing = Some(enabled);
                    }
                    _ => {}
                }
            }
            continue;
        }

        if trimmed.starts_with("error_pages:") {
            in_error_pages = true;
            continue;
        }

        if in_error_pages {
            if let Some((code, path)) = trimmed.split_once(':') {
                if let Ok(code) = code.trim().parse::<u16>() {
                    error_pages.insert(code, path.trim().to_string());
                }
            }
            continue;
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "server_address" => address = value.to_string(),
                "ports" => {
                    ports = value
                        .split(',')
                        .filter_map(|p| p.trim().parse::<u16>().ok())
                        .collect();
                }
                "server_name" => server_name = Some(value.to_string()),
                "client_max_body_size" => {
                    max_body_size = value.parse::<usize>().unwrap_or(0);
                }
                _ => {}
            }
        }
    }

    if let Some(route) = current_route {
        routes.push(route);
    }

    Ok(ServerConfig {
        address,
        ports,
        server_name,
        client_max_body_size: max_body_size,
        error_pages,
        routes,
    })
}
