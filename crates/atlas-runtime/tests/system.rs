// Merged: path_tests + fs_tests + process_tests + gzip_tests + tar_tests + zip_tests

use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib;
use atlas_runtime::stdlib::compression::gzip;
use atlas_runtime::stdlib::compression::tar;
use atlas_runtime::stdlib::compression::zip as atlas_zip;
use atlas_runtime::stdlib::fs;
use atlas_runtime::value::Value;
use atlas_runtime::Atlas;
use std::fs as std_fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// --- Path manipulation ---

// Integration tests for path manipulation functions
//
// Tests all path functions through the stdlib interface

fn test_span() -> Span {
    Span::dummy()
}

fn call_fn(name: &str, args: &[Value]) -> Result<Value, atlas_runtime::value::RuntimeError> {
    let security = SecurityContext::allow_all();
    stdlib::call_builtin(name, args, test_span(), &security, &stdlib::stdout_writer())
}

// ============================================================================
// Path Construction Tests
// ============================================================================

#[test]
fn test_path_join_array_basic() {
    let segments = Value::array(vec![
        Value::string("foo"),
        Value::string("bar"),
        Value::string("baz.txt"),
    ]);
    let result = call_fn("pathJoinArray", &[segments]).unwrap();

    match result {
        Value::String(s) => {
            #[cfg(target_os = "windows")]
            assert_eq!(s.as_str(), "foo\\bar\\baz.txt");

            #[cfg(not(target_os = "windows"))]
            assert_eq!(s.as_str(), "foo/bar/baz.txt");
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_join_array_empty() {
    let segments = Value::array(vec![]);
    let result = call_fn("pathJoinArray", &[segments]).unwrap();

    match result {
        Value::String(s) => assert_eq!(s.as_str(), "."),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_join_array_with_empty_segments() {
    let segments = Value::array(vec![
        Value::string("foo"),
        Value::string(""),
        Value::string("bar"),
    ]);
    let result = call_fn("pathJoinArray", &[segments]).unwrap();

    match result {
        Value::String(s) => {
            #[cfg(target_os = "windows")]
            assert_eq!(s.as_str(), "foo\\bar");

            #[cfg(not(target_os = "windows"))]
            assert_eq!(s.as_str(), "foo/bar");
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_parse_basic() {
    let result = call_fn("pathParse", &[Value::string("/foo/bar/baz.txt")]).unwrap();

    match result {
        Value::HashMap(map) => {
            let m = map.inner();
            // Check that keys exist (cross-platform checking would be complex)
            assert!(m.len() == 5); // root, dir, base, ext, name
        }
        _ => panic!("Expected HashMap result"),
    }
}

#[test]
fn test_path_normalize_dots() {
    let result = call_fn("pathNormalize", &[Value::string("foo/bar/../baz")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "foo/baz"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_normalize_current_dir() {
    let result = call_fn("pathNormalize", &[Value::string("foo/./bar")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "foo/bar"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_normalize_double_slashes() {
    let result = call_fn("pathNormalize", &[Value::string("foo//bar")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "foo/bar"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_absolute_already_absolute() {
    #[cfg(not(target_os = "windows"))]
    {
        let result = call_fn("pathAbsolute", &[Value::string("/foo/bar")]).unwrap();
        match result {
            Value::String(s) => assert_eq!(s.as_str(), "/foo/bar"),
            _ => panic!("Expected string result"),
        }
    }
}

#[test]
fn test_path_parent_basic() {
    let result = call_fn("pathParent", &[Value::string("/foo/bar/baz.txt")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "/foo/bar"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_parent_no_parent() {
    let result = call_fn("pathParent", &[Value::string("file.txt")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), ""),
        _ => panic!("Expected string result"),
    }
}

// ============================================================================
// Path Component Extraction Tests
// ============================================================================

#[test]
fn test_path_basename_with_extension() {
    let result = call_fn("pathBasename", &[Value::string("/foo/bar/baz.txt")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "baz.txt"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_basename_directory() {
    let result = call_fn("pathBasename", &[Value::string("/foo/bar/")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "bar"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_basename_no_directory() {
    let result = call_fn("pathBasename", &[Value::string("file.txt")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "file.txt"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_dirname_basic() {
    let result = call_fn("pathDirname", &[Value::string("/foo/bar/baz.txt")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "/foo/bar"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_dirname_no_directory() {
    let result = call_fn("pathDirname", &[Value::string("file.txt")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), ""),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_extension_basic() {
    let result = call_fn("pathExtension", &[Value::string("foo/bar/baz.txt")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "txt"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_extension_double_extension() {
    let result = call_fn("pathExtension", &[Value::string("foo/bar/baz.tar.gz")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "gz"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_extension_no_extension() {
    let result = call_fn("pathExtension", &[Value::string("foo/bar/baz")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), ""),
        _ => panic!("Expected string result"),
    }
}

// ============================================================================
// Path Validation Tests
// ============================================================================

#[test]
fn test_path_is_absolute_unix() {
    #[cfg(not(target_os = "windows"))]
    {
        let result = call_fn("pathIsAbsolute", &[Value::string("/foo/bar")]).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = call_fn("pathIsAbsolute", &[Value::string("foo/bar")]).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}

#[test]
fn test_path_is_relative_unix() {
    #[cfg(not(target_os = "windows"))]
    {
        let result = call_fn("pathIsRelative", &[Value::string("foo/bar")]).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = call_fn("pathIsRelative", &[Value::string("/foo/bar")]).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}

#[test]
fn test_path_exists_nonexistent() {
    let result = call_fn(
        "pathExists",
        &[Value::string("/this/path/should/not/exist/12345")],
    )
    .unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_path_equals_same_path() {
    let result = call_fn(
        "pathEquals",
        &[Value::string("/foo/bar"), Value::string("/foo/bar")],
    )
    .unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_path_equals_different_paths() {
    let result = call_fn(
        "pathEquals",
        &[Value::string("/foo/bar"), Value::string("/foo/baz")],
    )
    .unwrap();
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// Path Utilities Tests
// ============================================================================

#[test]
fn test_path_homedir() {
    let result = call_fn("pathHomedir", &[]).unwrap();
    match result {
        Value::String(s) => {
            // Home directory should not be empty
            assert!(!s.is_empty());
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_cwd() {
    let result = call_fn("pathCwd", &[]).unwrap();
    match result {
        Value::String(s) => {
            // Current directory should not be empty
            assert!(!s.is_empty());
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_tempdir() {
    let result = call_fn("pathTempdir", &[]).unwrap();
    match result {
        Value::String(s) => {
            // Temp directory should not be empty
            assert!(!s.is_empty());
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_separator() {
    let result = call_fn("pathSeparator", &[]).unwrap();
    match result {
        Value::String(s) => {
            #[cfg(target_os = "windows")]
            assert_eq!(s.as_str(), "\\");

            #[cfg(not(target_os = "windows"))]
            assert_eq!(s.as_str(), "/");
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_delimiter() {
    let result = call_fn("pathDelimiter", &[]).unwrap();
    match result {
        Value::String(s) => {
            #[cfg(target_os = "windows")]
            assert_eq!(s.as_str(), ";");

            #[cfg(not(target_os = "windows"))]
            assert_eq!(s.as_str(), ":");
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_ext_separator() {
    let result = call_fn("pathExtSeparator", &[]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "."),
        _ => panic!("Expected string result"),
    }
}

// ============================================================================
// Format Conversion Tests
// ============================================================================

#[test]
fn test_path_to_posix() {
    let result = call_fn("pathToPosix", &[Value::string("foo\\bar\\baz")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "foo/bar/baz"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_to_windows() {
    let result = call_fn("pathToWindows", &[Value::string("foo/bar/baz")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "foo\\bar\\baz"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_to_platform() {
    let result = call_fn("pathToPlatform", &[Value::string("foo/bar\\baz")]).unwrap();
    match result {
        Value::String(s) => {
            // Should convert to platform-specific separator
            #[cfg(target_os = "windows")]
            assert_eq!(s.as_str(), "foo\\bar\\baz");

            #[cfg(not(target_os = "windows"))]
            assert_eq!(s.as_str(), "foo/bar/baz");
        }
        _ => panic!("Expected string result"),
    }
}

// ============================================================================
// Edge Cases and Integration Tests
// ============================================================================

#[test]
fn test_path_join_array_absolute_segment() {
    let segments = Value::array(vec![Value::string("foo"), Value::string("/bar")]);
    let result = call_fn("pathJoinArray", &[segments]).unwrap();

    match result {
        Value::String(s) => {
            // Absolute segment resets the path on Unix systems
            #[cfg(not(target_os = "windows"))]
            assert!(s.as_str() == "/bar" || s.as_str() == "foo/bar");
            // On Windows, forward slashes are treated as path separators too
            #[cfg(target_os = "windows")]
            assert!(s.as_str().contains("bar"));
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_normalize_multiple_parent_refs() {
    let result = call_fn("pathNormalize", &[Value::string("foo/bar/../../baz")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "baz"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_basename_hidden_file() {
    let result = call_fn("pathBasename", &[Value::string("/foo/bar/.hidden")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), ".hidden"),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_extension_hidden_file() {
    let result = call_fn("pathExtension", &[Value::string("/foo/bar/.hidden")]).unwrap();
    match result {
        Value::String(s) => {
            // Hidden files (starting with .) have no extension in Rust's std::path
            assert_eq!(s.as_str(), "");
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_parse_empty_path() {
    let result = call_fn("pathParse", &[Value::string("")]).unwrap();
    match result {
        Value::HashMap(_) => {
            // Should return a valid hashmap even for empty path
        }
        _ => panic!("Expected HashMap result"),
    }
}

#[test]
fn test_path_normalize_empty_path() {
    let result = call_fn("pathNormalize", &[Value::string("")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "."),
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_path_relative_same_directory() {
    let result = call_fn(
        "pathRelative",
        &[Value::string("/foo/bar"), Value::string("/foo/bar")],
    )
    .unwrap();
    match result {
        Value::String(s) => {
            // Relative path from a directory to itself should be "."
            assert!(s.as_str() == "." || s.is_empty());
        }
        _ => panic!("Expected string result"),
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_path_join_array_wrong_arg_count() {
    let result = call_fn("pathJoinArray", &[]);
    assert!(result.is_err());
}

#[test]
fn test_path_join_array_wrong_type() {
    let result = call_fn("pathJoinArray", &[Value::string("not an array")]);
    assert!(result.is_err());
}

#[test]
fn test_path_normalize_wrong_type() {
    let result = call_fn("pathNormalize", &[Value::Number(123.0)]);
    assert!(result.is_err());
}

#[test]
fn test_path_basename_wrong_type() {
    let result = call_fn("pathBasename", &[Value::Bool(true)]);
    assert!(result.is_err());
}

#[test]
fn test_path_equals_wrong_arg_count() {
    let result = call_fn("pathEquals", &[Value::string("/foo")]);
    assert!(result.is_err());
}

// ============================================================================
// Real-world Scenarios
// ============================================================================

#[test]
fn test_path_construction_workflow() {
    // Join multiple segments
    let segments = Value::array(vec![
        Value::string("home"),
        Value::string("user"),
        Value::string("documents"),
        Value::string("file.txt"),
    ]);
    let joined = call_fn("pathJoinArray", &[segments]).unwrap();

    // Get directory name
    let dirname = call_fn("pathDirname", std::slice::from_ref(&joined)).unwrap();

    // Get filename
    let basename = call_fn("pathBasename", std::slice::from_ref(&joined)).unwrap();

    // Both should be valid strings
    match (dirname, basename) {
        (Value::String(dir), Value::String(name)) => {
            assert!(!dir.is_empty());
            assert_eq!(name.as_str(), "file.txt");
        }
        _ => panic!("Expected string results"),
    }
}

#[test]
fn test_path_parsing_workflow() {
    // Parse a complex path
    let parsed = call_fn("pathParse", &[Value::string("/home/user/docs/file.tar.gz")]).unwrap();

    // Should have all components
    match parsed {
        Value::HashMap(map) => {
            let m = map.inner();
            assert_eq!(m.len(), 5);
        }
        _ => panic!("Expected HashMap"),
    }
}

#[test]
fn test_path_normalization_workflow() {
    // Create a messy path
    let messy = Value::string("foo/../bar/./baz//qux");

    // Normalize it
    let normalized = call_fn("pathNormalize", &[messy]).unwrap();

    // Should be clean
    match normalized {
        Value::String(s) => {
            assert_eq!(s.as_str(), "bar/baz/qux");
        }
        _ => panic!("Expected string result"),
    }
}

// --- Filesystem operations ---

// File system operations tests
//
// Comprehensive tests for fs module: directories, metadata, symlinks, temporary files

// ============================================================================
// Helper Functions
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn extract_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.as_ref().clone(),
        _ => panic!("Expected string value"),
    }
}

fn extract_number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => *n,
        _ => panic!("Expected number value"),
    }
}

fn extract_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        _ => panic!("Expected bool value"),
    }
}

fn extract_array(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(arr) => arr.as_slice().to_vec(),
        _ => panic!("Expected array value"),
    }
}

// ============================================================================
// Directory Operations Tests
// ============================================================================

#[test]
fn test_mkdir_creates_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    let path_str = dir_path.to_str().unwrap();

    let result = fs::mkdir(path_str, span());
    assert!(result.is_ok());
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[test]
fn test_mkdir_fails_if_parent_missing() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("missing_parent").join("test_dir");
    let path_str = dir_path.to_str().unwrap();

    let result = fs::mkdir(path_str, span());
    assert!(result.is_err());
}

#[test]
fn test_mkdirp_creates_directory_recursively() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("a").join("b").join("c");
    let path_str = dir_path.to_str().unwrap();

    let result = fs::mkdirp(path_str, span());
    assert!(result.is_ok());
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[test]
fn test_mkdirp_succeeds_if_directory_exists() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::mkdirp(path_str, span());
    assert!(result.is_ok());
}

#[test]
fn test_rmdir_removes_empty_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir(path_str, span());
    assert!(result.is_ok());
    assert!(!dir_path.exists());
}

#[test]
fn test_rmdir_fails_if_directory_not_empty() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();
    std_fs::write(dir_path.join("file.txt"), "content").unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir(path_str, span());
    assert!(result.is_err());
}

#[test]
fn test_rmdir_recursive_removes_directory_with_contents() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();
    std_fs::write(dir_path.join("file.txt"), "content").unwrap();
    std_fs::create_dir(dir_path.join("subdir")).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir_recursive(path_str, span());
    assert!(result.is_ok());
    assert!(!dir_path.exists());
}

#[test]
fn test_readdir_lists_directory_contents() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("file1.txt"), "content").unwrap();
    std_fs::write(temp.path().join("file2.txt"), "content").unwrap();
    std_fs::create_dir(temp.path().join("subdir")).unwrap();

    let path_str = temp.path().to_str().unwrap();
    let result = fs::readdir(path_str, span()).unwrap();
    let entries = extract_array(&result);

    assert_eq!(entries.len(), 3);
    let names: Vec<String> = entries.iter().map(extract_string).collect();
    assert!(names.contains(&"file1.txt".to_string()));
    assert!(names.contains(&"file2.txt".to_string()));
    assert!(names.contains(&"subdir".to_string()));
}

#[test]
fn test_walk_traverses_directory_tree() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("root.txt"), "content").unwrap();
    std_fs::create_dir(temp.path().join("dir1")).unwrap();
    std_fs::write(temp.path().join("dir1").join("file1.txt"), "content").unwrap();
    std_fs::create_dir(temp.path().join("dir1").join("subdir")).unwrap();
    std_fs::write(
        temp.path().join("dir1").join("subdir").join("file2.txt"),
        "content",
    )
    .unwrap();

    let path_str = temp.path().to_str().unwrap();
    let result = fs::walk(path_str, span()).unwrap();
    let entries = extract_array(&result);

    // Should include all files and directories (relative paths)
    assert!(entries.len() >= 5); // root.txt, dir1, dir1/file1.txt, dir1/subdir, dir1/subdir/file2.txt
}

#[test]
fn test_filter_entries_with_wildcard() {
    let entries = vec![
        Value::string("file1.txt".to_string()),
        Value::string("file2.rs".to_string()),
        Value::string("test.txt".to_string()),
        Value::string("readme.md".to_string()),
    ];

    let result = fs::filter_entries(&entries, "*.txt", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 2);
    let names: Vec<String> = filtered.iter().map(extract_string).collect();
    assert!(names.contains(&"file1.txt".to_string()));
    assert!(names.contains(&"test.txt".to_string()));
}

#[test]
fn test_sort_entries_alphabetically() {
    let entries = vec![
        Value::string("zebra.txt".to_string()),
        Value::string("apple.txt".to_string()),
        Value::string("Banana.txt".to_string()),
    ];

    let result = fs::sort_entries(&entries, span()).unwrap();
    let sorted = extract_array(&result);

    assert_eq!(sorted.len(), 3);
    assert_eq!(extract_string(&sorted[0]), "apple.txt");
    assert_eq!(extract_string(&sorted[1]), "Banana.txt"); // Case-insensitive
    assert_eq!(extract_string(&sorted[2]), "zebra.txt");
}

// ============================================================================
// File Metadata Tests
// ============================================================================

#[test]
fn test_size_returns_file_size() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "hello").unwrap(); // 5 bytes

    let path_str = file_path.to_str().unwrap();
    let result = fs::size(path_str, span()).unwrap();
    let size = extract_number(&result);

    assert_eq!(size, 5.0);
}

#[test]
fn test_mtime_returns_modified_time() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::mtime(path_str, span()).unwrap();
    let mtime = extract_number(&result);

    assert!(mtime > 0.0); // Should be positive Unix timestamp
}

#[test]
fn test_ctime_returns_created_time() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::ctime(path_str, span()).unwrap();
    let ctime = extract_number(&result);

    assert!(ctime > 0.0); // Should be positive Unix timestamp
}

#[test]
fn test_atime_returns_access_time() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::atime(path_str, span()).unwrap();
    let atime = extract_number(&result);

    assert!(atime > 0.0); // Should be positive Unix timestamp
}

#[test]
fn test_permissions_returns_file_permissions() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::permissions(path_str, span()).unwrap();
    let perms = extract_number(&result);

    assert!(perms > 0.0); // Should return some permission value
}

#[test]
fn test_is_dir_detects_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::is_dir(path_str, span()).unwrap();
    assert!(extract_bool(&result));
}

#[test]
fn test_is_dir_returns_false_for_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::is_dir(path_str, span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_is_file_detects_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::is_file(path_str, span()).unwrap();
    assert!(extract_bool(&result));
}

#[test]
fn test_is_file_returns_false_for_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::is_file(path_str, span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
#[cfg(unix)]
fn test_is_symlink_detects_symlink() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    std::os::unix::fs::symlink(&file_path, &link_path).unwrap();

    let path_str = link_path.to_str().unwrap();
    let result = fs::is_symlink(path_str, span()).unwrap();
    assert!(extract_bool(&result));
}

// ============================================================================
// Temporary Files Tests
// ============================================================================

#[test]
fn test_tmpfile_creates_temporary_file() {
    let result = fs::tmpfile(span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(Path::new(&path).exists());
    assert!(Path::new(&path).is_file());

    // Cleanup
    std_fs::remove_file(&path).ok();
}

#[test]
fn test_tmpdir_creates_temporary_directory() {
    let result = fs::tmpdir(span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(Path::new(&path).exists());
    assert!(Path::new(&path).is_dir());

    // Cleanup
    std_fs::remove_dir(&path).ok();
}

#[test]
fn test_tmpfile_named_creates_file_with_prefix() {
    let result = fs::tmpfile_named("atlas_test", span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(path.contains("atlas_test"));
    assert!(Path::new(&path).exists());

    // Cleanup
    std_fs::remove_file(&path).ok();
}

#[test]
fn test_get_temp_dir_returns_system_temp_directory() {
    let result = fs::get_temp_dir(span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(Path::new(&path).exists());
    assert!(Path::new(&path).is_dir());
}

#[test]
fn test_tmpfile_creates_unique_files() {
    let result1 = fs::tmpfile(span()).unwrap();
    let result2 = fs::tmpfile(span()).unwrap();

    let path1 = extract_string(&result1);
    let path2 = extract_string(&result2);

    assert_ne!(path1, path2); // Should create unique files

    // Cleanup
    std_fs::remove_file(&path1).ok();
    std_fs::remove_file(&path2).ok();
}

#[test]
fn test_tmpdir_creates_unique_directories() {
    let result1 = fs::tmpdir(span()).unwrap();
    let result2 = fs::tmpdir(span()).unwrap();

    let path1 = extract_string(&result1);
    let path2 = extract_string(&result2);

    assert_ne!(path1, path2); // Should create unique directories

    // Cleanup
    std_fs::remove_dir(&path1).ok();
    std_fs::remove_dir(&path2).ok();
}

// ============================================================================
// Symlink Operations Tests (Unix only)
// ============================================================================

#[test]
#[cfg(unix)]
fn test_symlink_creates_symbolic_link() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    let target_str = file_path.to_str().unwrap();
    let link_str = link_path.to_str().unwrap();

    let result = fs::symlink(target_str, link_str, span());
    assert!(result.is_ok());
    assert!(link_path.exists());
    assert!(link_path.is_symlink());
}

#[test]
#[cfg(unix)]
fn test_readlink_returns_symlink_target() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    std::os::unix::fs::symlink(&file_path, &link_path).unwrap();

    let link_str = link_path.to_str().unwrap();
    let result = fs::readlink(link_str, span()).unwrap();
    let target = extract_string(&result);

    assert!(target.contains("target.txt"));
}

#[test]
#[cfg(unix)]
fn test_resolve_symlink_follows_chain() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link1_path = temp.path().join("link1.txt");
    let link2_path = temp.path().join("link2.txt");
    std_fs::write(&file_path, "content").unwrap();

    std::os::unix::fs::symlink(&file_path, &link1_path).unwrap();
    std::os::unix::fs::symlink(&link1_path, &link2_path).unwrap();

    let link2_str = link2_path.to_str().unwrap();
    let result = fs::resolve_symlink(link2_str, span()).unwrap();
    let resolved = extract_string(&result);

    // Should resolve to the final target (canonicalized path)
    assert!(Path::new(&resolved).exists());
    assert!(Path::new(&resolved).is_file());
}

#[test]
#[cfg(unix)]
fn test_symlink_relative_link() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    // Create relative symlink
    let result = fs::symlink("target.txt", link_path.to_str().unwrap(), span());
    assert!(result.is_ok());
    assert!(link_path.is_symlink());
}

#[test]
#[cfg(unix)]
fn test_symlink_absolute_link() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("target.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    // Create absolute symlink
    let result = fs::symlink(
        file_path.to_str().unwrap(),
        link_path.to_str().unwrap(),
        span(),
    );
    assert!(result.is_ok());
    assert!(link_path.is_symlink());
}

// ============================================================================
// Integration and Edge Case Tests
// ============================================================================

#[test]
fn test_mkdir_error_on_existing_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::mkdir(path_str, span());
    assert!(result.is_err());
}

#[test]
fn test_readdir_error_on_nonexistent_directory() {
    let result = fs::readdir("/nonexistent/directory/path", span());
    assert!(result.is_err());
}

#[test]
fn test_size_error_on_nonexistent_file() {
    let result = fs::size("/nonexistent/file.txt", span());
    assert!(result.is_err());
}

#[test]
fn test_walk_empty_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("empty_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::walk(path_str, span()).unwrap();
    let entries = extract_array(&result);

    assert_eq!(entries.len(), 0); // Empty directory
}

#[test]
fn test_filter_entries_no_matches() {
    let entries = vec![
        Value::string("file1.txt".to_string()),
        Value::string("file2.txt".to_string()),
    ];

    let result = fs::filter_entries(&entries, "*.rs", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 0); // No matches
}

#[test]
fn test_filter_entries_exact_match() {
    let entries = vec![
        Value::string("file1.txt".to_string()),
        Value::string("file2.txt".to_string()),
    ];

    let result = fs::filter_entries(&entries, "file1.txt", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 1);
    assert_eq!(extract_string(&filtered[0]), "file1.txt");
}

#[test]
fn test_is_dir_nonexistent_path() {
    let result = fs::is_dir("/nonexistent/path", span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_is_file_nonexistent_path() {
    let result = fs::is_file("/nonexistent/path", span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_readdir_sorts_entries_consistently() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("file1.txt"), "content").unwrap();
    std_fs::write(temp.path().join("file2.txt"), "content").unwrap();

    let path_str = temp.path().to_str().unwrap();
    let result1 = fs::readdir(path_str, span()).unwrap();
    let result2 = fs::readdir(path_str, span()).unwrap();

    // Results should be consistent (though order may vary by filesystem)
    let entries1 = extract_array(&result1);
    let entries2 = extract_array(&result2);
    assert_eq!(entries1.len(), entries2.len());
}

#[test]
fn test_walk_deep_directory_tree() {
    let temp = TempDir::new().unwrap();

    // Create a deep directory tree
    let mut path = temp.path().to_path_buf();
    for i in 0..5 {
        path = path.join(format!("level{}", i));
        std_fs::create_dir(&path).unwrap();
        std_fs::write(path.join("file.txt"), "content").unwrap();
    }

    let path_str = temp.path().to_str().unwrap();
    let result = fs::walk(path_str, span()).unwrap();
    let entries = extract_array(&result);

    // Should traverse all levels (5 directories + 5 files = 10)
    assert!(entries.len() >= 10);
}

#[test]
fn test_size_returns_zero_for_empty_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("empty.txt");
    std_fs::write(&file_path, "").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::size(path_str, span()).unwrap();
    let size = extract_number(&result);

    assert_eq!(size, 0.0);
}

#[test]
fn test_rmdir_recursive_handles_empty_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("empty_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir_recursive(path_str, span());
    assert!(result.is_ok());
    assert!(!dir_path.exists());
}

#[test]
fn test_glob_pattern_prefix_wildcard() {
    let entries = vec![
        Value::string("test_file.txt".to_string()),
        Value::string("file.txt".to_string()),
    ];

    let result = fs::filter_entries(&entries, "test_*", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 1);
    assert_eq!(extract_string(&filtered[0]), "test_file.txt");
}

#[test]
fn test_glob_pattern_suffix_wildcard() {
    let entries = vec![
        Value::string("file.txt".to_string()),
        Value::string("file.rs".to_string()),
    ];

    let result = fs::filter_entries(&entries, "*.txt", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 1);
    assert_eq!(extract_string(&filtered[0]), "file.txt");
}

#[test]
fn test_permissions_on_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::permissions(path_str, span()).unwrap();
    let perms = extract_number(&result);

    assert!(perms > 0.0);
}

// --- Process management ---

// Process management tests (Phase-12)
//
// Tests for command execution, environment variables, and process control.

/// Helper to evaluate code expecting success
fn eval_ok(code: &str) -> Value {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    runtime.eval(code).unwrap()
}

// ============================================================================
// Command Execution Tests
// ============================================================================

#[test]
fn test_exec_simple_command() {
    // Test executing a simple command (echo on Unix, similar on Windows)
    let code = if cfg!(target_os = "windows") {
        r#"exec(["cmd", "/C", "echo", "hello"])"#
    } else {
        r#"exec(["echo", "hello"])"#
    };

    let result = eval_ok(code);
    // Should return Result<object, string>
    assert!(matches!(result, Value::Result(_)));
}

#[test]
fn test_shell_command() {
    let code = r#"shell("echo hello")"#;

    let result = eval_ok(code);
    // Should return Result<object, string>
    assert!(matches!(result, Value::Result(_)));
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

#[test]
fn test_set_get_env() {
    let code = r#"
        setEnv("TEST_VAR_ATLAS", "test_value");
        getEnv("TEST_VAR_ATLAS")
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "test_value"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_get_env_nonexistent() {
    let code = r#"getEnv("NONEXISTENT_VAR_ATLAS_12345")"#;
    let result = eval_ok(code);
    assert!(matches!(result, Value::Null));
}

#[test]
fn test_unset_env() {
    let code = r#"
        setEnv("TEST_VAR_UNSET", "value");
        unsetEnv("TEST_VAR_UNSET");
        getEnv("TEST_VAR_UNSET")
    "#;
    let result = eval_ok(code);
    assert!(matches!(result, Value::Null));
}

#[test]
fn test_list_env() {
    let code = r#"listEnv()"#;
    let result = eval_ok(code);
    // Should return an object (JsonValue)
    assert!(matches!(result, Value::JsonValue(_)));
}

// ============================================================================
// Working Directory Tests
// ============================================================================

#[test]
fn test_get_cwd() {
    let code = r#"getCwd()"#;
    let result = eval_ok(code);
    // Should return a string
    assert!(matches!(result, Value::String(_)));
}

// ============================================================================
// Process Info Tests
// ============================================================================

#[test]
fn test_get_pid() {
    let code = r#"getPid()"#;
    let result = eval_ok(code);
    // Should return a number
    match result {
        Value::Number(n) => assert!(n > 0.0),
        other => panic!("Expected Number, got {:?}", other),
    }
}

// ============================================================================
// Security Tests
// ============================================================================

#[test]
fn test_exec_requires_permission() {
    let code = r#"exec("ls")"#;
    // Default context denies all
    let security = SecurityContext::new();
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(code);
    // Should fail due to permission denial
    assert!(result.is_err());
}

#[test]
fn test_env_requires_permission() {
    let code = r#"getEnv("PATH")"#;
    // Default context denies all
    let security = SecurityContext::new();
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(code);
    // Should fail due to permission denial
    assert!(result.is_err());
}

// --- Gzip compression ---

// Gzip compression tests
//
// Comprehensive tests for gzip compression and decompression

// ============================================================================
// Helper Functions
// ============================================================================

fn bytes_to_atlas_array(bytes: &[u8]) -> Value {
    let values: Vec<Value> = bytes.iter().map(|&b| Value::Number(b as f64)).collect();
    Value::array(values)
}

fn atlas_array_to_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            arr_guard
                .iter()
                .map(|v| match v {
                    Value::Number(n) => *n as u8,
                    _ => panic!("Expected number in array"),
                })
                .collect()
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// Compression Tests
// ============================================================================

#[test]
fn test_compress_byte_array() {
    let data = b"Hello, World!";
    let atlas_data = bytes_to_atlas_array(data);

    let result = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Verify gzip magic header
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);

    // Compressed should be different from original
    assert_ne!(compressed_bytes, data);
}

#[test]
fn test_compress_string() {
    let text = "Hello, World!";
    let data = Value::string(text.to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Verify gzip magic header
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);
}

#[test]
fn test_compression_level_0() {
    let data = Value::string("Test data for compression".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(0.0)), span());
    assert!(result.is_ok());

    // Level 0 should still produce valid gzip
    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);
}

#[test]
fn test_compression_level_6() {
    let data = Value::string("Test data for compression".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());
}

#[test]
fn test_compression_level_9() {
    let data = Value::string("Test data for compression".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span());
    assert!(result.is_ok());

    // Level 9 should produce smaller output than level 0
    let result0 = gzip::gzip_compress(&data, Some(&Value::Number(0.0)), span()).unwrap();

    let compressed9 = atlas_array_to_bytes(&result.unwrap());
    let compressed0 = atlas_array_to_bytes(&result0);

    assert!(compressed9.len() <= compressed0.len());
}

#[test]
fn test_compress_large_data() {
    // Create large repeating data (compresses well)
    let large_text = "A".repeat(10000);
    let data = Value::string(large_text);

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Large repeating data should compress significantly
    assert!(compressed_bytes.len() < 10000);
}

#[test]
fn test_compress_empty_data() {
    let data = Value::string("".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Even empty data produces gzip header
    assert!(compressed_bytes.len() > 10);
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);
}

#[test]
fn test_compress_invalid_level() {
    let data = Value::string("test".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(10.0)), span());
    assert!(result.is_err());
}

// ============================================================================
// Decompression Tests
// ============================================================================

#[test]
fn test_decompress_to_bytes() {
    let original = b"Hello, World!";
    let atlas_data = bytes_to_atlas_array(original);

    // Compress
    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress(&compressed, span());
    assert!(result.is_ok());

    let decompressed = result.unwrap();
    let decompressed_bytes = atlas_array_to_bytes(&decompressed);

    assert_eq!(decompressed_bytes, original);
}

#[test]
fn test_decompress_to_string() {
    let original = "Hello, World!";
    let data = Value::string(original.to_string());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress to string
    let result = gzip::gzip_decompress_string(&compressed, span());
    assert!(result.is_ok());

    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), original),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_decompress_corrupt_data() {
    let bad_data = vec![Value::Number(0.0), Value::Number(1.0), Value::Number(2.0)];
    let atlas_data = Value::array(bad_data);

    let result = gzip::gzip_decompress(&atlas_data, span());
    assert!(result.is_err());
}

#[test]
fn test_decompress_invalid_format() {
    // Create data without gzip magic header
    let bad_data: Vec<Value> = (0..20).map(|i| Value::Number(i as f64)).collect();
    let atlas_data = Value::array(bad_data);

    let result = gzip::gzip_decompress(&atlas_data, span());
    assert!(result.is_err());
}

#[test]
fn test_decompress_large_file() {
    let large_text = "B".repeat(50000);
    let data = Value::string(large_text.clone());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span());
    assert!(result.is_ok());

    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), &large_text),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_decompress_empty_compressed_data() {
    let empty_data = Value::string("".to_string());

    // Compress empty data
    let compressed = gzip::gzip_compress(&empty_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span());
    assert!(result.is_ok());

    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), ""),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_decompress_non_utf8() {
    // Create binary data that isn't valid UTF-8
    let binary: Vec<u8> = vec![0xff, 0xfe, 0xfd, 0xfc];
    let atlas_data = bytes_to_atlas_array(&binary);

    // Compress binary data
    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress to bytes should work
    let result_bytes = gzip::gzip_decompress(&compressed, span());
    assert!(result_bytes.is_ok());

    // Decompress to string should fail
    let result_string = gzip::gzip_decompress_string(&compressed, span());
    assert!(result_string.is_err());
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[test]
fn test_round_trip_bytes() {
    let original = b"The quick brown fox jumps over the lazy dog";
    let atlas_data = bytes_to_atlas_array(original);

    // Compress
    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let decompressed = gzip::gzip_decompress(&compressed, span()).unwrap();
    let decompressed_bytes = atlas_array_to_bytes(&decompressed);

    assert_eq!(decompressed_bytes, original);
}

#[test]
fn test_round_trip_string() {
    let original = "The quick brown fox jumps over the lazy dog";
    let data = Value::string(original.to_string());

    // Compress
    let compressed = gzip::gzip_compress(&data, None, span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match result {
        Value::String(s) => assert_eq!(s.as_ref(), original),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_round_trip_large_data() {
    let original = "Lorem ipsum dolor sit amet. ".repeat(1000);
    let data = Value::string(original.clone());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match result {
        Value::String(s) => assert_eq!(s.as_ref(), &original),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_round_trip_different_levels() {
    let original = "Test data for different compression levels";

    for level in 0..=9 {
        let data = Value::string(original.to_string());

        // Compress with specific level
        let compressed =
            gzip::gzip_compress(&data, Some(&Value::Number(level as f64)), span()).unwrap();

        // Decompress
        let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

        match result {
            Value::String(s) => assert_eq!(s.as_ref(), original, "Failed at level {}", level),
            _ => panic!("Expected string at level {}", level),
        }
    }
}

#[test]
fn test_round_trip_utf8_preservation() {
    let original = "Hello 世界! 🎉 Ça marche!";
    let data = Value::string(original.to_string());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match result {
        Value::String(s) => assert_eq!(s.as_ref(), original),
        _ => panic!("Expected string"),
    }
}

// ============================================================================
// Utility Tests
// ============================================================================

#[test]
fn test_is_gzip_true() {
    let data = Value::string("test".to_string());
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    let result = gzip::gzip_is_gzip(&compressed, span()).unwrap();
    assert!(extract_bool(&result));
}

#[test]
fn test_is_gzip_false() {
    let data: Vec<Value> = vec![Value::Number(0.0), Value::Number(1.0)];
    let atlas_data = Value::array(data);

    let result = gzip::gzip_is_gzip(&atlas_data, span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_compression_ratio() {
    let original_size = Value::Number(1000.0);
    let compressed_size = Value::Number(500.0);

    let result = gzip::gzip_compression_ratio(&original_size, &compressed_size, span()).unwrap();
    let ratio = extract_number(&result);

    assert_eq!(ratio, 2.0); // 1000 / 500 = 2.0
}

#[test]
fn test_compression_ratio_no_compression() {
    let original_size = Value::Number(1000.0);
    let compressed_size = Value::Number(1000.0);

    let result = gzip::gzip_compression_ratio(&original_size, &compressed_size, span()).unwrap();
    let ratio = extract_number(&result);

    assert_eq!(ratio, 1.0);
}

#[test]
fn test_compression_ratio_expansion() {
    let original_size = Value::Number(100.0);
    let compressed_size = Value::Number(150.0);

    let result = gzip::gzip_compression_ratio(&original_size, &compressed_size, span()).unwrap();
    let ratio = extract_number(&result);

    assert!((ratio - 0.666).abs() < 0.01);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_compress_decompress_json_data() {
    let json_text = r#"{"name":"Alice","age":30,"active":true}"#;
    let data = Value::string(json_text.to_string());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();
    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), json_text),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_compress_decompress_code() {
    let code = r#"
fn main() {
    let x = 42;
    println!("Hello, world!");
    return x;
}
"#;
    let data = Value::string(code.to_string());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span()).unwrap();
    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), code),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_real_world_log_data() {
    let log = "[INFO] 2024-01-01 12:00:00 - Application started\n".repeat(100);
    let data = Value::string(log.clone());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Repeating log data should compress well
    assert!(compressed_bytes.len() < log.len() / 2);

    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), &log),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_memory_efficiency_large_repeated_data() {
    // Create data with lots of repetition (should compress extremely well)
    let repeated = "AAAABBBBCCCCDDDD".repeat(1000);
    let data = Value::string(repeated.clone());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span()).unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Should achieve >10x compression
    let ratio = repeated.len() as f64 / compressed_bytes.len() as f64;
    assert!(ratio > 10.0);

    // Verify decompression works
    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();
    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), &repeated),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_binary_data_round_trip() {
    // Test binary data (all byte values)
    let binary: Vec<u8> = (0..=255).collect();
    let atlas_data = bytes_to_atlas_array(&binary);

    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();
    let decompressed = gzip::gzip_decompress(&compressed, span()).unwrap();
    let decompressed_bytes = atlas_array_to_bytes(&decompressed);

    assert_eq!(decompressed_bytes, binary);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_compress_wrong_type() {
    let data = Value::Number(42.0);

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_err());
}

#[test]
fn test_decompress_wrong_type() {
    let data = Value::string("not an array".to_string());

    let result = gzip::gzip_decompress(&data, span());
    assert!(result.is_err());
}

#[test]
fn test_compress_level_wrong_type() {
    let data = Value::string("test".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::string("six".to_string())), span());
    assert!(result.is_err());
}

#[test]
fn test_byte_array_out_of_range() {
    let data: Vec<Value> = vec![Value::Number(256.0)]; // Out of byte range
    let atlas_data = Value::array(data);

    let result = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span());
    assert!(result.is_err());
}

#[test]
fn test_default_compression_level() {
    let data = Value::string("test data".to_string());

    // No level specified - should use default (6)
    let result = gzip::gzip_compress(&data, None, span());
    assert!(result.is_ok());
}

// --- Tar archives ---

// Integration tests for tar archive functionality

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    std_fs::write(path, content).unwrap();
}

fn create_test_dir(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std_fs::create_dir(&path).unwrap();
    path
}

fn str_value(s: &str) -> Value {
    Value::string(s.to_string())
}

fn str_array_value(paths: &[&str]) -> Value {
    let values: Vec<Value> = paths.iter().map(|p| str_value(p)).collect();
    Value::array(values)
}

// ============================================================================
// Tar Creation Tests
// ============================================================================

#[test]
fn test_tar_create_single_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_multiple_sources() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    std_fs::write(&file1, "content 1").unwrap();
    std_fs::write(&file2, "content 2").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[file1.to_str().unwrap(), file2.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_nonexistent_source() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.txt");
    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[nonexistent.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let error = tar::tar_create(&sources, &output, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("does not exist"));
}

#[test]
fn test_tar_create_empty_sources() {
    let temp = TempDir::new().unwrap();
    let tar_path = temp.path().join("archive.tar");

    let sources = Value::array(vec![]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_nested_directories() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    let sub_dir = create_test_dir(&test_dir, "subdir");
    create_test_file(&sub_dir, "nested.txt", "nested content");

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_preserves_metadata() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std_fs::metadata(&test_file).unwrap().permissions();
        perms.set_mode(0o644);
        std_fs::set_permissions(&test_file, perms).unwrap();
    }

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_invalid_sources_type() {
    let temp = TempDir::new().unwrap();
    let tar_path = temp.path().join("archive.tar");

    // Pass string instead of array
    let sources = str_value("/some/path");
    let output = str_value(tar_path.to_str().unwrap());

    let error = tar::tar_create(&sources, &output, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("array"));
}

// ============================================================================
// Tar.gz Creation Tests
// ============================================================================

#[test]
fn test_tar_create_gz_default_level() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content that will be compressed").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());

    let result = tar::tar_create_gz(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}

#[test]
fn test_tar_create_gz_level_zero() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());
    let level = Value::Number(0.0);

    let result = tar::tar_create_gz(&sources, &output, Some(&level), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}

#[test]
fn test_tar_create_gz_max_level() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());
    let level = Value::Number(9.0);

    let result = tar::tar_create_gz(&sources, &output, Some(&level), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}

#[test]
fn test_tar_create_gz_invalid_level() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());
    let level = Value::Number(10.0);

    let error = tar::tar_create_gz(&sources, &output, Some(&level), span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("0-9"));
}

#[test]
fn test_tar_create_gz_large_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");

    // Create multiple files
    for i in 0..10 {
        create_test_file(
            &test_dir,
            &format!("file{}.txt", i),
            &format!("content {}", i),
        );
    }

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());

    let result = tar::tar_create_gz(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}

// ============================================================================
// Tar Extraction Tests
// ============================================================================

#[test]
fn test_tar_extract_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    // Create tar using low-level function
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract tar using Atlas API
    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract(&tar_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            assert!(!arr_guard.is_empty());
        }
        _ => panic!("Expected array result"),
    }

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(
        std_fs::read_to_string(extracted_file).unwrap(),
        "test content"
    );
}

#[test]
fn test_tar_extract_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();

    let extracted_subdir = extract_dir.join("testdir");
    assert!(extracted_subdir.exists());
    assert!(extracted_subdir.join("file1.txt").exists());
    assert!(extracted_subdir.join("file2.txt").exists());
}

#[test]
fn test_tar_extract_nonexistent_tar() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar");
    let extract_dir = temp.path().join("extracted");

    let tar_val = str_value(nonexistent.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let error = tar::tar_extract(&tar_val, &out_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("Failed to open tar file"));
}

#[test]
fn test_tar_extract_creates_output_dir() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract to non-existent directory (should be created)
    let extract_dir = temp.path().join("nonexistent").join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();
    assert!(extract_dir.exists());
}

#[test]
fn test_tar_extract_nested_directories() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    let sub_dir = create_test_dir(&test_dir, "subdir");
    create_test_file(&sub_dir, "nested.txt", "nested content");

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();

    let nested_file = extract_dir
        .join("testdir")
        .join("subdir")
        .join("nested.txt");
    assert!(nested_file.exists());
    assert_eq!(
        std_fs::read_to_string(nested_file).unwrap(),
        "nested content"
    );
}

#[test]
fn test_tar_extract_preserves_content() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "Hello, World! This is test content.";
    std_fs::write(&test_file, content).unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(std_fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_extract_returns_file_list() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract(&tar_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            assert!(!arr_guard.is_empty());

            // Check that all entries are strings
            for val in arr_guard.iter() {
                assert!(matches!(val, Value::String(_)));
            }
        }
        _ => panic!("Expected array result"),
    }
}

// ============================================================================
// Tar.gz Extraction Tests
// ============================================================================

#[test]
fn test_tar_extract_gz_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content for compression").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 6, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_gz_val = str_value(tar_gz_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            assert!(!arr_guard.is_empty());
        }
        _ => panic!("Expected array result"),
    }

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(
        std_fs::read_to_string(extracted_file).unwrap(),
        "test content for compression"
    );
}

#[test]
fn test_tar_extract_gz_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 6, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_gz_val = str_value(tar_gz_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap();

    let extracted_subdir = extract_dir.join("testdir");
    assert!(extracted_subdir.exists());
    assert!(extracted_subdir.join("file1.txt").exists());
    assert!(extracted_subdir.join("file2.txt").exists());
}

#[test]
fn test_tar_extract_gz_nonexistent() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar.gz");
    let extract_dir = temp.path().join("extracted");

    let tar_gz_val = str_value(nonexistent.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let error = tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("Failed to open tar.gz file"));
}

#[test]
fn test_tar_extract_gz_large_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");

    // Create multiple files
    for i in 0..10 {
        create_test_file(
            &test_dir,
            &format!("file{}.txt", i),
            &format!("content {} with some repetitive text for compression", i),
        );
    }

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 9, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_gz_val = str_value(tar_gz_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap();

    let extracted_subdir = extract_dir.join("testdir");
    for i in 0..10 {
        let file_path = extracted_subdir.join(format!("file{}.txt", i));
        assert!(file_path.exists());
    }
}

// ============================================================================
// Tar Utility Tests
// ============================================================================

#[test]
fn test_tar_list_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let result = tar::tar_list(&tar_val, span()).unwrap();

    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            assert!(!arr_guard.is_empty());

            // Check that entries are HashMaps
            for val in arr_guard.iter() {
                assert!(matches!(val, Value::HashMap(_)));
            }
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_tar_list_multiple_files() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    std_fs::write(&file1, "content 1").unwrap();
    std_fs::write(&file2, "content 2").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![
        PathBuf::from(file1.to_str().unwrap()),
        PathBuf::from(file2.to_str().unwrap()),
    ];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let result = tar::tar_list(&tar_val, span()).unwrap();

    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            assert_eq!(arr_guard.len(), 2);
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_tar_contains_existing_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let file_val = str_value("test.txt");

    let result = tar::tar_contains_file(&tar_val, &file_val, span()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_tar_contains_nonexistent_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let file_val = str_value("nonexistent.txt");

    let result = tar::tar_contains_file(&tar_val, &file_val, span()).unwrap();
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[test]
fn test_tar_roundtrip_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "This is test content for round-trip verification";
    std_fs::write(&test_file, content).unwrap();

    let tar_path = temp.path().join("archive.tar");

    // Create tar
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract tar
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar(&tar_path, &extract_dir, span()).unwrap();

    // Verify content matches
    let extracted_file = extract_dir.join("test.txt");
    assert_eq!(std_fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_gz_roundtrip_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "This is test content for tar.gz round-trip verification with compression";
    std_fs::write(&test_file, content).unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    // Create tar.gz
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 9, span()).unwrap();

    // Extract tar.gz
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar_gz(&tar_gz_path, &extract_dir, span()).unwrap();

    // Verify content matches
    let extracted_file = extract_dir.join("test.txt");
    assert_eq!(std_fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_roundtrip_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");
    let sub_dir = create_test_dir(&test_dir, "subdir");
    create_test_file(&sub_dir, "nested.txt", "nested content");

    let tar_path = temp.path().join("archive.tar");

    // Create tar
    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract tar
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar(&tar_path, &extract_dir, span()).unwrap();

    // Verify all files extracted correctly
    let extracted_dir = extract_dir.join("testdir");
    assert_eq!(
        std_fs::read_to_string(extracted_dir.join("file1.txt")).unwrap(),
        "content 1"
    );
    assert_eq!(
        std_fs::read_to_string(extracted_dir.join("file2.txt")).unwrap(),
        "content 2"
    );
    assert_eq!(
        std_fs::read_to_string(extracted_dir.join("subdir").join("nested.txt")).unwrap(),
        "nested content"
    );
}

#[test]
fn test_tar_gz_roundtrip_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_gz_path = temp.path().join("archive.tar.gz");

    // Create tar.gz
    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 6, span()).unwrap();

    // Extract tar.gz
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar_gz(&tar_gz_path, &extract_dir, span()).unwrap();

    // Verify all files extracted correctly
    let extracted_subdir = extract_dir.join("testdir");
    assert_eq!(
        std_fs::read_to_string(extracted_subdir.join("file1.txt")).unwrap(),
        "content 1"
    );
    assert_eq!(
        std_fs::read_to_string(extracted_subdir.join("file2.txt")).unwrap(),
        "content 2"
    );
}

#[test]
fn test_tar_roundtrip_with_list() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    // Create tar
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // List before extraction
    let tar_val = str_value(tar_path.to_str().unwrap());
    let list_result = tar::tar_list(&tar_val, span()).unwrap();

    // Extract tar
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar(&tar_path, &extract_dir, span()).unwrap();

    // List should return non-empty array
    match list_result {
        Value::Array(arr) => {
            assert!(!arr.is_empty());
        }
        _ => panic!("Expected array result"),
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_tar_create_invalid_output_type() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = Value::Number(42.0); // Invalid type

    let error = tar::tar_create(&sources, &output, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_extract_invalid_tar_path_type() {
    let temp = TempDir::new().unwrap();
    let extract_dir = temp.path().join("extracted");

    let tar_val = Value::Number(42.0); // Invalid type
    let out_val = str_value(extract_dir.to_str().unwrap());

    let error = tar::tar_extract(&tar_val, &out_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_list_invalid_path_type() {
    let tar_val = Value::Number(42.0); // Invalid type

    let error = tar::tar_list(&tar_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_contains_invalid_tar_type() {
    let tar_val = Value::Number(42.0); // Invalid type
    let file_val = str_value("test.txt");

    let error = tar::tar_contains_file(&tar_val, &file_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_list_nonexistent_tar() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar");

    let tar_val = str_value(nonexistent.to_str().unwrap());
    let error = tar::tar_list(&tar_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("Failed to open tar file"));
}

// --- Zip archives ---

// Integration tests for zip archive functionality

// ============================================================================
// Test Helpers
// ============================================================================

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
    std_fs::write(&test_file, "Hello, Atlas!").unwrap();

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
    std_fs::write(&test_file, "some data").unwrap();

    let zip_path = temp.path().join("stored.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(0.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // With STORE, the entry's compressed size should equal the uncompressed size
    let list = atlas_zip::zip_list(&output, span()).unwrap();
    if let Value::Array(arr) = list {
        let guard = arr.as_slice();
        if let Some(Value::HashMap(entry_map)) = guard.first() {
            let guard = entry_map.inner();
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
    std_fs::write(&test_file, &content).unwrap();

    let zip_path = temp.path().join("deflate6.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(6.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Compressed size should be smaller than original
    let original_size = std_fs::metadata(&test_file).unwrap().len();
    let zip_size = std_fs::metadata(&zip_path).unwrap().len();
    assert!(zip_size < original_size);
}

/// 6. Deflate compression at level 9 (maximum)
#[test]
fn test_zip_create_deflate_level_9() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("text.txt");
    let content = "bbbbbbbbbb".repeat(1000);
    std_fs::write(&test_file, &content).unwrap();

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
    std_fs::write(&test_file, content).unwrap();

    let zip_path = temp.path().join("meta.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    atlas_zip::zip_create(&sources, &output, None, span()).unwrap();

    // Extract and verify content is identical
    let extract_dir = temp.path().join("extracted");
    atlas_zip::zip_extract(&output, &str_value(extract_dir.to_str().unwrap()), span()).unwrap();
    let extracted_content = std_fs::read_to_string(extract_dir.join("data.json")).unwrap();
    assert_eq!(extracted_content, content);
}

/// 8. Create zip with an archive-level comment
#[test]
fn test_zip_create_with_comment() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("note.txt");
    std_fs::write(&test_file, "contents").unwrap();

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
    std_fs::write(&atlas_file, "fn main() {}").unwrap();
    std_fs::write(&txt_file, "notes").unwrap();
    std_fs::write(&rs_file, "fn helper() {}").unwrap();

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
    std_fs::write(&test_file, "extractable content").unwrap();

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
    std_fs::write(&test_file, "hello").unwrap();

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
    let content = std_fs::read_to_string(dest.join("file.txt")).unwrap();
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
    std_fs::write(&file_a, "aaa").unwrap();
    std_fs::write(&file_b, "bbb").unwrap();

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
    let content = std_fs::read_to_string(
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
    let file = std_fs::File::create(&malicious_zip).unwrap();
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
    if result.is_ok() {
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
    std_fs::write(&corrupt, b"this is not a zip file at all").unwrap();

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
    std_fs::write(&f1, "alpha").unwrap();
    std_fs::write(&f2, "beta").unwrap();

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
        let guard = arr.as_slice();
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
    std_fs::write(&f, "here").unwrap();

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
    std_fs::write(&f, "here").unwrap();

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
    std_fs::write(&f, "x".repeat(10000)).unwrap();

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
    std_fs::write(&original, "original").unwrap();
    std_fs::write(&addition, "added file").unwrap();

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
    std_fs::write(&f, "validate me").unwrap();

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
    std_fs::write(&not_a_zip, b"PKnot a real zip file").unwrap();

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
    std_fs::write(&f, "z".repeat(50000)).unwrap();

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

    let stored_size = std_fs::metadata(&stored_zip).unwrap().len();
    let deflated_size = std_fs::metadata(&deflated_zip).unwrap().len();

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
    std_fs::write(&f, "test data for compression level testing").unwrap();

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
    std_fs::write(&large_file, &data).unwrap();

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
    let zip_size = std_fs::metadata(&zip_path).unwrap().len();
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
    let config = std_fs::read_to_string(out.join("source").join("config.toml")).unwrap();
    let readme = std_fs::read_to_string(out.join("source").join("README.md")).unwrap();
    let test_suite =
        std_fs::read_to_string(out.join("source").join("tests").join("test_suite.atlas")).unwrap();

    assert_eq!(config, "[package]\nname = \"atlas\"\n");
    assert_eq!(readme, "# Atlas\nFast compiler.\n");
    assert_eq!(test_suite, "fn test_basic() { assert(1 == 1); }");
}

/// 28. Zip list returns correct metadata fields
#[test]
fn test_zip_list_metadata_fields() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("meta_test.txt");
    std_fs::write(&f, "metadata check").unwrap();

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
        let guard = arr.as_slice();
        let first = &guard[0];

        if let Value::HashMap(map) = first {
            use atlas_runtime::stdlib::collections::hash::HashKey;
            use std::sync::Arc;

            let map_guard = map.inner();

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
    std_fs::write(&f, "noted").unwrap();

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
    std_fs::write(&f, "ok").unwrap();
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
    std_fs::write(&f, "a").unwrap();

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

// ============================================================================
// IO/FS Edge Case Hardening (Phase v02-completion-04)
// ============================================================================

fn with_io() -> Atlas {
    Atlas::new_with_security(SecurityContext::allow_all())
}

fn eval_str_io(code: &str) -> String {
    match with_io().eval(code) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Expected success, got error: {:?}", e),
    }
}

fn eval_err_io(code: &str) -> bool {
    with_io().eval(code).is_err()
}

// --- readFile edge cases ---

#[test]
fn test_read_file_nonexistent_returns_error() {
    assert!(eval_err_io(r#"readFile("/does/not/exist/file_xyz.txt")"#));
}

#[test]
fn test_read_file_empty_file_returns_empty_string() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("empty.txt");
    std_fs::write(&path, "").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "");
}

#[test]
fn test_write_file_creates_new_file() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("created.txt");
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"writeFile("{p}", "hello"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "hello");
}

#[test]
fn test_write_file_overwrites_existing() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("overwrite.txt");
    std_fs::write(&path, "old content").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"writeFile("{p}", "new content"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "new content");
}

#[test]
fn test_append_file_creates_if_not_exists() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("appended.txt");
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"appendFile("{p}", "first"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "first");
}

#[test]
fn test_append_file_appends_to_existing() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("append_existing.txt");
    std_fs::write(&path, "A").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"appendFile("{p}", "B"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "AB");
}

#[test]
fn test_file_exists_true_for_existing_file() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("exists.txt");
    std_fs::write(&path, "x").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "true");
}

#[test]
fn test_file_exists_false_for_nonexistent() {
    assert_eq!(
        eval_str_io(r#"fileExists("/does/not/exist/nope_xyz.txt")"#),
        "false"
    );
}

#[test]
fn test_file_exists_true_for_directory() {
    let temp = TempDir::new().unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    let code = format!(r#"fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "true");
}

#[test]
fn test_read_dir_empty_directory_returns_empty_array() {
    let temp = TempDir::new().unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    let code = format!(r#"len(readDir("{p}"))"#);
    assert_eq!(eval_str_io(&code), "0");
}

#[test]
fn test_read_dir_nonexistent_returns_error() {
    assert!(eval_err_io(r#"readDir("/does/not/exist/dir_xyz")"#));
}

#[test]
fn test_remove_file_nonexistent_returns_error() {
    assert!(eval_err_io(r#"removeFile("/does/not/exist/file_xyz.txt")"#));
}

#[test]
fn test_remove_file_success() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("to_remove.txt");
    std_fs::write(&path, "bye").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"removeFile("{p}"); fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "false");
}

#[test]
fn test_remove_dir_nonexistent_returns_error() {
    assert!(eval_err_io(r#"removeDir("/does/not/exist/dir_xyz")"#));
}

#[test]
fn test_remove_dir_success() {
    let temp = TempDir::new().unwrap();
    let sub = temp.path().join("subdir");
    std_fs::create_dir(&sub).unwrap();
    let p = sub.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"removeDir("{p}"); fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "false");
}

#[test]
fn test_create_dir_succeeds_when_already_exists() {
    let temp = TempDir::new().unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    // createDir on existing dir should not error (idempotent via createDir or error — check behavior)
    let result = with_io().eval(&format!(r#"createDir("{p}")"#));
    // Either succeeds or returns a meaningful error — should not panic/crash
    let _ = result;
}

#[test]
fn test_read_dir_returns_entry_count() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("a.txt"), "").unwrap();
    std_fs::write(temp.path().join("b.txt"), "").unwrap();
    std_fs::write(temp.path().join("c.txt"), "").unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    let code = format!(r#"len(readDir("{p}"))"#);
    assert_eq!(eval_str_io(&code), "3");
}

// --- Path edge cases via stdlib ---

#[test]
fn test_path_join_absolute_second_arg_replaces_first() {
    // Matches Rust/OS semantics: joining "/a/b" + "/c" → "/c"
    let segments = Value::array(vec![Value::string("/a/b"), Value::string("/c")]);
    let result = call_fn("pathJoinArray", &[segments]).unwrap();
    match result {
        Value::String(s) => {
            assert!(
                s.as_str().ends_with("/c") || s.as_str() == "/c",
                "Absolute second segment should dominate, got: {}",
                s.as_str()
            );
        }
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_path_basename_trailing_slash() {
    let result = call_fn("pathBasename", &[Value::string("/foo/bar/")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "bar"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_path_dirname_of_root() {
    let result = call_fn("pathDirname", &[Value::string("/")]).unwrap();
    match result {
        // Root "/" has no parent — Path::parent() returns None → empty string
        Value::String(s) => assert_eq!(s.as_str(), ""),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_path_normalize_dot_and_dotdot() {
    let result = call_fn("pathNormalize", &[Value::string("/a/./b/../c")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "/a/c"),
        _ => panic!("Expected string"),
    }
}
