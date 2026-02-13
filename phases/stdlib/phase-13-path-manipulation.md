# Phase 13: Path Manipulation and File System

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** String stdlib and file I/O must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/stdlib/string.rs
ls crates/atlas-runtime/src/stdlib/io.rs
cargo test stdlib_string
cargo test stdlib_io
```

**What's needed:**
- String stdlib from stdlib/phase-01
- File I/O from stdlib/phase-05
- Cross-platform path handling (std::path)
- File metadata access

**If missing:** Complete stdlib phases 01 and 05 first

---

## Objective
Implement comprehensive path manipulation and file system utilities enabling cross-platform path operations, directory traversal, file metadata access, and temporary files - providing Node.js path module equivalent for Atlas.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/path.rs` (~700 lines)
**Create:** `crates/atlas-runtime/src/stdlib/fs.rs` (~600 lines)
**Update:** `docs/stdlib.md` (~300 lines path and fs docs)
**Tests:** `crates/atlas-runtime/tests/path_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/fs_tests.rs` (~500 lines)

## Dependencies
- String manipulation
- File I/O operations
- std::path for cross-platform paths
- std::fs for file system ops
- Security permissions

## Implementation

### Path Construction and Parsing
Join path components with join function. Parse path into components with parse. Normalize path removing dots and redundant separators. Absolute path conversion with absolute. Relative path computation with relative. Parent directory with parent. File name extraction with basename. Directory name with dirname. File extension with extension. Platform-specific separator handling.

### Path Comparison and Validation
Compare paths for equality. Check if path is absolute or relative. Check if path is directory or file. Validate path syntax. Check path exists with exists function. Canonical path resolution resolving symlinks. Case sensitivity handling per platform. Path prefix and suffix checking.

### Directory Operations
Create directory with mkdir. Create directory recursively with mkdirp. Remove directory with rmdir. Remove directory recursively with rmdirRecursive. List directory contents with readdir. Walk directory tree with walk. Filter directory listing. Sort directory entries. Glob pattern matching for paths.

### File Metadata
Get file metadata size, permissions, timestamps. File size with size function. Modified time with mtime. Created time with ctime. Access time with atime. File permissions query. Check if file is directory, file, or symlink. File type detection. Inode information (Unix).

### Temporary Files and Directories
Create temporary file with tmpfile. Create temporary directory with tmpdir. Automatic cleanup on exit. Named temporary files. Secure temporary file creation. Temporary path generation. Platform-specific temp directory.

### Symlink Operations
Create symbolic link with symlink. Read symlink target with readlink. Check if path is symlink. Resolve symlink chains. Relative and absolute symlinks. Platform support check. Permission handling for symlinks.

### Path Utilities
Home directory with homedir. Current working directory with cwd. System temp directory with tempdir. Path separator constant. Directory separator constant. Extension separator. Drive letter extraction (Windows). UNC path support (Windows). Path escaping for shell commands.

### Cross-Platform Compatibility
Handle Windows vs Unix path differences. Convert between path formats. Drive letters on Windows. UNC paths on Windows. Case insensitivity on Windows. Symlink support differences. Permission model differences. Path length limits per platform.

## Tests (TDD - Use rstest)

**Path construction tests:**
1. Join path components
2. Parse path
3. Normalize path
4. Absolute path conversion
5. Relative path computation
6. Parent directory
7. Basename extraction
8. Dirname extraction
9. Extension extraction
10. Platform separator handling

**Path validation tests:**
1. Check path is absolute
2. Check path is relative
3. Path exists check
4. Canonical path resolution
5. Path comparison
6. Case sensitivity handling

**Directory operations tests:**
1. Create directory
2. Create directory recursively
3. Remove directory
4. Remove directory recursively
5. List directory contents
6. Walk directory tree
7. Glob pattern matching
8. Filter and sort entries

**File metadata tests:**
1. Get file size
2. Get modified time
3. Get created time
4. Get access time
5. Query file permissions
6. Check is directory
7. Check is file
8. Check is symlink

**Temporary files tests:**
1. Create temporary file
2. Create temporary directory
3. Automatic cleanup
4. Named temporary file
5. Secure temp creation
6. Temp directory location

**Symlink tests:**
1. Create symbolic link
2. Read symlink target
3. Check is symlink
4. Resolve symlink chain
5. Relative symlink
6. Absolute symlink

**Path utilities tests:**
1. Home directory
2. Current working directory
3. System temp directory
4. Path separator constant
5. Extension separator
6. Drive letter extraction (Windows)

**Cross-platform tests:**
1. Windows path handling
2. Unix path handling
3. Path format conversion
4. UNC path support
5. Case insensitivity Windows
6. Symlink platform differences
7. Path length limits

**Integration tests:**
1. File system traversal
2. Recursive file operations
3. Path-based filtering
4. Cross-platform scripts
5. Real-world path scenarios

**Minimum test count:** 80 tests

## Integration Points
- Uses: String stdlib from phase-01
- Uses: File I/O from phase-05
- Uses: Security permissions from foundation/phase-15
- Creates: Path manipulation API
- Creates: File system utilities
- Output: Cross-platform file operations

## Acceptance
- Path construction and parsing work
- Directory operations functional
- File metadata accessible
- Temporary files work securely
- Symlink operations supported
- Cross-platform compatibility verified
- 80+ tests pass on all platforms
- Path utilities comprehensive
- Documentation with cross-platform notes
- No clippy warnings
- cargo test passes
