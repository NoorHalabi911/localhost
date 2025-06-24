use std::fs;
use std::path::Path;
use crate::config::{Route, ServerConfig};
use crate::request::Request;

#[derive(Debug)]
pub enum RouteResult {
    Ok(String),               // HTML content
    Redirect(String),         // Redirection URL
    NotFound,
    Forbidden,
    MethodNotAllowed,
}

pub fn route_request(config: &ServerConfig, request: &Request) -> RouteResult {
    println!("🔍 Incoming request path: {}", request.path);
    println!("🔍 Incoming method: {}", request.method);

    for route in &config.routes {
        println!("📁 Checking route: {}", route.path);
        println!("✅ Allowed methods: {:?}", route.methods);

        if request.path.starts_with(&route.path) {
            if !route.methods.contains(&request.method) {
                return RouteResult::MethodNotAllowed;
            }

            // Redirection
            if let Some(ref redirect) = route.redirection {
                return RouteResult::Redirect(redirect.clone());
            }

            let mut rel_path = request
                .path
                .trim_start_matches(&route.path)
                .trim_start_matches('/')
                .to_string();

            if rel_path.is_empty() {
                if let Some(default_file) = &route.default_file {
                    rel_path = default_file.clone();
                } else {
                    return RouteResult::NotFound;
                }
            }

            let root = match &route.root {
                Some(r) => r,
                None => return RouteResult::NotFound,
            };

            let full_path = format!("{}/{}", root, rel_path);

            if Path::new(&full_path).exists() {
                match fs::read_to_string(&full_path) {
                    Ok(content) => return RouteResult::Ok(content),
                    Err(_) => return RouteResult::Forbidden,
                }
            } else {
                return RouteResult::NotFound;
            }
        }
    }

    RouteResult::NotFound
}
