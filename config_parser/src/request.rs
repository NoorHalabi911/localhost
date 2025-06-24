use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub fn from_raw(raw: &str) -> Option<Self> {
        let mut lines = raw.lines();
        let request_line = lines.next()?;

        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        let mut headers = HashMap::new();
        let mut body = String::new();
        let mut in_body = false;

        for line in lines {
            if line.is_empty() {
                in_body = true;
                continue;
            }

            if in_body {
                body.push_str(line);
                body.push('\n');
            } else if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Some(Request {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}
