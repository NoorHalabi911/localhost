use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub fn parse_http_request(raw: &str) -> Option<Request> {
    let mut lines = raw.split("\r\n");
    let request_line = lines.next()?; // e.g. GET /index.html HTTP/1.1

    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let version = parts.next()?.to_string();

    let mut headers = HashMap::new();
    for line in &mut lines {
        if line.is_empty() {
            break; // End of headers
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    // Join remaining lines as body (if any)
    let body = lines.collect::<Vec<&str>>().join("\r\n").into_bytes();

    Some(Request {
        method,
        path,
        version,
        headers,
        body,
    })
}

pub fn build_response(res: Response) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 {} {}\r\n", res.status_code, res.reason_phrase);

    if !res.headers.contains_key("Content-Length") {
        response += &format!("Content-Length: {}\r\n", res.body.len());
    }

    for (key, value) in res.headers {
        response += &format!("{}: {}\r\n", key, value);
    }

    response += "\r\n";

    let mut full_response = response.into_bytes();
    full_response.extend(res.body);

    full_response
}
