# Advanced Stdlib Modules — Status

**Last Updated:** 2026-02-21
**Phase:** v02-completion-04

These modules are functional for common cases but have known gaps documented below.
They are explicitly excluded from the v02-completion-04 hardening scope.

---

## `http.rs`

**Status:** Common-case tested. Advanced features pending.

**Works:**
- `httpGet`, `httpPost`, `httpPut`, `httpDelete` — basic request/response
- JSON body serialization/deserialization
- Custom headers

**Known gaps:**
- No connection pooling or keep-alive
- No timeout configuration
- No redirect following control
- No streaming response bodies
- No multipart form data

**v0.3 hardening:** Dedicated HTTP phase with mock server for deterministic tests.

---

## `async_io.rs`

**Status:** Common-case tested. Advanced features pending.

**Works:**
- Basic async file read/write
- Channel send/receive

**Known gaps:**
- No backpressure on channels
- No select! across multiple futures
- No async directory traversal
- Timeout handling incomplete

**v0.3 hardening:** Requires tokio LocalSet context — needs async runtime phase first.

---

## `async_primitives.rs`

**Status:** Common-case tested. Advanced features pending.

**Works:**
- `asyncSleep`, `asyncDelay`
- Basic future chaining

**Known gaps:**
- No cancellation tokens
- No async mutex/semaphore primitives
- No structured concurrency (task groups)

**v0.3 hardening:** Blocked on async runtime architecture decisions.

---

## `compression/`

**Status:** Common-case tested (gzip, tar, zip all have integration tests in `tests/system.rs`).

**Works:**
- gzip compress/decompress
- tar create/extract
- zip create/extract/list

**Known gaps:**
- No streaming compression (full-buffer only)
- No password-protected zip
- No bzip2 or xz support
- Compression level validation is best-effort

**v0.3 hardening:** Low priority — current coverage sufficient for typical use cases.
