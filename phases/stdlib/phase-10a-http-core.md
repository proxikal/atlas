# Phase 10a: HTTP Core - Request & Response Basics

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase-09b complete (datetime), Result types, basic security system

**Verification:**
```bash
grep -r "dateTimeFormat" crates/atlas-runtime/src/stdlib/
ls crates/atlas-runtime/src/value.rs  # Result type exists
cargo test -p atlas-runtime
```

**What's needed:**
- Phase-09b complete
- Result type in Value enum
- reqwest crate (blocking mode)

**If missing:** Complete phase-09b first.

---

## Objective
Implement core HTTP client functionality with request building, response handling, and basic GET/POST operations - laying foundation for web requests and API calls.

## Scope
**This phase (10a):** HTTP request/response types, GET/POST operations, basic error handling
**Next phase (10b):** All HTTP methods, permission integration, timeouts, advanced features

## Files
**Create:** `crates/atlas-runtime/src/stdlib/http.rs` (~400 lines)
**Update:** `crates/atlas-runtime/Cargo.toml` (add reqwest with blocking feature)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register functions)
**Create:** `crates/atlas-runtime/tests/http_core_tests.rs` (~300 lines)

## Dependencies
- reqwest = { version = "0.12", features = ["blocking", "json"] }
- Phase-09b complete (datetime for headers)
- Result type for error handling

## Implementation

### 1. HTTP Request Type
**Value variant:**
- Add `HttpRequest` variant to Value enum (stores request configuration)
- Representation: HashMap with {method, url, headers, body, timeout}

**Functions:**
- `httpRequest(method: string, url: string)` - Create HTTP request
- `httpRequestGet(url: string)` - Create GET request (convenience)
- `httpRequestPost(url: string, body: string)` - Create POST request (convenience)
- `httpSetHeader(request: HttpRequest, key: string, value: string)` - Add header
- `httpSetBody(request: HttpRequest, body: string)` - Set body
- `httpSetTimeout(request: HttpRequest, seconds: number)` - Set timeout

**Behavior:**
- Request is immutable (each function returns new request)
- URL validation (must start with http:// or https://)
- Method validation (GET, POST only in this phase)
- Default timeout: 30 seconds
- Default headers: User-Agent: Atlas/0.1

### 2. HTTP Response Type
**Value variant:**
- Add `HttpResponse` variant to Value enum
- Stores status code, headers, body, url

**Functions:**
- `httpStatus(response: HttpResponse)` - Get status code (number)
- `httpBody(response: HttpResponse)` - Get body as string
- `httpHeader(response: HttpResponse, key: string)` - Get header value
- `httpHeaders(response: HttpResponse)` - Get all headers as HashMap
- `httpUrl(response: HttpResponse)` - Get final URL (after redirects)
- `httpIsSuccess(response: HttpResponse)` - Check if status 200-299

**Behavior:**
- Response is immutable
- Headers stored as HashMap (case-insensitive keys)
- Body stored as String
- Status code as Number

### 3. HTTP Client Functions
**Core operations:**
- `httpSend(request: HttpRequest)` - Execute request, return Result<HttpResponse>
- `httpGet(url: string)` - Simple GET request
- `httpPost(url: string, body: string)` - Simple POST request
- `httpPostJson(url: string, json: JsonValue)` - POST JSON data

**Behavior:**
- All blocking (synchronous) operations
- Follow redirects by default (max 10)
- Return Result type (Ok with response, Err with error message)
- Network errors return Err with descriptive message
- Timeout errors return Err with timeout message

### 4. Error Handling
**Error types:**
- Network errors (connection refused, DNS failure)
- Timeout errors (request exceeded timeout)
- Invalid URL errors (malformed URL)
- HTTP errors (status >= 400)

**Behavior:**
- All errors wrapped in Result::Err
- Error messages descriptive and actionable
- HTTP errors include status code in message

### 5. JSON Integration
**Functions:**
- `httpParseJson(response: HttpResponse)` - Parse response body as JSON
- `httpPostJson(url: string, json: JsonValue)` - POST with Content-Type: application/json

**Behavior:**
- Use existing JSON parsing from phase-04
- Set Content-Type header automatically for JSON
- Parse errors return descriptive error messages

## Tests (25 tests minimum, 100% pass rate)

### Request Building Tests (8):
1. Create GET request with URL
2. Create POST request with body
3. Set custom header on request
4. Set multiple headers
5. Set request timeout
6. Default timeout is 30 seconds
7. Invalid URL error (no protocol)
8. Invalid method error (only GET/POST allowed)

### Response Handling Tests (7):
1. Parse response status code
2. Parse response body
3. Get response header by name
4. Get all response headers
5. Check if response is success (200)
6. Check if response is error (404)
7. Get final URL after redirects

### HTTP Operations Tests (10):
1. Simple GET request to httpbin.org
2. GET request with custom headers
3. Simple POST request with body
4. POST JSON data
5. Parse JSON response
6. Handle 404 error
7. Handle network timeout
8. Handle invalid URL
9. Handle connection error (invalid host)
10. Follow redirects

**Note:** Tests use httpbin.org or local mock server for real HTTP testing.

## Acceptance Criteria
- ✅ HttpRequest and HttpResponse types added to Value enum
- ✅ httpRequest creates request with method and URL
- ✅ httpSetHeader, httpSetBody, httpSetTimeout work correctly
- ✅ httpSend executes request and returns Result
- ✅ httpGet and httpPost convenience functions work
- ✅ httpStatus, httpBody, httpHeader extract response data
- ✅ httpIsSuccess correctly identifies success status
- ✅ httpParseJson parses JSON responses
- ✅ httpPostJson sends JSON with correct Content-Type
- ✅ Invalid URLs produce errors
- ✅ Network errors return Result::Err with descriptive message
- ✅ Timeouts handled correctly
- ✅ 25+ tests pass (100% pass rate)
- ✅ Interpreter/VM parity maintained
- ✅ cargo test -p atlas-runtime passes (full suite)
- ✅ cargo clippy clean (zero warnings)
- ✅ cargo fmt clean

## Implementation Notes
- Use reqwest blocking client for synchronous operations
- Store HttpRequest as HashMap for mutability pattern
- Store HttpResponse with actual response data (status, headers, body)
- All functions are pure stdlib functions (not intrinsics)
- Errors use RuntimeError::TypeError or custom HTTP error types
- Follow existing stdlib patterns (Rc<RefCell<>> for mutable state)
- Request/response are Atlas-native types (not exposing reqwest directly)

## Integration with Phase-10b
This phase provides:
- HttpRequest and HttpResponse types
- Basic GET/POST operations
- Request/response manipulation
- Error handling foundation

Phase-10b will add:
- PUT, DELETE, PATCH methods
- Permission integration (SecurityContext checks)
- Advanced timeout/retry logic
- File upload/download
- Form data submission
- Connection pooling
- ~25 additional tests

Together, phase-10a + phase-10b provide complete HTTP client with 50+ tests and 15+ functions.
