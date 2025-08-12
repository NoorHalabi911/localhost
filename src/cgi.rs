// use std::error::Error;
use std::io::{self, Write};
use std::process::Command;
/// Execute a Python CGI script and return its stdout as the response body.
///
/// Why: Provide simple server-side execution for dynamic responses.
/// How: Spawns `python <script_path>`, sets `PATH_INFO`, writes the HTTP
/// request body to the child's stdin, and captures stdout.
pub fn run_cgi_script(script_path: &str, body: &str, path_info: &str) -> io::Result<String> {
    let output = Command::new("python")
        .arg(script_path)
        .env("PATH_INFO", path_info)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(body.as_bytes())?;
            }
            let output = child.wait_with_output()?;
            Ok(output)
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "CGI script failed"))
    }
}
