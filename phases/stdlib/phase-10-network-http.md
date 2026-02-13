# Phase 10: Network and HTTP Client

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Security permissions and Result types must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/security/permissions.rs
ls crates/atlas-runtime/src/result_type.rs
cargo test security
```

**What's needed:**
- Security permissions from foundation/phase-15
- Result types from foundation/phase-09
- HTTP client library (reqwest)
- Async runtime (tokio) if async

**If missing:** Complete foundation phases 09 and 15 first

---

## Objective
Implement HTTP client and basic networking for making web requests and API calls - enabling Atlas programs to interact with web services and external APIs essential for modern applications.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/http.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/stdlib/net.rs` (~400 lines)
**Update:** `Cargo.toml` (add reqwest dependency)
**Update:** `docs/stdlib.md` (~400 lines network docs)
**Create:** `docs/http-guide.md` (~500 lines)
**Tests:** `crates/atlas-runtime/tests/http_tests.rs` (~600 lines)

## Dependencies
- reqwest for HTTP client
- Security permission system
- Result types for errors
- JSON support from stdlib/phase-04
- async runtime if async (or blocking)

## Implementation

### HTTP Request Building
Create HTTP request with method and URL. GET, POST, PUT, DELETE, PATCH methods. Request headers configuration. Query parameters. Request body JSON, form data, raw. Timeout configuration. Follow redirects option. Authentication headers. User agent setting.

### HTTP Response Handling
Response object with status code. Response headers access. Response body as string or bytes. JSON parsing from response. Status code checking. Success and error responses. Response metadata. Content type detection.

### Synchronous HTTP Client
Blocking HTTP operations for simplicity. fetch function for GET requests. post, put, delete functions. Configure client with timeout, headers. Connection pooling for performance. Retry logic for failures. Error handling network errors, timeouts, invalid responses.

### Permission Integration
Check network permission before requests. Domain-based permission scoping. Deny requests without permission. Security policy for allowed domains. Audit log network requests. Permission errors clear. Sandboxed code cannot access network.

### Common HTTP Operations
Helper functions for common patterns. GET JSON from API. POST JSON to API. Download file from URL. Upload file to server. Form submission. API client building blocks.

## Tests (TDD - Use rstest)
1. GET request to URL
2. POST request with JSON
3. PUT request
4. DELETE request
5. Request headers set
6. Query parameters
7. Response status code
8. Response body as string
9. Parse JSON response
10. Network permission required
11. Permission denied blocks request
12. Timeout handling
13. Error response handling
14. Follow redirects
15. Connection reuse

**Minimum test count:** 50 tests

## Acceptance
- HTTP requests work for all methods
- Response handling functional
- Permission system enforced
- JSON request/response works
- Timeouts handled
- 50+ tests pass
- HTTP guide documentation
- cargo test passes
