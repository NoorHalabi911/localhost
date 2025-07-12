## **1. Multi-Port, Multi-Server** ✅TESTED

- **Test:** Start your server. Ensure it listens on all ports in `config.json`.
- **How:**  
  - `curl http://localhost:8080/`  
  - `curl http://localhost:9090/`  
  - Both should respond.

---

## **2. HTTP/1.1 Compliance** ✅TESTED

- **Test:** Use `curl -v` to check headers and protocol.
- **How:**  
  - `curl -v http://localhost:8080/`
  - Check for `HTTP/1.1` in the response.

---

## **3. GET, POST, DELETE Methods**

- **GET:**  
  - `curl http://localhost:8080/file.txt` ✅TESTED
- **POST (upload):**  
  - `curl -X POST -F 'file=@file.txt' http://localhost:8080/upload` ✅TESTED
  - `curl.exe -v -X POST -F "file=@file.txt" http://localhost:8080/upload` for a detailed curl
- **DELETE:**  
  - `curl -X DELETE http://localhost:8080/file.txt` ✅TESTED
  - File should be deleted.

---

## **4. File Uploads**

- **Test:** Upload a file and check it appears in the upload directory.
- **How:**  
  - `curl -F 'file=@test.txt' http://localhost:8080/upload`
  - Check the upload directory for `test.txt`.

---

## **5. Cookies and Sessions**

- **Test:**  
  - `curl -i http://localhost:8080/`  
  - Look for `Set-Cookie: session_id=...` in the response.
  - Reuse the cookie in a second request:
    - `curl -b 'session_id=...' http://localhost:8080/`
  - Should reuse the same session.

---

## **6. Custom Error Pages**

- **Test:**  
  - Request a non-existent file: `curl -i http://localhost:8080/doesnotexist`
  - Should return your custom 404 page.
  - Try forbidden access, method not allowed, etc., and check for custom error pages.

---

## **7. Directory Listing and Index File**

- **Test:**  
  - Remove `index.html` from a directory with `directory_listing: true`.
  - `curl http://localhost:8080/` should show an HTML file list.
  - Add `index.html` back; it should be served instead.

---

## **8. Redirections**

- **Test:**  
  - Add a route in config with `"redirection": { "target": "/new", "status": 301 }`.
  - `curl -i http://localhost:8080/old` should return a 301/302 and `Location: /new`.

---

## **9. CGI Support**

- **Test:**  
  - Add a route with a CGI extension (e.g., `.py`).
  - `curl http://localhost:8080/script.py`
  - Should execute the script and return its output.
  - Check that `PATH_INFO` is set correctly in the script output.

---

## **10. Timeouts**

- **Test:**  
  - Open a connection (e.g., with `telnet localhost 8080`), send part of a request, and wait >10 seconds.
  - The server should close the connection.

---

## **11. Chunked Transfer Encoding**

- **Test:**  
  - Use a tool like `curl` to upload with chunked encoding:
    - `curl -T test.txt -H "Transfer-Encoding: chunked" http://localhost:8080/upload`
  - The upload should succeed.

---

## **12. Status Codes**

- **Test:**  
  - For each error and success case, check the HTTP status code in the response (`curl -i ...`).

---

## **13. Stress and Memory Leak Testing**

- **Stress:**  
  - `siege -b http://localhost:8080/`
- **Memory:**  
  - Run your server with `valgrind` or use Rust’s `cargo valgrind`/`cargo check` tools.
- **Goal:**  
  - No crashes, no memory leaks, >99.5% availability.

---

## **14. Config File Features**

- **Test:**  
  - Change config (ports, error pages, methods, roots, redirections, etc.), restart server, and verify behavior.

---

## **15. Forbidden Features**

- **Test:**  
  - Review `Cargo.toml` and code: no `tokio`, `nix`, or other forbidden crates.

---

## **16. One Process, One Thread**

- **Test:**  
  - Use `htop` or `ps` to verify only one process/thread is running for the server.

---

## **17. Non-blocking I/O**

- **Test:**  
  - Confirm all I/O is via `mio`/epoll in code (already done).

---

### **Extra: Automated Test Script Example**

You can write a shell script to automate many of these checks using `curl` and `diff` for expected outputs.

---
