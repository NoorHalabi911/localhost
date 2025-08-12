# Localhost — Non-blocking Rust HTTP Server

A lightweight HTTP/1.1 server written in Rust using `mio` for non-blocking I/O.

* (when we read from a socket or write to a file, a blocking I/O will let threads wait until the operation finishes). when we have multiple clients we will nead for each client a thread to avoid blocking othes.
* non blocking: IO calls immediately return if there is no data. and checks periodically or gets notified when the socket is ready to read/write. single thread handles many connections.
* mio = metal io . low level event driven io library sam as epoll.

our server supports multiple ports, static files, file uploads (multipart/form-data), simple CGI, configurable routing, directory listing, redirections, basic sessions/cookies, and request timeouts.

### Project structure

* `src/main.rs`: Event loop (mio), multi-port listeners, request buffering, timeout handling, and centralized `handle_request` that routes to the correct handler.
* `src/serverConfig.rs`: Configuration types (servers, routes, redirections).
* `src/requests.rs`: HTTP request parsing and response building; decodes chunked bodies.
* `src/static_file.rs`: Static file reading, index handling, directory listing, and response builder.
* `src/upload_handler.rs`: Multipart/form-data parsing, file saving, size limit, and upload responses.
* `src/cgi.rs`: Runs a Python script as CGI (`python` must be available on PATH).
* `src/session_manager.rs`: In-memory session store; sets/reuses `session_id` cookie.
* `src/config.json`: JSON configuration for ports, routes, error pages, etc.
* `public/`: Default static assets and CGI script example (`py.py`).

### Quick start

Build and run:

```bash
cargo run
```

By default (from `src/config.json`) the server listens on:

* `127.0.0.1:8080`
* `127.0.0.1:9090`

Open a browser at `http://127.0.0.1:8080/`.

### Configuration (`src/config.json`)

Key fields:

* `server_address`: Array of `{ ip, port }` pairs to listen on
* `max_body_size`: Maximum request body size (bytes)
* `error_msg`: Map from status code to page name (looked up as `html/<name>.html`)
* `router`: Array of route objects:
  * `path`: URL prefix to match (longest prefix wins)
  * `methods`: Allowed methods (e.g., `GET`, `POST`, `DELETE`)
  * `root`: Filesystem root for that route
  * `index`: Optional default file for directories
  * `directory_listing`: Optional `true/false`
  * `redirection`: Optional `{ target, status }`
  * `cgi`: Optional `[".<ext>", "script_name"]` — when path ends with `.<ext>`, run `root/script_name` as CGI

Example (excerpt):

```json
{
  "server_name": "Main.Server",
  "server_address": [{ "ip": "127.0.0.1", "port": 8080 }],
  "router": [
    { "path": "/", "root": "./public", "index": "index.html", "methods": ["GET","POST","DELETE"], "directory_listing": true },
    { "path": "/upload", "root": "/", "methods": ["POST"] },
    { "path": "/old", "root": "./public", "methods": ["GET"], "redirection": { "target": "/new", "status": 301 } },
    { "path": "/new", "root": "./public", "index": "index.html", "methods": ["GET"] },
    { "path": "/cgi-bin/py.py", "root": "./public", "methods": ["GET"], "cgi": [".py", "py.py"] }
  ]
}
```

### Default routes in this repo

* `GET /` → serves `./public/index.html` (or directory listing if enabled)
* `GET /new` → serves `./public/new/index.html`
* `GET /old` → 301 redirect to `/new`
* `POST /upload` → Accepts multipart/form-data and writes files to `./uploads`
* `GET /cgi-bin/py.py` → Runs `./public/py.py` via Python; response is returned as plain text
* `DELETE /path/to/file` → Deletes a file under the matched route `root` (not directories)

### Usage examples (curl)

* Static file:

```bash
curl -i http://127.0.0.1:8080/
```

* Directory listing (when enabled):

```bash
curl -i http://127.0.0.1:8080/
```

* Redirection:

```bash
curl -i http://127.0.0.1:8080/old
```

* File upload (multipart/form-data):

```bash
curl -i -X POST http://127.0.0.1:8080/upload \
  -F "file=@public/index.html"
```

Uploaded files land in `./uploads/`.

* Delete a file (under the route root):

```bash
curl -i -X DELETE http://127.0.0.1:8080/uploads/file.txt
```

* CGI (Python):

```bash
curl -i http://127.0.0.1:8080/cgi-bin/py.py
```

### Sessions and cookies

* Every request is checked for a `session_id` cookie.
* If missing/invalid, a new session is created and `Set-Cookie: session_id=<id>; Path=/; HttpOnly` is added to the response.
* Session data is in-memory and extensible; currently used to demonstrate cookie flow.

### Error handling

* Custom error pages can be configured via `error_msg` (e.g., `404: "Not Found"`).
* The server looks up `html/<mapped_name>.html` (e.g., `html/not_found.html`). If not found, falls back to a simple default message.

### Timeouts

* Idle connections are closed after 30 seconds (configurable in code via `CLIENT_TIMEOUT`).

### Integration member report (what was integrated)

the following work was done to connect and harden the system end-to-end:

* **Centralized routing**: Implemented `handle_request` to parse incoming bytes, manage sessions, match the longest route prefix, and dispatch to the correct handler (static, upload, CGI, delete, or redirection).
* **Config-driven behavior**: Enabled routing, allowed methods (405 on mismatch), directory listing, index files, redirections, route roots, and CGI from `src/config.json`.
* **Session/cookie flow**: Added `session_id` cookie creation/reuse and injection into responses, not only static files.
* **Upload integration**: Wired POST `/upload` to `upload_handler`, with multipart parsing, size limit, and response builder.
* **CGI integration**: Hooked `cgi.rs` into routing to execute Python scripts from the configured route root with `PATH_INFO` and request body.
* **Custom error pages**: Lookups based on `error_msg` mapping; serve 403/404/405/500 where appropriate.
* **DELETE support**: Safe file deletion under the route root; guarded against directory deletion and path traversal.
* **Timeouts**: Added idle connection timeout (30s) in the mio event loop; auto-closes stalled clients.
* **Chunked requests**: Decoding in `requests.rs` and upload path for practical chunked payloads.
