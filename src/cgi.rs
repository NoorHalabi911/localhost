use std::error::Error;                  // Brings the generic Error trait into scope (not used in this snippet).
use std::io::{self, ErrorKind, Write};  // io::Result, ErrorKind enum, and the Write trait for writing to stdin.
use std::process::Command;              // Command lets you spawn external processes (like running Python).

pub fn run_cgi_script(                  // Public function you can call from other modules.
    script_path: &str,                  // Path to the Python CGI script file.
    body: &str,                         // The data to send to the script's standard input (e.g., HTTP body).
    path_info: &str                     // Value for the PATH_INFO environment variable (CGI convention).
) -> io::Result<String> {               // Returns Ok(output_string) or Err(io::Error).

    let output = Command::new("python") // Start building a command: run the "python" executable.
        .arg(script_path)               // First argument to python: the script to execute.
        .env("PATH_INFO", path_info)    // Set env var PATH_INFO for the child process (CGI uses this).
        .stdin(std::process::Stdio::piped())   // Ask for a writable pipe to the child's stdin.
        .stdout(std::process::Stdio::piped())  // Ask for a readable pipe from the child's stdout.
        .spawn()                        // Actually start the process (non-blocking).
        .and_then(|mut child| {         // If spawn succeeded, continue with the running child process.
            if let Some(stdin) = child.stdin.as_mut() { // Get a mutable handle to child's stdin, if present.
                stdin.write_all(body.as_bytes())?;      // Write all of `body` bytes into the script's stdin.
            }                                           // (If no stdin, just skip writing.)
            let output = child.wait_with_output()?;     // Block until the script exits; collect stdout/stderr.
            Ok(output)                                  // Return the collected Output to the outer scope.
        })?;                                            // Propagate any error from spawn/write/wait using `?`.

    if output.status.success() {                        // Check if the child exited with status code 0.
        Ok(String::from_utf8_lossy(&output.stdout)      // Convert stdout bytes to UTF-8 (lossy if invalid).
            .to_string())                               // Turn Cow<str> into owned String.
    } else {                                            // Non-zero exit status → treat as failure.
        Err(io::Error::new(                             // Create an I/O error to return.
            io::ErrorKind::Other,                       // Generic "Other" error kind.
            "CGI script failed"                         // Error message (doesn't include stderr).
        ))
    }
}
