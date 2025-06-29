use std::collections::HashMap;
use crate::config::{Route, ServerConfig};

pub fn parse_config_str(content: &str) -> Result<ServerConfig, String> {
    let mut address = String::new();
    let mut ports = Vec::new();
    let mut server_name = None;
    let mut client_max_body_size = 0usize;
    let mut error_pages = HashMap::new();
    let mut routes = Vec::new();
    let mut in_error = false;
    let mut current_route: Option<Route> = None;
    let mut in_cgi = false;

    for line in content.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if line.starts_with("route ") && line.ends_with(':') {
            if let Some(r) = current_route.take() { routes.push(r); }
            in_error = false;
            in_cgi = false;

            let path = line["route ".len()..line.len()-1].trim().to_string();
            current_route = Some(Route {
                path,
                methods: vec![],
                redirection: None,
                root: None,
                default_file: None,
                directory_listing: None,
                cgi: None,
            });
            continue;
        }

        if let Some(route) = current_route.as_mut() {
            if line.starts_with("cgi:") {
                in_cgi = true;
                route.cgi = Some(HashMap::new());
                continue;
            }
            if in_cgi {
                if let Some((ext, cmd)) = line.split_once(':') {
                    route.cgi.as_mut()
                        .unwrap()
                        .insert(ext.trim().to_string(), cmd.trim().to_string());
                }
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                let v = v.trim();
                match k.trim() {
                    "methods" => route.methods = v.split(',').map(|s| s.trim().to_string()).collect(),
                    "redirection" => route.redirection = Some(v.to_string()),
                    "root" => route.root = Some(v.to_string()),
                    "default_file" => route.default_file = Some(v.to_string()),
                    "directory_listing" => {
                        let flag = matches!(v.to_lowercase().as_str(), "on"|"true");
                        route.directory_listing = Some(flag);
                    }
                    _ => {}
                }
            }
            continue;
        }

        if line.starts_with("error_pages:") {
            in_error = true;
            continue;
        }
        if in_error {
            if let Some((code, path)) = line.split_once(':') {
                let code = code.trim().parse::<u16>()
                    .map_err(|_| format!("Invalid error code '{}'", code.trim()))?;
                error_pages.insert(code, path.trim().to_string());
            }
            continue;
        }

        if let Some((k, v)) = line.split_once(':') {
            let v = v.trim();
            match k.trim() {
                "server_address" => address = v.to_string(),
                "ports" => ports = v.split(',')
                    .map(|s| s.trim().parse::<u16>()
                        .map_err(|e| format!("Invalid port {}", e)))
                    .collect::<Result<_, _>>()?,
                "server_name" => server_name = Some(v.to_string()),
                "client_max_body_size" => client_max_body_size = v.parse::<usize>()
                    .map_err(|e| format!("Invalid max body size '{}': {}", v, e))?,
                _ => {},
            }
        }
    }

    if let Some(r) = current_route.take() {
        routes.push(r);
    }

    Ok(ServerConfig {
        address,
        ports,
        server_name,
        client_max_body_size,
        error_pages,
        routes,
    })
}

pub fn parse_config_file(path: &str) -> Result<ServerConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Could not open config '{}': {}", path, e))?;
    parse_config_str(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config_full() {
        let s = "
            server_address: 127.0.0.1
            ports: 8080,8081
            client_max_body_size: 1500
            error_pages:
              404: /err/404.html

            route /test:
              methods: GET,POST
              root: /tmp
              default_file: index.html
              directory_listing: on
              cgi:
                .py: /usr/bin/python3
                .rb: /usr/bin/ruby
        ";
        let cfg = parse_config_str(s).unwrap();
        assert_eq!(cfg.address, "127.0.0.1");
        assert_eq!(cfg.ports, vec![8080, 8081]);
        assert_eq!(cfg.client_max_body_size, 1500);
        assert_eq!(cfg.error_pages.get(&404).unwrap(), "/err/404.html");
        let route = &cfg.routes[0];
        assert_eq!(route.path, "/test");
        assert!(route.methods.contains(&"POST".to_string()));
        assert!(route.directory_listing.unwrap());
        let cgi_map = route.cgi.as_ref().unwrap();
        assert_eq!(cgi_map.get(".py").unwrap(), "/usr/bin/python3");
        assert_eq!(cgi_map.get(".rb").unwrap(), "/usr/bin/ruby");
    }

    #[test]
    fn invalid_port_fails() {
        let s = "ports: notnum";
        assert!(parse_config_str(s).is_err());
    }

    #[test]
    fn invalid_body_size_fails() {
        let s = "client_max_body_size: abc";
        assert!(parse_config_str(s).is_err());
    }

    #[test]
    fn parse_directory_listing_flags() {
        let s = "
            route /files:
              methods: GET
              directory_listing: off
        ";
        let cfg = parse_config_str(s).unwrap();
        assert_eq!(cfg.routes[0].directory_listing, Some(false));
    }

    #[test]
    fn parse_cgi_mappings() {
        let s = "
            route /scripts:
              methods: GET
              cgi:
                .py: /usr/bin/python3
        ";
        let cfg = parse_config_str(s).unwrap();
        let cgi_map = &cfg.routes[0].cgi;
        assert!(cgi_map.is_some());
        assert_eq!(cgi_map.as_ref().unwrap().get(".py").unwrap(), "/usr/bin/python3");
    }
}
