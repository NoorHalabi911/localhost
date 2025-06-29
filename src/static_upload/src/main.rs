mod static_file;
mod upload_handler;

use std::fs;
use static_file::{read_static_file, build_http_response, FileResponse};
use upload_handler::{handle_file_upload, build_upload_response};

fn main() {
    // مثال يدوي: كأنك استقبلت طلب من المتصفح

    // 1. طلب GET لملف ثابت
    let method = "GET";
    let path = "/index.html";

    if method == "GET" {
        let result = read_static_file(path);
        let response = build_http_response(result);
        println!("--- GET Response ---");
        println!("{}", String::from_utf8_lossy(&response));
    }

    // 2. طلب POST لرفع ملف
    let method = "POST";
    let content_type = "multipart/form-data; boundary=----XYZ";
    let fake_body = fs::read("tests/upload_example_body.txt").unwrap_or_default();

    if method == "POST" {
        let result = handle_file_upload(&fake_body, content_type);
        let response = build_upload_response(result);
        println!("--- POST Response ---");
        println!("{}", String::from_utf8_lossy(&response));
    }
}
