//! Integration tests for zip archive functionality

use atlas_runtime::span::Span;
use atlas_runtime::stdlib::compression::zip as atlas_zip;
use atlas_runtime::value::Value;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn create_test_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(path, content).unwrap();
}

fn create_test_dir(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::create_dir(&path).unwrap();
    path
}

fn str_value(s: &str) -> Value {
    Value::string(s.to_string())
}

fn str_array_value(paths: &[&str]) -> Value {
    let values: Vec<Value> = paths.iter().map(|p| str_value(p)).collect();
    Value::array(values)
}

fn num_value(n: f64) -> Value {
    Value::Number(n)
}

// ============================================================================
// Zip Creation Tests (9)
// ============================================================================

/// 1. Create an empty zip archive (no sources)
#[test]
fn test_zip_create_empty() {
    let temp = TempDir::new().unwrap();
    let zip_path = temp.path().join("empty.zip");

    let sources = str_array_value(&[]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());
    assert!(zip_path.metadata().unwrap().len() > 0);
}

/// 2. Create zip with a single file
#[test]
fn test_zip_create_single_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("hello.txt");
    fs::write(&test_file, "Hello, Atlas!").unwrap();

    let zip_path = temp.path().join("single.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());
}

/// 3. Create zip from a directory (recursively adds all contents)
#[test]
fn test_zip_create_directory_recursive() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "myproject");
    create_test_file(&test_dir, "main.atlas", "fn main() {}");
    create_test_file(&test_dir, "README.md", "# My Project");

    let sub = create_test_dir(&test_dir, "src");
    create_test_file(&sub, "lib.atlas", "// lib");

    let zip_path = temp.path().join("project.zip");
    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Verify contents include nested file
    let val = atlas_zip::zip_contains_file(&output, &str_value("myproject/src/lib.atlas"), span())
        .unwrap();
    assert_eq!(val, Value::Bool(true));
}

/// 4. Store compression (level 0 — no compression)
#[test]
fn test_zip_create_store_compression() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("data.txt");
    fs::write(&test_file, "some data").unwrap();

    let zip_path = temp.path().join("stored.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(0.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // With STORE, the entry's compressed size should equal the uncompressed size
    let list = atlas_zip::zip_list(&output, span()).unwrap();
    if let Value::Array(arr) = list {
        let guard = arr.lock().unwrap();
        if let Some(Value::HashMap(entry_map)) = guard.first() {
            let guard = entry_map.lock().unwrap();
            use atlas_runtime::stdlib::collections::hash::HashKey;
            use std::sync::Arc;
            let method_key = HashKey::String(Arc::new("method".to_string()));
            if let Some(Value::String(method)) = guard.get(&method_key) {
                assert_eq!(method.as_ref(), "stored");
            }
        }
    }
}

/// 5. Deflate compression at level 6 (default)
#[test]
fn test_zip_create_deflate_level_6() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("text.txt");
    // Write compressible data
    let content = "aaaaaaaaaa".repeat(1000);
    fs::write(&test_file, &content).unwrap();

    let zip_path = temp.path().join("deflate6.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(6.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Compressed size should be smaller than original
    let original_size = fs::metadata(&test_file).unwrap().len();
    let zip_size = fs::metadata(&zip_path).unwrap().len();
    assert!(zip_size < original_size);
}

/// 6. Deflate compression at level 9 (maximum)
#[test]
fn test_zip_create_deflate_level_9() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("text.txt");
    let content = "bbbbbbbbbb".repeat(1000);
    fs::write(&test_file, &content).unwrap();

    let zip_path = temp.path().join("deflate9.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(9.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());
}

