// # كود قراءة الملفات الثابتة من المسار المطلوب

use std::fs;
// use std::io;
use std::path::Path;

/// Result of attempting to read a static resource.
///
/// Why: Encapsulates the outcome so the caller can build an HTTP response
/// consistently.
/// How: Distinguishes success, not found, forbidden, and generated
/// directory listing HTML.
pub enum FileResponse {
    Ok(Vec<u8>),
    NotFound,
    Forbidden,
    DirectoryListing(String),
}

/// Read a static path relative to `base_path`, with index and optional listing.
///
/// Why: Safely serve files from a configured root and optionally list
/// directories.
/// How: Canonicalizes the target, prevents path traversal by enforcing that
/// the resolved path stays under `base_path`, serves `index` for directories if
/// present, or returns a generated directory listing when enabled.
pub fn read_static_file_with_listing(
    request_path: &str,
    base_path: &str,
    index: Option<&str>,
    directory_listing: bool,
) -> FileResponse {
    let base = Path::new(base_path);
    let full_path = base.join(&request_path.trim_start_matches('/'));
    let full_path = match fs::canonicalize(&full_path) {
        Ok(path) => path,
        Err(_) => return FileResponse::NotFound,
    };
    if !full_path.starts_with(fs::canonicalize(base).unwrap()) {
        return FileResponse::Forbidden;
    }
    if full_path.is_dir() {
        // Try index file first
        if let Some(index_file) = index {
            let index_path = full_path.join(index_file);
            if index_path.exists() && index_path.is_file() {
                return match fs::read(&index_path) {
                    Ok(contents) => FileResponse::Ok(contents),
                    Err(_) => FileResponse::NotFound,
                };
            }
        }
        // Directory listing
        if directory_listing {
            let mut html = String::from("<html><body><h1>Directory listing</h1><ul>");
            if let Ok(entries) = fs::read_dir(&full_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let display = if entry.path().is_dir() {
                        format!("{}/", name)
                    } else {
                        name.clone()
                    };
                    html.push_str(&format!("<li><a href=\"{}\">{}</a></li>", display, display));
                }
            }
            html.push_str("</ul></body></html>");
            return FileResponse::DirectoryListing(html);
        } else {
            return FileResponse::Forbidden;
        }
    }
    // Serve file
    match fs::read(&full_path) {
        Ok(contents) => FileResponse::Ok(contents),
        Err(_) => FileResponse::NotFound,
    }
}

/// Build an HTTP response corresponding to `FileResponse`.
///
/// Why: Central mapping from file read outcome to HTTP status/body.
/// How: Emits minimal headers (defaults to `text/html`) and sets
/// `Content-Length` to match the payload.
pub fn build_http_response(file_response: FileResponse) -> Vec<u8> {
    match file_response {
        // 1. الملف موجود ويمكن قرائته
        FileResponse::Ok(contents) => {
            let status_line = "HTTP/1.1 200 OK\r\n";
            let content_type = "Content-Type: text/html\r\n";
            let content_length = format!("Content-Length: {}\r\n", contents.len());
            let headers = format!("{}{}{}\r\n", status_line, content_type, content_length);

            let mut response = headers.into_bytes();
            response.extend_from_slice(&contents);
            response
        }
        FileResponse::NotFound => {
            let body = b"<h1>404 Not Found</h1>".to_vec();
            let headers = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(&body);
            response
        }
        FileResponse::Forbidden => {
            let body = b"<h1>403 Forbidden</h1>".to_vec();
            let headers = format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(&body);
            response
        }
        FileResponse::DirectoryListing(html) => {
            let body = html.as_bytes();
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(body);
            response
        }
    }
}
