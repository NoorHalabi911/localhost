// # كود قراءة الملفات الثابتة من المسار المطلوب

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// تمثل نتيجة قراءة الملف: إما نجاح وفيه البايتات، أو خطأ وفيه رسالة
pub enum FileResponse {
    Ok(Vec<u8>),
    NotFound,
    Forbidden,
}

/// قراءة الملف الثابت من المسار المطلوب
pub fn read_static_file(request_path: &str) -> FileResponse {
    // 1. تحديد المسار الأساسي للملفات الثابتة
    let base_path = Path::new("public");

    // 2. إنشاء المسار الكامل للملف المطلوب
    let full_path = base_path.join(&request_path.trim_start_matches('/'));

    // 3. التحقق من وجود الملف
    let full_path = match fs::canonicalize(&full_path) {
        Ok(path) => path,
        Err(_) => return FileResponse::NotFound, // لا يوجد ملف
    };

    // 4. التحقق من المسار المطلوب
    if !full_path.starts_with(fs::canonicalize(base_path).unwrap()) {
        return FileResponse::Forbidden;
    }
    // 5. قراءة الملف
    match fs::read(&full_path) {
        Ok(contents) => FileResponse::Ok(contents),
        Err(_) => FileResponse::NotFound,
    }
}

pub fn build_http_response(file_response: FileResponse) -> Vec<u8> {
    match file_response {
        // 1. الملف موجود ويمكن قرائته
        FileResponse::Ok(contents) => {
            let status_line = "HTTP/1.1 200 OK\r\n";
            let content_type = "Content-Type: text/html\r\n";
            let content_length = format!("Content-Length: {}\r\n", contents.len());
            let headers = format!("{}{}{}\r\n", status_line, content_type, content_length);

            let response = headers.into_bytes();
            response
        }

        // 2. الملف غير موجود
        FileResponse::NotFound => {
            let body = b"<h1>404 Not Found</h1>";
            let headers = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(body);
            response
        }

        // 3. الملف موجود ولكن لا يمكن قرائته
        FileResponse::Forbidden => {
            let body = b"<h1>403 Forbidden</h1>";
            let headers = format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(body);
            response
        }
    }
}