/// 7. Verify file metadata is preserved (content round-trip)
#[test]
fn test_zip_create_preserves_file_content() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("data.json");
    let content = r#"{"key": "value", "num": 42}"#;
    fs::write(&test_file, content).unwrap();

    let zip_path = temp.path().join("meta.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    atlas_zip::zip_create(&sources, &output, None, span()).unwrap();

    // Extract and verify content is identical
    let extract_dir = temp.path().join("extracted");
    atlas_zip::zip_extract(&output, &str_value(extract_dir.to_str().unwrap()), span()).unwrap();
    let extracted_content = fs::read_to_string(extract_dir.join("data.json")).unwrap();
    assert_eq!(extracted_content, content);
}

/// 8. Create zip with an archive-level comment
#[test]
fn test_zip_create_with_comment() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("note.txt");
    fs::write(&test_file, "contents").unwrap();

    let zip_path = temp.path().join("commented.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    let comment = str_value("Atlas archive v1.0");

    let result =
        atlas_zip::zip_create_with_comment(&sources, &output, &comment, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Read back the comment
    let read_comment = atlas_zip::zip_comment_fn(&output, span()).unwrap();
    assert_eq!(
        read_comment,
        Value::string("Atlas archive v1.0".to_string())
    );
}

/// 9. Filter files during creation (create zip with subset of files)
#[test]
fn test_zip_create_filtered_sources() {
    let temp = TempDir::new().unwrap();
    let atlas_file = temp.path().join("main.atlas");
    let txt_file = temp.path().join("notes.txt");
    let rs_file = temp.path().join("helper.rs");
    fs::write(&atlas_file, "fn main() {}").unwrap();
    fs::write(&txt_file, "notes").unwrap();
    fs::write(&rs_file, "fn helper() {}").unwrap();

    // Only include .atlas and .txt files (simulate filtering)
    let zip_path = temp.path().join("filtered.zip");
    let sources = str_array_value(&[atlas_file.to_str().unwrap(), txt_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    atlas_zip::zip_create(&sources, &output, None, span()).unwrap();

    // Verify only filtered files are included
    let contains_atlas =
        atlas_zip::zip_contains_file(&output, &str_value("main.atlas"), span()).unwrap();
    let contains_txt =
        atlas_zip::zip_contains_file(&output, &str_value("notes.txt"), span()).unwrap();
    let contains_rs =
        atlas_zip::zip_contains_file(&output, &str_value("helper.rs"), span()).unwrap();
    assert_eq!(contains_atlas, Value::Bool(true));
    assert_eq!(contains_txt, Value::Bool(true));
    assert_eq!(contains_rs, Value::Bool(false));
}

// ============================================================================
// Zip Extraction Tests (7)
// ============================================================================

/// 10. Extract a zip archive
#[test]
fn test_zip_extract_archive() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("extract_me.txt");
    fs::write(&test_file, "extractable content").unwrap();

    let zip_path = temp.path().join("archive.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    atlas_zip::zip_create(&sources, &output, None, span()).unwrap();

    let extract_dir = temp.path().join("out");
    let result =
        atlas_zip::zip_extract(&output, &str_value(extract_dir.to_str().unwrap()), span()).unwrap();

    // Returns array of extracted file paths
    assert!(matches!(result, Value::Array(_)));
    assert!(extract_dir.join("extract_me.txt").exists());
}

/// 11. Extract to a specified directory
#[test]
fn test_zip_extract_to_directory() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("file.txt");
    fs::write(&test_file, "hello").unwrap();

    let zip_path = temp.path().join("arch.zip");
    atlas_zip::zip_create(
        &str_array_value(&[test_file.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let dest = temp.path().join("destination");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(dest.to_str().unwrap()),
        span(),
    )
    .unwrap();

    assert!(dest.exists());
    assert!(dest.join("file.txt").exists());
    let content = fs::read_to_string(dest.join("file.txt")).unwrap();
    assert_eq!(content, "hello");
}

/// 12. Preserve directory structure on extraction
#[test]
fn test_zip_extract_preserves_directory_structure() {
    let temp = TempDir::new().unwrap();
    let src_dir = create_test_dir(temp.path(), "project");
    let sub = create_test_dir(&src_dir, "lib");
    create_test_file(&src_dir, "main.atlas", "main");
    create_test_file(&sub, "utils.atlas", "utils");

    let zip_path = temp.path().join("project.zip");
    atlas_zip::zip_create(
        &str_array_value(&[src_dir.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("output");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    )
    .unwrap();

    assert!(out.join("project").join("main.atlas").exists());
    assert!(out.join("project").join("lib").join("utils.atlas").exists());
}

/// 13. Extract specific files by name
#[test]
fn test_zip_extract_specific_files() {
    let temp = TempDir::new().unwrap();
    let file_a = temp.path().join("a.txt");
    let file_b = temp.path().join("b.txt");
    fs::write(&file_a, "aaa").unwrap();
    fs::write(&file_b, "bbb").unwrap();

    let zip_path = temp.path().join("both.zip");
    atlas_zip::zip_create(
        &str_array_value(&[file_a.to_str().unwrap(), file_b.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("partial");
    let files_to_extract = str_array_value(&["a.txt"]);
    let result = atlas_zip::zip_extract_files(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        &files_to_extract,
        span(),
    )
    .unwrap();

    assert!(matches!(result, Value::Array(_)));
    assert!(out.join("a.txt").exists());
    assert!(!out.join("b.txt").exists());
}

/// 14. Handle nested directories on extraction
#[test]
fn test_zip_extract_nested_directories() {
    let temp = TempDir::new().unwrap();
    let root = create_test_dir(temp.path(), "root");
    let level1 = create_test_dir(&root, "level1");
    let level2 = create_test_dir(&level1, "level2");
    create_test_file(&level2, "deep.txt", "deep content");

    let zip_path = temp.path().join("nested.zip");
    atlas_zip::zip_create(
        &str_array_value(&[root.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("nested_out");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    )
    .unwrap();

    assert!(out
        .join("root")
        .join("level1")
        .join("level2")
        .join("deep.txt")
        .exists());
    let content = fs::read_to_string(
        out.join("root")
            .join("level1")
            .join("level2")
            .join("deep.txt"),
    )
    .unwrap();
    assert_eq!(content, "deep content");
}

/// 15. Path traversal prevention
#[test]
fn test_zip_extract_path_traversal_prevention() {
    use ::zip::write::FileOptions;
    use ::zip::ZipWriter as ExtZipWriter;
    use std::io::Write as IoWrite;

    let temp = TempDir::new().unwrap();
    let malicious_zip = temp.path().join("malicious.zip");

    // Manually craft a zip with a path traversal entry
    let file = fs::File::create(&malicious_zip).unwrap();
    let mut writer = ExtZipWriter::new(file);
    let opts = FileOptions::default();
    writer.start_file("../../../etc/passwd", opts).unwrap();
    writer.write_all(b"evil content").unwrap();
    writer.finish().unwrap();

    let out = temp.path().join("safe_out");
    let result = atlas_zip::zip_extract(
        &str_value(malicious_zip.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    );

    // Must error or extract safely
    if let Ok(_) = result {
        // If it didn't error, verify the file was not extracted outside the output dir
        let escaped = temp
            .path()
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("etc").join("passwd"));
        if let Some(escaped_path) = escaped {
            assert!(
                !escaped_path.exists(),
                "Path traversal succeeded - security bug!"
            );
        }
    } else {
        // Error is the expected behaviour
        assert!(result.is_err());
    }
}

/// 16. Corrupt zip returns an error
#[test]
fn test_zip_extract_corrupt_zip_error() {
    let temp = TempDir::new().unwrap();
    let corrupt = temp.path().join("corrupt.zip");
    fs::write(&corrupt, b"this is not a zip file at all").unwrap();

    let out = temp.path().join("out");
    let result = atlas_zip::zip_extract(
        &str_value(corrupt.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    );

    assert!(result.is_err());
}

// ============================================================================
// Zip Utilities Tests (5)
// ============================================================================

/// 17. List zip archive contents
#[test]
fn test_zip_list_contents() {
    let temp = TempDir::new().unwrap();
    let f1 = temp.path().join("alpha.txt");
    let f2 = temp.path().join("beta.txt");
    fs::write(&f1, "alpha").unwrap();
    fs::write(&f2, "beta").unwrap();

    let zip_path = temp.path().join("list_test.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f1.to_str().unwrap(), f2.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let list = atlas_zip::zip_list(&str_value(zip_path.to_str().unwrap()), span()).unwrap();

    if let Value::Array(arr) = &list {
        let guard = arr.lock().unwrap();
        assert_eq!(guard.len(), 2);
    } else {
        panic!("zipList should return an array");
    }
}

/// 18. Check file exists in zip (present)
#[test]
fn test_zip_contains_present() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("present.txt");
    fs::write(&f, "here").unwrap();

    let zip_path = temp.path().join("contains.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let result = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("present.txt"),
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Bool(true));
}

/// 19. Check file exists in zip (absent)
#[test]
fn test_zip_contains_absent() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("file.txt");
    fs::write(&f, "here").unwrap();

    let zip_path = temp.path().join("contains2.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let result = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("ghost.txt"),
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Bool(false));
}

/// 20. Get compression ratio
#[test]
fn test_zip_compression_ratio() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("ratio.txt");
    // Write highly compressible data
    fs::write(&f, "x".repeat(10000)).unwrap();

    let zip_path = temp.path().join("ratio.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    let ratio =
        atlas_zip::zip_compression_ratio(&str_value(zip_path.to_str().unwrap()), span()).unwrap();

    if let Value::Number(r) = ratio {
        assert!(r >= 0.0, "ratio should be non-negative");
        assert!(r <= 1.0, "deflate ratio should be at most 1.0");
        assert!(r < 0.5, "10000 'x' chars should compress well");
    } else {
        panic!("zipCompressionRatio should return a number");
    }
}

/// 21. Add file to existing zip
#[test]
fn test_zip_add_file_to_existing() {
    let temp = TempDir::new().unwrap();
    let original = temp.path().join("original.txt");
    let addition = temp.path().join("added.txt");
    fs::write(&original, "original").unwrap();
    fs::write(&addition, "added file").unwrap();

    let zip_path = temp.path().join("grow.zip");
    atlas_zip::zip_create(
        &str_array_value(&[original.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    // Add new file
    let result = atlas_zip::zip_add_file_fn(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(addition.to_str().unwrap()),
        None,
        None,
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Null);

    // Both files should now be in the archive
    let has_original = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("original.txt"),
        span(),
    )
    .unwrap();
    let has_added = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("added.txt"),
        span(),
    )
    .unwrap();

    assert_eq!(has_original, Value::Bool(true));
    assert_eq!(has_added, Value::Bool(true));
}

// ============================================================================
// Zip Validation Tests (2)
// ============================================================================

/// 22. Validate a valid zip file
#[test]
fn test_zip_validate_valid() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("val.txt");
    fs::write(&f, "validate me").unwrap();

    let zip_path = temp.path().join("valid.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let result =
        atlas_zip::zip_validate_fn(&str_value(zip_path.to_str().unwrap()), span()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

/// 23. Validate an invalid file
#[test]
fn test_zip_validate_invalid() {
    let temp = TempDir::new().unwrap();
    let not_a_zip = temp.path().join("fake.zip");
    fs::write(&not_a_zip, b"PKnot a real zip file").unwrap();

    let result =
        atlas_zip::zip_validate_fn(&str_value(not_a_zip.to_str().unwrap()), span()).unwrap();
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// Compression Method Tests (3)
// ============================================================================

/// 24. Store vs deflate: stored archive is larger
#[test]
fn test_zip_store_vs_deflate_size_comparison() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("compare.txt");
    // Highly compressible content
    fs::write(&f, "z".repeat(50000)).unwrap();

    let stored_zip = temp.path().join("stored.zip");
    let deflated_zip = temp.path().join("deflated.zip");

    let sources = str_array_value(&[f.to_str().unwrap()]);

    atlas_zip::zip_create(
        &sources,
        &str_value(stored_zip.to_str().unwrap()),
        Some(&num_value(0.0)),
        span(),
    )
    .unwrap();
    atlas_zip::zip_create(
        &sources,
        &str_value(deflated_zip.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    let stored_size = fs::metadata(&stored_zip).unwrap().len();
    let deflated_size = fs::metadata(&deflated_zip).unwrap().len();

    // Deflated archive must be smaller than stored for highly compressible data
    assert!(
        deflated_size < stored_size,
        "deflated ({}) should be smaller than stored ({})",
        deflated_size,
        stored_size
    );
}

/// 25. Multiple compression levels all produce valid archives
#[test]
fn test_zip_compression_levels_all_valid() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("levels.txt");
    fs::write(&f, "test data for compression level testing").unwrap();

    for level in [0.0_f64, 1.0, 3.0, 5.0, 6.0, 9.0] {
        let zip_path = temp.path().join(format!("level_{}.zip", level as u32));
        atlas_zip::zip_create(
            &str_array_value(&[f.to_str().unwrap()]),
            &str_value(zip_path.to_str().unwrap()),
            Some(&num_value(level)),
            span(),
        )
        .unwrap();

        let valid =
            atlas_zip::zip_validate_fn(&str_value(zip_path.to_str().unwrap()), span()).unwrap();
        assert_eq!(
            valid,
            Value::Bool(true),
            "Level {} should produce a valid zip",
            level
        );
    }
}

/// 26. Large file compression (> 512 KB)
#[test]
fn test_zip_large_file_compression() {
    let temp = TempDir::new().unwrap();
    let large_file = temp.path().join("large.bin");
    // 1 MB of repeating bytes (highly compressible)
    let data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 64) as u8).collect();
    fs::write(&large_file, &data).unwrap();

    let zip_path = temp.path().join("large.zip");
    let result = atlas_zip::zip_create(
        &str_array_value(&[large_file.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Zip should be smaller than original
    let zip_size = fs::metadata(&zip_path).unwrap().len();
    assert!(zip_size < data.len() as u64);
}

// ============================================================================
// Integration Tests (3)
// ============================================================================

/// 27. Full round-trip: create → extract → verify content integrity
#[test]
fn test_zip_round_trip_integrity() {
    let temp = TempDir::new().unwrap();

    // Create source files with known content
    let src = create_test_dir(temp.path(), "source");
    create_test_file(&src, "config.toml", "[package]\nname = \"atlas\"\n");
    create_test_file(&src, "README.md", "# Atlas\nFast compiler.\n");

    let sub = create_test_dir(&src, "tests");
    create_test_file(
        &sub,
        "test_suite.atlas",
        "fn test_basic() { assert(1 == 1); }",
    );

    // Create zip
    let zip_path = temp.path().join("roundtrip.zip");
    atlas_zip::zip_create(
        &str_array_value(&[src.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    // Extract zip
    let out = temp.path().join("restored");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    )
    .unwrap();

    // Verify content integrity
    let config = fs::read_to_string(out.join("source").join("config.toml")).unwrap();
    let readme = fs::read_to_string(out.join("source").join("README.md")).unwrap();
    let test_suite =
        fs::read_to_string(out.join("source").join("tests").join("test_suite.atlas")).unwrap();

    assert_eq!(config, "[package]\nname = \"atlas\"\n");
    assert_eq!(readme, "# Atlas\nFast compiler.\n");
    assert_eq!(test_suite, "fn test_basic() { assert(1 == 1); }");
}

/// 28. Zip list returns correct metadata fields
#[test]
fn test_zip_list_metadata_fields() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("meta_test.txt");
    fs::write(&f, "metadata check").unwrap();

    let zip_path = temp.path().join("metadata.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let list = atlas_zip::zip_list(&str_value(zip_path.to_str().unwrap()), span()).unwrap();

    if let Value::Array(arr) = &list {
        let guard = arr.lock().unwrap();
        let first = &guard[0];

        if let Value::HashMap(map) = first {
            use atlas_runtime::stdlib::collections::hash::HashKey;
            use std::sync::Arc;

            let map_guard = map.lock().unwrap();

            // Must have all required fields
            let name_key = HashKey::String(Arc::new("name".to_string()));
            let size_key = HashKey::String(Arc::new("size".to_string()));
            let csize_key = HashKey::String(Arc::new("compressedSize".to_string()));
            let isdir_key = HashKey::String(Arc::new("isDir".to_string()));
            let method_key = HashKey::String(Arc::new("method".to_string()));

            assert!(map_guard.get(&name_key).is_some(), "missing 'name' field");
            assert!(map_guard.get(&size_key).is_some(), "missing 'size' field");
            assert!(
                map_guard.get(&csize_key).is_some(),
                "missing 'compressedSize' field"
            );
            assert!(map_guard.get(&isdir_key).is_some(), "missing 'isDir' field");
            assert!(
                map_guard.get(&method_key).is_some(),
                "missing 'method' field"
            );

            // File should not be a directory
            assert_eq!(map_guard.get(&isdir_key), Some(&Value::Bool(false)));
        } else {
            panic!("list entry should be a HashMap");
        }
    } else {
        panic!("zipList should return an Array");
    }
}

/// 29. Comment round-trip: write and read back correctly
#[test]
fn test_zip_comment_round_trip() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("note.txt");
    fs::write(&f, "noted").unwrap();

    let zip_path = temp.path().join("with_comment.zip");
    let long_comment = "This archive was created by Atlas stdlib phase-14c. Version: 0.2.0-dev";

    atlas_zip::zip_create_with_comment(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        &str_value(long_comment),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    let read_back =
        atlas_zip::zip_comment_fn(&str_value(zip_path.to_str().unwrap()), span()).unwrap();
    assert_eq!(read_back, Value::string(long_comment.to_string()));
}

// ============================================================================
// Error Handling Tests (4)
// ============================================================================

/// 30. Missing source file returns error
#[test]
fn test_zip_create_missing_source_error() {
    let temp = TempDir::new().unwrap();
    let zip_path = temp.path().join("will_fail.zip");

    let result = atlas_zip::zip_create(
        &str_array_value(&["/does/not/exist/file.txt"]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    );

    assert!(result.is_err());
}

/// 31. Invalid compression level returns error
#[test]
fn test_zip_create_invalid_level_error() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("ok.txt");
    fs::write(&f, "ok").unwrap();
    let zip_path = temp.path().join("bad_level.zip");

    let result = atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(10.0)), // out of range
        span(),
    );

    assert!(result.is_err());
}

/// 32. Extract specific file that doesn't exist returns error
#[test]
fn test_zip_extract_missing_entry_error() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("a.txt");
    fs::write(&f, "a").unwrap();

    let zip_path = temp.path().join("one_file.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("out");
    let result = atlas_zip::zip_extract_files(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        &str_array_value(&["nonexistent.txt"]),
        span(),
    );

    assert!(result.is_err());
}

/// 33. Type error on non-string sources returns error
#[test]
fn test_zip_create_type_error_sources() {
    let temp = TempDir::new().unwrap();
    let zip_path = temp.path().join("type_err.zip");

    // Pass a number inside the sources array
    let bad_sources = Value::array(vec![Value::Number(42.0)]);
    let result = atlas_zip::zip_create(
        &bad_sources,
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    );

    assert!(result.is_err());
}
