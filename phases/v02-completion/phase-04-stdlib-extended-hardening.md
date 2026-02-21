# Phase v02-completion-04: Stdlib Extended Hardening (fs, path, datetime, regex, collections)

## Dependencies

**Required:** v02-completion-03 complete (core stdlib hardened)

**Verification:**
```bash
cargo nextest run -p atlas-runtime --test stdlib 2>&1 | tail -3  # must pass
ls crates/atlas-runtime/src/stdlib/fs.rs
ls crates/atlas-runtime/src/stdlib/datetime.rs
ls crates/atlas-runtime/src/stdlib/regex.rs
ls crates/atlas-runtime/src/stdlib/collections/
```

---

## Objective

Continuing the stdlib hardening from phase-03, this phase covers the remaining testable-without-network modules: `fs`, `path`, `datetime`, `regex`, and `collections` (HashMap, HashSet, Queue, Stack). Explicitly excludes `http`, `async_io`, `async_primitives`, and `compression` — these require network or complex external state and are formally documented as "common-case tested, advanced features pending" rather than fixed in this phase.

---

## Files

**Update (fixes only — no new functions):**
- `crates/atlas-runtime/src/stdlib/fs.rs` (~548 lines)
- `crates/atlas-runtime/src/stdlib/path.rs` (~532 lines)
- `crates/atlas-runtime/src/stdlib/datetime.rs` (~1292 lines)
- `crates/atlas-runtime/src/stdlib/regex.rs` (~622 lines)
- `crates/atlas-runtime/src/stdlib/collections/` (all files)

**Create (tests):**
- `crates/atlas-runtime/tests/stdlib/fs_hardening.rs` — ~150 lines, 20+ tests
- `crates/atlas-runtime/tests/stdlib/path_hardening.rs` — ~120 lines, 20+ tests
- `crates/atlas-runtime/tests/stdlib/datetime_hardening.rs` — ~150 lines, 20+ tests
- `crates/atlas-runtime/tests/stdlib/regex_hardening.rs` — ~120 lines, 15+ tests
- `crates/atlas-runtime/tests/stdlib/collections_hardening.rs` — ~150 lines, 25+ tests

**Create (documentation):**
- `crates/atlas-runtime/src/stdlib/ADVANCED_MODULES_STATUS.md` — 30 lines documenting http/async/compression status

**Total new code:** ~700 lines tests + fixes as needed + 30 lines doc
**Minimum test count:** 60 tests

---

## Implementation Notes

**Audit process (same as phase-03 — read each module before writing tests):**

**Filesystem (`fs.rs`) — testable with temp dirs:**
Edge cases: `read_file` (nonexistent file → error, empty file → empty string, binary file), `write_file` (write to read-only location, overwrite existing), `append_file` (file doesn't exist yet → creates it), `delete_file` (nonexistent → error or no-op?), `list_dir` (empty dir, nonexistent dir → error), `exists` (file, dir, nonexistent all return correct bool), `create_dir`/`create_dir_all` (already exists), `move_file` (destination exists, source doesn't exist), `copy_file` (same as move).
Use `std::env::temp_dir()` in tests — always clean up after test.

**Path (`path.rs`) — pure computation, no disk I/O:**
Edge cases: `join` (absolute second arg replaces first — matches Rust/OS semantics), `basename` (trailing slash, file without extension), `dirname` (root path, file in root), `extension` (no extension → empty/null, multiple dots), `is_absolute` (relative and absolute paths), `normalize` (double slashes, `.` and `..` in middle), `resolve` (relative from different bases), path separator differences (document posix-only or handle both).

**Datetime (`datetime.rs`) — large module, complex:**
Edge cases: `parse` (invalid format, empty string, edge dates like Feb 29 in non-leap year, midnight), `format` (invalid format strings, all format tokens), `now` (not deterministic — test that it returns a valid timestamp, not a specific value), `add_days`/`add_hours`/`add_minutes` (missing from v0.2 — if not implemented, document as limitation not bug), date comparison (`before`/`after`/`equals`), timezone handling (if supported — document limitations).

