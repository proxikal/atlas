# Phase 05: Complete File I/O API

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Security model for file I/O must be defined.

**Verification:**
```bash
ls docs/*io* docs/*security*
cat docs/io-security-model.md
```

**What's needed:**
- docs/io-security-model.md exists from v0.1
- Security policy defined

**If missing:** Review security model before implementing

---

## Objective
Implement complete file I/O API with 10 functions covering reading, writing, directory operations, and metadata access with proper error handling and security.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/io.rs` (~900 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (add io module)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (register functions)
**Update:** `crates/atlas-runtime/src/error.rs` (add IoError variant)
**Update:** `Cargo.toml` (add tempfile dev-dep)
**Tests:** `crates/atlas-runtime/tests/stdlib_io_tests.rs` (~600 lines)
**VM Tests:** `crates/atlas-runtime/tests/vm_stdlib_io_tests.rs` (~600 lines)

## Dependencies
- v0.1 security model from docs/io-security-model.md
- Rust std::fs for file operations
- tempfile crate for test isolation

## Implementation

### File Reading/Writing (4 functions)
Implement readFile, writeFile, appendFile, fileExists. Reading loads entire file as UTF-8 validating encoding. Writing creates or overwrites with UTF-8. Appending adds to end creating if needed. Exists checks without reading. All check security policy.

### Directory Operations (4 functions)
Implement readDir, createDir, removeFile, removeDir. ReadDir lists entries as array of names. CreateDir uses mkdir -p behavior. RemoveFile deletes files only. RemoveDir requires empty directory for safety.

### File Metadata (2 functions)
Implement fileInfo and pathJoin. FileInfo returns object with size, modified timestamp, type flags. PathJoin combines path components platform-aware using OS separator.

### Architecture Notes
Check all paths against security policy before operations. Validate UTF-8 on read returning clear errors for binary files. Use Rust std::fs delegating to OS. Platform-aware path handling for Windows/Unix differences. Add IoError variant to RuntimeError if missing. Tests use temporary directories for isolation.

## Tests (TDD - Use rstest)

**I/O tests cover:**
1. Read/write basic operations
2. UTF-8 validation
3. Directory operations
4. File metadata
5. Path joining cross-platform
6. Error conditions - not found, permission, invalid UTF-8
7. Security policy checks
8. VM parity
9. Temp file usage preventing pollution

**Minimum test count:** 100 tests (50 interpreter, 50 VM)

## Integration Points
- Uses: std::fs for operations
- Uses: std::path::PathBuf for paths
- Updates: error.rs with IoError variant
- Uses: Value::Object for fileInfo return
- Uses: Security model from docs
- Updates: Cargo.toml with tempfile
- Updates: prelude.rs with 10 functions
- Updates: docs/stdlib.md
- Output: Complete I/O API

## Acceptance
- All 10 functions implemented
- UTF-8 validation in readFile
- Path security checks logged
- Platform-aware path handling
- Error handling comprehensive
- Temp directories in tests
- 100+ tests pass
- Interpreter/VM parity verified
- io.rs under 1000 lines
- Test files under 700 lines each
- Documentation updated
- No clippy warnings
- cargo test passes
