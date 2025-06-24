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
if !content_type.starts_with("multipart/form-data; boundary=") {
    return UploadResult::BadRequest;
}


// 3. نطلع الباوندري (الفاصل بين الأجزاء)
let boundary = match content_type.split("boundary=").nth(1){
    Some(b) => format!("--{}",b),
    None => return UploadResult::BadRequest,
};

// 4. تقسيم الجسم حسب الباوندري
let section = body.split(|window|{
window == &b'\r' || window == &b'\n'
}).collect::<Vec<&[u8]>>();

// 5. تحويل الجسم إلى سلسلة
let body_str = match std::str::from_utf8(body){
    Ok(s) => s,
    Err(_) => return UploadResult::BadRequest,
};

// 6. التحقق من وجود الجزء المطلوب
let parts: Vec<&[u8]> = body.split(|b| b"\r\n".contains(b)).collect();


for part in parts{
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







UploadResult::InternalError

}