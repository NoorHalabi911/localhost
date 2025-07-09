Let's break down your requirements and current state, then create a clear plan to connect all components and guide you to a fully functional HTTP server in Rust, as specified.

---

## 1. **Project Structure Discovery**

From your attached files and folder structure, you have:

- `src/main.rs`: Main server logic, uses `mio` for evented I/O (epoll equivalent).
- `src/cgi.rs`: Presumably for CGI handling.
- `src/serverConfig.rs`: Server configuration parsing and types.
- `src/static_file.rs`: Static file serving logic.
- `src/upload_handler.rs`: File upload handling.
- `src/html/`: HTML files (static content).
- `src/config.json`: Configuration file (JSON).
- `src/py.py`: Python script, likely for CGI.
- `roadmap/`: Images, probably for planning.
- `t.tldr`: Large text file, possibly documentation.

---

## 2. **Current State Analysis**

Your `main.rs` already:

- Parses a config file.
- Listens on multiple addresses/ports.
- Uses `mio` (epoll-based) for non-blocking I/O.
- Handles basic HTTP GET requests for static HTML files.

**What’s missing or incomplete:**

- Full HTTP/1.1 compliance (methods, headers, chunked encoding, etc.).
- POST, DELETE, and file upload support.
- Cookie/session management.
- Error page handling (400, 403, 404, 405, 413, 500).
- CGI execution based on file extension.
- Route configuration (methods, redirection, root dir, default file, CGI, directory listing).
- Timeout handling for requests.
- Chunked/unchunked request management.
- Only one epoll call per client/server communication.
- All reads/writes via epoll.
- No use of server crates like tokio/nix.

---

## 3. **Step-by-Step Plan**

### **A. Configuration**

- [ ] Expand `serverConfig.rs` to support all config options (routes, error pages, body size, etc.).
- [ ] Parse and validate `config.json` for all required fields.

### **B. HTTP Protocol Handling**

- [ ] Parse HTTP/1.1 requests (GET, POST, DELETE, headers, cookies, chunked encoding).
- [ ] Build correct HTTP responses (status, headers, body, chunked/unchunked).
- [ ] Implement timeout logic for requests.

### **C. Routing**

- [ ] Route requests based on config (root dir, default file, allowed methods, redirection, CGI, directory listing).

### **D. Static Files & Uploads**

- [ ] Serve static files from configured directories.
- [ ] Handle file uploads (multipart/form-data, size limits).

### **E. CGI**

- [ ] Execute CGI scripts based on file extension (e.g., `.py`).
- [ ] Set up environment variables (PATH_INFO, etc.).
- [ ] Fork process for CGI, handle input/output.

### **F. Error Handling**

- [ ] Serve custom error pages for 400, 403, 404, 405, 413, 500.

### **G. Cookies & Sessions**

- [ ] Parse and set cookies.
- [ ] Implement basic session management (in-memory or file-based).

### **H. Epoll/Non-blocking I/O**

- [ ] Ensure all I/O is non-blocking and goes through `mio` (epoll).
- [ ] Only one epoll call per client/server communication.

---

## 4. **How We'll Proceed**

1. **Review and expand your config system** (`serverConfig.rs` and `config.json`).
2. **Upgrade your HTTP parser** to handle all required methods, headers, and chunked encoding.
3. **Implement routing logic** based on config.
4. **Integrate static file, upload, and CGI handling** into the main event loop.
5. **Add error handling, cookies, and sessions**.
6. **Test and refine for robustness and compliance**.

---

## 5. **Next Step: Configuration System**

**Let's start by reviewing your `serverConfig.rs` and `config.json` to ensure they can support all the required configuration options.**

**Would you like to:**

- See a proposed schema for `config.json` and the corresponding Rust structs?
- Or, would you like to review your current `serverConfig.rs` and `config.json` first?

**Let me know your preference, and I'll guide you step by step!**

---

Great! Let's make a clear, actionable plan to connect all your components and get your HTTP server fully working, step by step.

---

## **1. Current State Recap**

You have these main modules:

- `main.rs`: Entry point, event loop, basic request handling.
- `cgi.rs`: For CGI script execution.
- `serverConfig.rs`: For config parsing and server settings.
- `static_file.rs`: For serving static files.
- `upload_handler.rs`: For handling file uploads.

But, as you said, **most functions are not yet connected** in the main event loop. The server currently only serves static HTML files for GET requests.

---

## **2. What Needs to Happen**

