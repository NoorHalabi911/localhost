TIMEOUTS:
ncat localhost 8080 (wait 10 secs and it'll disconnect)
nc localhost 8080
curl -v telnet://127.0.0.1:8080  (Partial Get closes connection after 10 secs)

error pages:
  curl -i <http://localhost:8080/doesnotexist.html>

To test the **custom error pages** feature, you want to trigger each error condition and verify that:

- If a custom error page is configured and present in `html/`, it is served.
- If not, the default error message is served.

Here’s how you can test each error code:

---

## **1. Prepare Custom Error Pages**

- In your `html/` directory, create files named to match your config, e.g.:
  - `not_found.html` for 404
  - `forbidden.html` for 403
  - `internal_server_error.html` for 500
  - `bad_request.html` for 400
  - `method_not_allowed.html` for 405

  (The mapping is based on the value in `config.json`'s `error_msg` field, lowercased and spaces replaced with underscores.)

- Put a unique message in each file, e.g.:

  ```html
  <h1>Custom 404 Page</h1>
  ```

---

## **2. Trigger Each Error**

### **404 Not Found**

- Request a non-existent file, e.g.:

  ```
  curl http://localhost:8080/doesnotexist.html
  ```

- You should see your custom 404 page.

### **403 Forbidden**

- Try to access a file outside the allowed directory (if your static file handler is set up to block this).
- Or, temporarily change permissions on a file to make it unreadable, then request it.

### **500 Internal Server Error**

- Cause a CGI script to fail (e.g., by making it crash or return a non-zero exit code).
- Example:

  ```
  curl http://localhost:8080/some_script.py
  ```

  (if you have a Python CGI route and the script errors out)

### **400 Bad Request**

- Send a malformed HTTP request (e.g., missing the HTTP version or headers).
- Example:

  ```
  printf "BADREQUEST\r\n\r\n" | nc localhost 8080
  ```

### **405 Method Not Allowed**

- Use a method not allowed by the route, e.g.:

  ```
  curl -X DELETE http://localhost:8080/
  ```

  (if DELETE is not in the allowed methods for `/` in your config)

### **(Optional) 413 Payload Too Large**

- Try uploading a file larger than the allowed size.

---

## **3. Remove or Rename a Custom Error Page**

- Temporarily rename `not_found.html` to something else.
- Trigger a 404 again.
- You should see the default `<h1>404 Not Found</h1>` message.

---

## **4. Check HTTP Status Codes**

- Use `curl -i` to see the HTTP status code and headers:

  ```
  curl -i http://localhost:8080/doesnotexist.html
  ```

---

## **5. Automated Testing (Optional)**

You can write a simple script (bash, Python, etc.) to automate these tests and check the response body for your custom message.

---

**Summary Table:**

| Error | How to Trigger | What to Check |
|-------|---------------|---------------|
| 404   | Non-existent file | Custom 404 page or default |
| 403   | Forbidden file/path | Custom 403 page or default |
| 500   | CGI script error | Custom 500 page or default |
| 400   | Malformed request | Custom 400 page or default |
| 405   | Disallowed method | Custom 405 page or default |
| 413   | Large upload | Custom 413 page or default |

---

### **How to Test Directory Listing and Index File Support**

#### **1. Update Your Config**

Make sure your `config.json` for the relevant route includes the `directory_listing` field, e.g.:

```json
{
  "path": "/",
  "root": "./public",
  "index": "index.html",
  "methods": ["GET", "POST"],
  "directory_listing": true
}
```

- `"directory_listing": true` enables directory listing for `/`.
- `"index": "index.html"` means if `public/index.html` exists, it will be served for `/`.

#### **2. Prepare Your Directory**

- Create a `public/` directory in your project root (or whatever you set as `root`).
- Add some files and/or subdirectories inside `public/`.
- Optionally, add an `index.html` file inside `public/`.

#### **3. Test Scenarios**

- **Case 1: Directory with index file**
  - If `public/index.html` exists, navigating to `http://localhost:8080/` should show the contents of `index.html`.
- **Case 2: Directory without index file, listing enabled**
  - If you remove or rename `index.html`, navigating to `http://localhost:8080/` should show an HTML listing of files and folders in `public/`.
- **Case 3: Directory listing disabled**
  - Set `"directory_listing": false` in your config for `/`.
  - Navigating to `http://localhost:8080/` (with no `index.html`) should return a 403 Forbidden error.

#### **4. Test Subdirectories**

- Create a subdirectory, e.g., `public/images/`, and add files.
- Navigate to `http://localhost:8080/images/` to test directory listing and index file logic for subdirectories.

#### **5. Test File Access**

- Place a file, e.g., `public/test.txt`.
- Navigate to `http://localhost:8080/test.txt` to ensure it is served directly.

---
