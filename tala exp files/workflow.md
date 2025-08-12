### Request workflow: handle_request

This document breaks down how `handle_request` processes an incoming HTTP request end-to-end. The function lives in `src/main.rs` and is the central integration point for parsing, sessions, routing, and dispatching to specific handlers.

#### 1) Parse the request

- Input: raw bytes from the mio event loop.
- It calls `requests::parse_http_request` to obtain a `Request { method, path, version, headers, body }`.
- If parsing fails, returns `400 Bad Request` and, if configured, serves the custom 400 page.

Key points:

- Headers end at the first occurrence of `\r\n\r\n`.
- Bodies marked as `Transfer-Encoding: chunked` are decoded in `parse_http_request`.

#### 2) Manage session and Set-Cookie

- Extract `Cookie` header, call `SessionManager::get_or_create_session`.
- If the client has no valid `session_id`, a new session is created.
- The function prepares a `Set-Cookie: session_id=<id>; Path=/; HttpOnly` header to inject into the final response.

Why: Establishes a consistent session model (even for static requests) and demonstrates cookie handling.

#### 3) Route selection (longest prefix wins)

- Iterates `server_config.router` and selects the route with the longest `path` that matches the request path prefix.
- If no route matches, returns `404 Not Found` (custom page if configured).

Why: Supports nested routes while ensuring the most specific route handles the request.

#### 4) Redirection handling

- If the matched route has `redirection`, immediately returns an HTTP redirect.
- Status defaults to 302 unless a specific code is set (e.g., 301, 307, 308).
- Adds `Location: <target>` and `Set-Cookie` if needed.

Why: Decouple path migrations from application code via config.

#### 5) Method enforcement (405)

- Checks `route.methods` for the incoming method.
- If not allowed, returns `405 Method Not Allowed` and injects `Set-Cookie` if needed.

Why: Enforce per-route method policies from configuration.

#### 6) CGI execution (optional)

- If `route.cgi = [".<ext>", "script_name"]` and the path ends with `.<ext>`, the server will:
  - Build `script_path = format!("{}/{}", route.root, script_name)`.
  - Call `cgi::run_cgi_script(script_path, body_as_utf8, path_info)`.
  - On success: return `200 OK` with the script output (as `text/plain`).
  - On failure: return `500 Internal Server Error` (custom page if configured).

Why: Provide a simple way to generate dynamic responses.

#### 7) Upload handler (POST /upload)

- If method is `POST` and the matched route path is `/upload`:
  - Read `Content-Type` to extract the multipart boundary.
  - Call `upload_handler::handle_file_upload(body, content_type)` which:
    - Enforces a max payload size.
    - Validates `multipart/form-data; boundary=...`.
    - Optionally decodes chunked bodies.
    - Extracts `filename` and writes bytes to `uploads/<filename>`.
  - Build the response via `build_upload_response(result)`.
  - Inject `Set-Cookie` if needed by inserting it before the blank line.

Why: Implements a clean separation of concerns and keeps the router thin.

#### 8) DELETE handler

- If method is `DELETE`:
  - Compute `full_path = canonicalize(route.root + request.path)`.
  - Reject if the resolved path is outside `route.root` (prevents traversal).
  - Reject if the path is a directory (only files can be deleted).
  - Attempt `remove_file` and return:
    - `200 OK` on success
    - `404 Not Found` if missing
    - `500 Internal Server Error` on other errors
  - Inject `Set-Cookie` if needed.

Why: Provide safe file deletion under configured roots.

#### 9) Static file serving (default)

- Compute a relative path (strip the leading `/`).
- Call `static_file::read_static_file_with_listing(rel_path, route.root, route.index, route.directory_listing)`.
  - Returns `Ok(bytes)`, `NotFound`, `Forbidden`, or `DirectoryListing(html)`.
- For `Ok` and `DirectoryListing`, build a `200 OK` via `build_http_response`.
- For `NotFound` or `Forbidden`, build an error response, preferring custom pages.
- Inject `Set-Cookie` if needed by inserting the header before the blank line.

Why: Unifies static files, index resolution, and optional directory listing.

#### 10) No matching route (404)

- If no route matched in step 3, returns `404 Not Found` with custom page if available.

#### 11) Error pages lookup

- Helper `custom_error_body(code, config)` maps status code to a configured page name, resolving to `html/<name>.html`.
- Falls back to a simple inline HTML message when the file is missing.

Why: Customizes error surfaces without changing code.

#### 12) Set-Cookie injection logic

- After building a response, when a new session was created, the code inserts `Set-Cookie: session_id=...` before the `\r\n\r\n` (blank line) in the serialized response.

Why: Keep session creation orthogonal to specific handlers and always return the cookie when needed.

---

At a glance: control flow

1. Parse → 2. Session → 3. Route → 4. Redirect? → 5. 405? → 6. CGI? → 7. Upload? → 8. DELETE? → 9. Static → 10. 404

Throughout: Set-Cookie injection and custom error page lookup.