**Regex (`regex.rs`) — wraps Rust `regex` crate:**
Edge cases: invalid regex pattern → error at construction not at use, `match` on empty string, `find_all` returning empty list (no matches), `replace` vs `replace_all` semantics, named capture groups (if supported), special chars in replacement string, `split` on pattern that matches at start/end, very long string performance (should not panic).

**Collections — HashMap, HashSet, Queue, Stack:**
Read `collections/` files first — understand what's implemented.
HashMap: `get` on missing key (null vs error), `set` overwrite existing, `keys`/`values` on empty, `has`, `delete` on nonexistent key, `size`, iteration order (documented as unordered — test that it doesn't crash, not order).
HashSet: `add` duplicate (idempotent), `has`, `remove` nonexistent, `union`/`intersection`/`difference` on empty sets, `to_array` from empty set.
Queue: `enqueue`/`dequeue` on empty (error), `peek` on empty, `size`, `is_empty`.
Stack: `push`/`pop` on empty (error), `peek` on empty, `size`, `is_empty`.

**Advanced modules — document, don't fix:**
For `http.rs`, `async_io.rs`, `async_primitives.rs`, and `compression/`:
Create `ADVANCED_MODULES_STATUS.md` in stdlib src dir documenting:
- Which functions are "works for common case"
- Known gaps (e.g., no connection pooling in http, no backpressure in async_io)
- What "v0.3 hardening" would look like for each
Do NOT attempt to fix these in this phase — network/async edge cases require dedicated infrastructure.

**Critical requirements:**
- Filesystem tests MUST clean up temp files (use `scopeguard` or manual cleanup in test)
- Datetime tests must not assert specific "now" values — test format/parse round-trips instead
- Regex tests must verify error handling for invalid patterns
- Collections tests must verify empty-container error behavior

**Error handling:**
- File not found → RuntimeError with IoError code
- Invalid regex → RuntimeError at construction time (not at use)
- Pop/dequeue from empty → RuntimeError (not panic)
- All errors must have proper AT-codes — add codes if missing

---

## Tests (TDD Approach)

**Filesystem hardening:** (20 tests)
- read/write/delete round-trip, error cases for missing files, directory operations

**Path hardening:** (20 tests)
- All path manipulation functions, including edge cases with separators and relative paths

**Datetime hardening:** (20 tests)
- Parse/format round-trips, comparison operations, error cases for invalid inputs

**Regex hardening:** (15 tests)
- Pattern validity, match/find/replace operations, edge cases for empty and special inputs

**Collections hardening:** (25 tests)
- Per-collection: empty container edge cases, overwrite/duplicate semantics, size tracking

**Minimum test count:** 60 tests

**Parity requirement:** All tests run in both interpreter and VM with identical results.

**Test approach:**
- Filesystem: use `tempfile` crate if already in dev-dependencies, else `std::env::temp_dir()`
- Datetime: round-trip tests (parse → format → parse) are deterministic
- Use `#[rstest]` for input/output parameterized cases

---

## Acceptance Criteria

- ✅ fs, path, datetime, regex, collections modules audited and edge cases fixed
- ✅ 60+ new tests across 5 modules
- ✅ All tests pass in both interpreter and VM
- ✅ Filesystem tests clean up after themselves (no temp file leaks)
- ✅ `ADVANCED_MODULES_STATUS.md` created documenting http/async/compression status
- ✅ All RuntimeErrors from stdlib have proper AT-codes (no uncoded errors)
- ✅ No clippy warnings
- ✅ `cargo nextest run -p atlas-runtime` passes

---

## References

**Specifications:** `docs/specification/stdlib.md`
**Related phases:** v02-completion-03 (prerequisite — same methodology), v02-completion-05 (JIT)