To connect everything, your event loop (in `run_mio_server`) must:

- Parse the HTTP request (method, path, headers, body).
- Route the request:
  - Serve static files (GET).
  - Handle file uploads (POST).
  - Run CGI scripts (e.g., for `.py` files).
  - Return error pages for unsupported methods, not found, etc.
- Set correct HTTP status, headers, and body.
- Support multiple methods (GET, POST, DELETE).
- Handle cookies and sessions.
- Use config for routing, error pages, etc.

---

## **3. Step-by-Step Connection Plan**

### **A. Centralize Request Parsing**

- Parse the HTTP request fully (method, path, headers, body).
- Decide what to do based on method and path.

### **B. Routing Logic**

- If path matches a static file, use `static_file.rs`.
- If path is for upload, use `upload_handler.rs`.
- If file extension matches CGI, use `cgi.rs`.
- If not found or method not allowed, serve error page.

### **C. Integrate All Components**

- Call the right handler from the event loop.
- Build the HTTP response (status, headers, body).
- Write the response back to the client.

### **D. Use Config**

- Use `serverConfig.rs` to get routing, error pages, etc.

---

## **4. What You Need to Do Next**

### **Step 1: Centralize Request Handling**

In your event loop (`run_mio_server`), after you parse the request, call a new function, e.g., `handle_request`, that:

- Takes the parsed request and config.
- Returns a response (status, headers, body).

**Example:**

```rust
let response = handle_request(&request, &server_config);
conn.write_buffer = response.into_bytes();
```

### **Step 2: Implement `handle_request`**

This function should:

- Match the method (GET, POST, DELETE).
- Use the path to decide:
  - Static file? → `static_file.rs`
  - Upload? → `upload_handler.rs`
  - CGI? → `cgi.rs`
  - Not found? → error page

### **Step 3: Connect Each Module**

- **static_file.rs**: Serve files, return 404 if not found.
- **upload_handler.rs**: Handle POST with file upload, check size limits.
- **cgi.rs**: If path ends with `.py`, run CGI and return output.
- **serverConfig.rs**: Use config to get root dirs, error pages, allowed methods, etc.

### **Step 4: Error Handling**

- If anything fails, serve the correct error page (from config).

---

## **5. Example Flow (Pseudocode)**

```rust
fn handle_request(request: &HttpRequest, config: &ServerConfig) -> HttpResponse {
    match request.method {
        "GET" => {
            if is_cgi(&request.path) {
                run_cgi_script(...)
            } else {
                serve_static_file(...)
            }
        }
        "POST" => {
            if is_upload_path(&request.path) {
                handle_file_upload(...)
            } else if is_cgi(&request.path) {
                run_cgi_script(...)
            } else {
                error_response(405)
            }
        }
        "DELETE" => {
            // Implement delete logic or error
        }
        _ => error_response(405)
    }
}
```

---

## **6. Next Steps for You**

1. **Create a `handle_request` function** that takes a parsed request and config, and returns a response.
2. **Update your event loop** to use this function for every request.
3. **Wire up each module** (static file, upload, CGI) inside `handle_request`.
4. **Test each route** (static, upload, CGI, error).

---

## **Would you like:**

- A concrete code example for the `handle_request` function and how to call your modules?
- Or, a checklist of what to implement in each module?

**Let me know your preference, and I’ll guide you with code and integration details!**

---

Ran tool
Read file: src/serverConfig.rs
Read file: src/cgi.rs
Read file: src/static_file.rs
Read file: src/upload_handler.rs
Read file: src/main.rs
Here’s how to **connect all your components** for a fully working HTTP server, using your current code and modules. This will give you a single-threaded, non-blocking, epoll-based HTTP/1.1 server that supports static files, uploads, CGI, and error handling.

---

## **1. Centralize Request Handling**

**Create a function** (e.g., `handle_request`) that:

- Parses the HTTP request (method, path, headers, body).
- Uses the config to route the request.
- Calls the correct handler: static file, upload, CGI, or error.
- Returns a full HTTP response as bytes.

---

## **2. Integrate All Modules in the Event Loop**

**In your event loop** (`run_mio_server`), after reading a full request, call `handle_request` and set `conn.write_buffer` to its result.

---

## **3. Example Integration Plan**

### **A. Add a Request Parser**

You need to parse:

- Method (GET, POST, DELETE)
- Path
- Headers (Content-Type, Cookie, etc.)
- Body (for POST)

You can write a simple parser or use a struct like:

```rust
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
```

