# Phase 14: Compression and Archives

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** File I/O and path manipulation must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/stdlib/io.rs
ls crates/atlas-runtime/src/stdlib/path.rs
cargo test stdlib_io
```

**What's needed:**
- File I/O from stdlib/phase-05
- Path manipulation from stdlib/phase-13
- Compression libraries (flate2, tar, zip)
- Result types for errors

**If missing:** Complete stdlib phases 05 and 13 first

---

## Objective
Implement compression and archive utilities supporting gzip, tar, and zip formats - enabling package distribution, data compression, and archive management essential for build systems and package managers.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/compression/mod.rs` (~200 lines)
**Create:** `crates/atlas-runtime/src/stdlib/compression/gzip.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/stdlib/compression/tar.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/stdlib/compression/zip.rs` (~500 lines)
**Update:** `Cargo.toml` (add flate2, tar, zip dependencies)
**Update:** `docs/stdlib.md` (~300 lines compression docs)
**Tests:** `crates/atlas-runtime/tests/compression_tests.rs` (~700 lines)

## Dependencies
- flate2 for gzip compression
- tar crate for tar archives
- zip crate for zip archives
- File I/O stdlib
- Path manipulation stdlib
- Result types

## Implementation

### Gzip Compression
Compress data with gzip_compress function. Takes bytes or string returns compressed bytes. Compression level configuration. Decompress with gzip_decompress function. Validate gzip format. Stream compression for large files. Memory-efficient processing. Error handling for corrupt data.

### Tar Archive Creation
Create tar archive with tar_create function. Add files and directories to archive. Preserve file metadata permissions, timestamps. Recursive directory archiving. Compression option for tar.gz. Stream tar creation. Filter files during archiving. Archive metadata validation.

### Tar Archive Extraction
Extract tar archive with tar_extract function. Extract to specified directory. Preserve file metadata on extraction. Selective file extraction. List archive contents without extracting. Validate archive integrity. Handle corrupt archives gracefully. Decompress tar.gz automatically.

### Zip Archive Creation
Create zip archive with zip_create function. Add files and directories. Store or deflate compression. Compression level setting. Recursive directory zipping. Password protection (optional). Archive comments. Central directory structure.

### Zip Archive Extraction
Extract zip archive with zip_extract function. Extract all or specific files. Preserve directory structure. Handle password-protected zips. List zip contents. Validate zip format. Extract to memory option. Streaming extraction for large zips.

### Compression Utilities
Detect compression format from file or bytes. Auto-decompress based on format. Compress and decompress streaming. Memory usage optimization. Compression ratio reporting. Benchmark compression speed. Multi-file compression batching.

### Archive Utilities
List archive contents for any format. Extract single file from archive. Add file to existing archive. Remove file from archive. Update file in archive. Archive validation and repair. Convert between archive formats.

### Error Handling
Handle corrupt archive errors. Checksum validation failures. Insufficient permissions errors. Disk space errors during extraction. Path traversal attack prevention. Invalid compression format errors. Clear error messages with recovery suggestions.

## Tests (TDD - Use rstest)

**Gzip tests:**
1. Compress string with gzip
2. Decompress gzip data
3. Round-trip compression
4. Compression levels
5. Stream compression
6. Corrupt data handling
7. Large file compression

**Tar creation tests:**
1. Create tar archive
2. Add files to tar
3. Add directory recursively
4. Preserve file metadata
5. Create tar.gz compressed
6. Filter files during creation
7. Validate tar format

**Tar extraction tests:**
1. Extract tar archive
2. Extract to directory
3. Preserve metadata on extract
4. Extract specific files
5. List tar contents
6. Extract tar.gz
7. Handle corrupt tar

**Zip creation tests:**
1. Create zip archive
2. Add files to zip
3. Deflate compression
4. Recursive directory zip
5. Set compression level
6. Archive comments
7. Password protection (optional)

**Zip extraction tests:**
1. Extract zip archive
2. Extract specific files
3. Preserve directory structure
4. List zip contents
5. Handle password-protected
6. Validate zip format
7. Stream extraction

**Compression utilities tests:**
1. Auto-detect format
2. Auto-decompress
3. Streaming compression
4. Compression ratio
5. Multi-file batch

**Archive utilities tests:**
1. List any archive format
2. Extract single file
3. Add to existing archive
4. Update file in archive
5. Validate archive
6. Convert tar to zip

**Error handling tests:**
1. Corrupt archive error
2. Checksum failure
3. Permission errors
4. Disk space error
5. Path traversal prevention
6. Invalid format error

**Integration tests:**
1. Package distribution workflow
2. Backup and restore
3. Build artifact archiving
4. Multi-format handling
5. Real-world archive operations

**Minimum test count:** 70 tests

## Integration Points
- Uses: File I/O from phase-05
- Uses: Path manipulation from phase-13
- Uses: Result types from foundation/phase-09
- Creates: Compression utilities
- Creates: Archive management
- Output: Package distribution capabilities

## Acceptance
- Gzip compression works
- Tar archives create and extract
- Zip archives create and extract
- Format auto-detection
- Metadata preservation
- Stream processing for large files
- Error handling comprehensive
- 70+ tests pass
- Documentation with examples
- No clippy warnings
- cargo test passes
