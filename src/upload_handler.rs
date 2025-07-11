// # كود التعامل مع POST ورفع الملفات

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub enum UploadResult {
    Ok,
    PayloadTooLarge,
    BadRequest,
    InternalError,
}

const MAX_UPLOAD_SIZE: usize = 5 * 1024 * 1024; // 5MB

pub fn handle_file_upload(body: &[u8], content_type: &str) -> UploadResult {
    // 1. التحقق من حجم البيانات
    if body.len() > MAX_UPLOAD_SIZE {
        return UploadResult::PayloadTooLarge;
    }

    // 2. التحقق من نوع المحتوى

    //multipart/form-data اللي هو النوع المستخدم في رفع الملفات
    if !content_type.starts_with("multipart/form-data; boundary=") {
        return UploadResult::BadRequest;
    }

    // 3. نطلع الباوندري (الفاصل بين الأجزاء)
    let boundary = match content_type.split("boundary=").nth(1) {
        Some(b) => format!("--{}", b),
        None => return UploadResult::BadRequest,
    };

    // 5. تحويل الجسم إلى سلسلة
    // Support chunked transfer encoding (decode if needed)
    let body_bytes = if content_type.contains("chunked") {
        match decode_chunked_body(body) {
            Ok(decoded) => decoded,
            Err(_) => return UploadResult::BadRequest,
        }
    } else {
        body.to_vec()
    };
    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(_) => return UploadResult::BadRequest,
    };

    // 6. التحقق من وجود الجزء المطلوب
    let parts: Vec<&str> = body_str.split(&boundary).collect();

    for part in parts {
        if part.contains("filename=\"") {
            // 4. نجيب اسم الملف
            let filename_start = part.find("filename=\"").unwrap() + 10;
            let filename_end = part[filename_start..].find('"').unwrap() + filename_start;
            let filename = &part[filename_start..filename_end];

            // 5. نحدد بداية المحتوى بعد الرأس الفارغ
            let content_start = part.find("\r\n\r\n").unwrap() + 4;
            let content = &part.as_bytes()[content_start..];

            // 6. نجهز المسار
            let filepath = Path::new("uploads").join(filename);

            // 7. نكتب الملف
            if let Ok(mut file) = File::create(filepath) {
                if file.write_all(content).is_ok() {
                    return UploadResult::Ok;
                } else {
                    return UploadResult::InternalError;
                }
            } else {
                return UploadResult::InternalError;
            }
        }
    }

    UploadResult::BadRequest
}

/// /// تنشئ رد HTTP بناءً على نتيجة رفع الملف
pub fn build_upload_response(result: UploadResult) -> Vec<u8> {
    match result {
        UploadResult::Ok => {
            let body = "<h1>✅ File uploaded successfully!</h1>".as_bytes();
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(body);
            response
        }
        UploadResult::PayloadTooLarge => {
            let body = b"<h1>413 Payload Too Large</h1>";
            let headers = format!(
                "HTTP/1.1 413 Payload Too Large\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(body);
            response
        }
        UploadResult::BadRequest => {
            let body = b"<h1>400 Bad Request</h1>";
            let headers = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(body);
            response
        }
        UploadResult::InternalError => {
            let body = b"<h1>500 Internal Server Error</h1>";
            let headers = format!(
                "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let mut response = headers.into_bytes();
            response.extend_from_slice(body);
            response
        }
    }
}

// Helper to decode chunked transfer encoding
fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, ()> {
    let mut decoded = Vec::new();
    let mut i = 0;
    while i < body.len() {
        // Find the next CRLF
        let crlf = match body[i..].windows(2).position(|w| w == b"\r\n") {
            Some(pos) => i + pos,
            None => return Err(()),
        };
        let len_str = std::str::from_utf8(&body[i..crlf]).map_err(|_| ())?;
        let chunk_size = usize::from_str_radix(len_str.trim(), 16).map_err(|_| ())?;
        if chunk_size == 0 {
            break;
        }
        i = crlf + 2;
        if i + chunk_size > body.len() {
            return Err(());
        }
        decoded.extend_from_slice(&body[i..i + chunk_size]);
        i += chunk_size + 2; // skip chunk and trailing CRLF
    }
    Ok(decoded)
}
