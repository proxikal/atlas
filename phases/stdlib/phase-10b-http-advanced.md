# Phase 10b: HTTP Advanced - Methods, Permissions & Features

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase-10a must be complete.

**Verification:**
```bash
grep -r "httpSend" crates/atlas-runtime/src/stdlib/
cargo test -p atlas-runtime http_core_tests
```

**What's needed:**
- Phase-10a complete (HttpRequest, HttpResponse, GET/POST)
- Security permission system (from foundation/phase-15)
- reqwest already added

**If missing:** Complete phase-10a first.

---

## Objective
Implement advanced HTTP features including all methods (PUT/DELETE/PATCH), permission integration, timeouts, retry logic, and common operations like file upload/download.

## Scope
**Previous phase (10a):** HttpRequest/HttpResponse types, GET/POST operations
**This phase (10b):** All HTTP methods, permissions, advanced features, common operations

## Files
**Update:** `crates/atlas-runtime/src/stdlib/http.rs` (+400 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register functions)
**Create:** `crates/atlas-runtime/tests/http_advanced_tests.rs` (~300 lines)

## Dependencies
- Phase-10a complete
- Security permission system (SecurityContext, NetworkPermission)
- All HTTP methods support in reqwest

## Implementation

### 1. Additional HTTP Methods
**Functions:**
- `httpRequestPut(url: string, body: string)` - Create PUT request
- `httpRequestDelete(url: string)` - Create DELETE request
- `httpRequestPatch(url: string, body: string)` - Create PATCH request
- `httpPut(url: string, body: string)` - Simple PUT request
- `httpDelete(url: string)` - Simple DELETE request
- `httpPatch(url: string, body: string)` - Simple PATCH request

**Behavior:**
- Same pattern as GET/POST from phase-10a
- Full CRUD operations support
- Return Result<HttpResponse>

### 2. Query Parameters
**Functions:**
- `httpSetQuery(request: HttpRequest, key: string, value: string)` - Add query param
- `httpSetQueries(request: HttpRequest, params: HashMap)` - Add multiple query params
- `httpGetWithQuery(url: string, params: HashMap)` - GET with query params

**Behavior:**
- Query params properly URL-encoded
- Multiple values for same key supported
- Immutable pattern (returns new request)

### 3. Permission Integration
**Functions:**
- `httpCheckPermission(url: string)` - Check if network access allowed
- Permission checks in httpSend before executing request

**Behavior:**
- Check SecurityContext for network permission
- Domain-based permission scoping (if available)
- Deny requests without permission
- Return Result::Err with permission error message
- Permission errors have special error code

**Permission Rules:**
- `allow_all()` - All network requests allowed
- `default()` - Network requests denied by default
- Domain whitelist (future enhancement)

### 4. Advanced Request Configuration
**Functions:**
- `httpSetFollowRedirects(request: HttpRequest, follow: bool)` - Enable/disable redirects
- `httpSetMaxRedirects(request: HttpRequest, max: number)` - Set max redirect count
- `httpSetUserAgent(request: HttpRequest, agent: string)` - Set User-Agent
- `httpSetAuth(request: HttpRequest, user: string, pass: string)` - Basic auth

**Behavior:**
- Default: follow redirects (max 10)
- Basic authentication via Authorization header
- Custom User-Agent support

### 5. Response Utilities
**Functions:**
- `httpStatusText(response: HttpResponse)` - Get status text ("OK", "Not Found")
- `httpContentType(response: HttpResponse)` - Get Content-Type header
- `httpContentLength(response: HttpResponse)` - Get Content-Length
- `httpIsRedirect(response: HttpResponse)` - Check if status 300-399
- `httpIsClientError(response: HttpResponse)` - Check if status 400-499
- `httpIsServerError(response: HttpResponse)` - Check if status 500-599

**Behavior:**
- Convenience functions for common status checks
- Return None/null if header doesn't exist
- Parse Content-Length as number

### 6. Common Operations
**Functions:**
- `httpDownload(url: string, filepath: string)` - Download file to disk
- `httpUpload(url: string, filepath: string)` - Upload file (POST with file body)
- `httpGetJson(url: string)` - GET and parse JSON in one call
- `httpPostForm(url: string, data: HashMap)` - POST form data

**Behavior:**
- httpDownload requires file write permission
- httpUpload requires file read permission
- httpGetJson returns Result<JsonValue>
- httpPostForm sets Content-Type: application/x-www-form-urlencoded

## Tests (25 tests minimum, 100% pass rate)

### HTTP Methods Tests (6):
1. PUT request updates resource
2. DELETE request removes resource
3. PATCH request partial update
4. Simple PUT convenience function
5. Simple DELETE convenience function
6. Simple PATCH convenience function

### Query Parameters Tests (4):
1. Add single query parameter
2. Add multiple query parameters
3. Query parameters URL-encoded
4. GET with query params convenience

### Permission Tests (6):
1. Network request allowed with allow_all()
2. Network request denied with default()
3. httpCheckPermission returns false without permission
4. httpSend returns error without permission
5. Permission error has descriptive message
6. Allowed domain permits request

### Advanced Configuration Tests (5):
1. Disable redirect following
2. Set max redirects
3. Custom User-Agent header
4. Basic authentication
5. Multiple configuration options

### Response Utilities Tests (4):
1. Get status text from status code
2. Get Content-Type header
3. Check if response is redirect
4. Check if response is client/server error

### Common Operations Tests (optional, if time permits):
1. Download file to disk
2. Upload file from disk
3. GET and parse JSON in one call
4. POST form data

**Note:** Tests require security context configuration.

## Acceptance Criteria
- ✅ httpRequestPut, httpRequestDelete, httpRequestPatch create requests
- ✅ httpPut, httpDelete, httpPatch convenience functions work
- ✅ httpSetQuery adds query parameters correctly
- ✅ Query parameters properly URL-encoded
- ✅ httpCheckPermission checks SecurityContext
- ✅ httpSend enforces network permission
- ✅ Permission errors have descriptive messages
- ✅ httpSetFollowRedirects, httpSetMaxRedirects work
- ✅ httpSetAuth adds Authorization header
- ✅ httpStatusText returns correct status text
- ✅ httpContentType extracts Content-Type
- ✅ httpIsRedirect, httpIsClientError, httpIsServerError work correctly
- ✅ 25+ tests pass (100% pass rate)
- ✅ Interpreter/VM parity maintained
- ✅ cargo test -p atlas-runtime passes (full suite)
- ✅ cargo clippy clean (zero warnings)
- ✅ cargo fmt clean

## Implementation Notes
- Extend http.rs from phase-10a (don't rewrite)
- Use SecurityContext from phase-15
- Permission checks in httpSend before reqwest call
- Follow immutable request pattern
- All new functions follow existing stdlib patterns
- Error messages descriptive and actionable

## Integration with Phase-10a
This phase builds on:
- HttpRequest and HttpResponse types from phase-10a
- httpSend execution from phase-10a
- Request/response manipulation patterns
- Error handling foundation

Together, phase-10a + phase-10b provide:
- Complete HTTP client (GET, POST, PUT, DELETE, PATCH)
- Full CRUD operation support
- Request building with headers, query params, body, timeout
- Response handling with status, headers, body parsing
- JSON integration (send and receive)
- Permission-based security
- Advanced configuration (redirects, auth, user agent)
- Common operations (download, upload, forms)
- 50+ comprehensive tests
- Production-ready HTTP capabilities