### **B. Implement `handle_request`**

This function will:

- Use the config to find the matching route.
- Check allowed methods.
- If static file: call `read_static_file` and `build_http_response`.
- If upload: call `handle_file_upload` and `build_upload_response`.
- If CGI: call `run_cgi_script`.
- If error: return the correct error page (from config).

### **C. Update the Event Loop**

Replace your current logic in `run_mio_server` that just serves static files with a call to `handle_request`.

---

## **4. Example: `handle_request` Skeleton**

```rust
fn handle_request(
    request: &HttpRequest,
    config: &ServerConfig,
) -> Vec<u8> {
    // 1. Find matching route in config
    let route = config.router.iter().find(|r| request.path.starts_with(&r.path));
    if let Some(route) = route {
        // 2. Check allowed methods
        if !route.methods.contains(&request.method) {
            return build_error_response(405, config);
        }
        // 3. CGI
        if let Some((ext, cgi_path)) = &route.cgi {
            if request.path.ends_with(ext) {
                // Run CGI
                let cgi_result = run_cgi_script(
                    cgi_path,
                    std::str::from_utf8(&request.body).unwrap_or(""),
                    &request.path,
                );
                return match cgi_result {
                    Ok(output) => build_cgi_response(output),
                    Err(_) => build_error_response(500, config),
                };
            }
        }
        // 4. Upload
        if request.method == "POST" && request.headers.get("content-type").map(|v| v.starts_with("multipart/form-data")).unwrap_or(false) {
            let upload_result = handle_file_upload(&request.body, request.headers.get("content-type").unwrap());
            return build_upload_response(upload_result);
        }
        // 5. Static file
        let file_response = read_static_file(&request.path);
        return build_http_response(file_response);
    }
    // 6. No route matched
    build_error_response(404, config)
}
```

---

## **5. Update the Event Loop**

Replace this part in `run_mio_server`:

```rust
let request = String::from_utf8_lossy(&conn.read_buffer[..pos]);
let path = request
    .lines()
    .next()
    .and_then(|line| line.split_whitespace().nth(1))
    .unwrap_or("/");

let file = if path == "/" {
    handle_path("def")
} else {
    handle_path(path.trim_start_matches('/'))
};

conn.write_buffer = match file {
    Ok(content) => format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        content.len(),
        content
    ).into_bytes(),
    Err(_) => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec(),
};
```

**With:**

```rust
let http_request = parse_http_request(&conn.read_buffer);
conn.write_buffer = handle_request(&http_request, &server_config);
```

---

## **6. Error Handling**

Use your config’s `error_msg` to serve custom error pages for 400, 403, 404, 405, 413, 500.

---

## **7. Next Steps**

1. **Write a simple HTTP request parser** (method, path, headers, body).
2. **Implement `handle_request`** as above, calling your modules.
3. **Replace the static file logic in your event loop** with a call to `handle_request`.
4. **Test each feature** (static, upload, CGI, error).

---

## **Would you like:**

- A ready-to-paste code for the request parser and integration?
- Or, a step-by-step guide for each part?

**Let me know your preference, and I’ll provide the code or detailed steps!**

---

This error:

```
thread 'main' panicked at src/main.rs:145:52:
file don't exist: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

means that your code tried to open `./config.json` but couldn't find it **relative to the current working directory**.

### Why does this happen?

- Your code runs:  

  ```rust
  let file = fs::read_to_string("./config.json").expect("file don't exist");
  ```

- The working directory (as printed) is:  

  ```
  wd /Users/adam-07-01/Desktop/LH
  ```

- But your `config.json` is actually in `src/config.json`, **not** in the project root.

### Solution

You have two options:

#### **Option 1: Move the file**

Move `config.json` from `src/` to the project root (`/Users/adam-07-01/Desktop/LH/config.json`).

#### **Option 2: Change the path in your code**

Change:

```rust
let file = fs::read_to_string("./config.json").expect("file don't exist");
```

to:

```rust
let file = fs::read_to_string("./src/config.json").expect("file don't exist");
```

---

**Recommendation:**  
For a config file, it's best to keep it in the project root, so move it up one directory.

---

### **Summary Table**

| Current Location         | Code Path Needed         |
|-------------------------|-------------------------|
| `/Users/adam-07-01/Desktop/LH/config.json` | `"./config.json"`         |
| `/Users/adam-07-01/Desktop/LH/src/config.json` | `"./src/config.json"`    |

---

**Fix this and your error will be resolved!**
