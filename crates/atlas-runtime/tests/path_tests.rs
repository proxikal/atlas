//! Integration tests for path manipulation functions
//!
//! Tests all path functions through the stdlib interface

use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib;
use atlas_runtime::value::Value;

fn test_span() -> Span {
    Span::dummy()
}

fn call_fn(name: &str, args: &[Value]) -> Result<Value, atlas_runtime::value::RuntimeError> {
    let security = SecurityContext::allow_all();
    stdlib::call_builtin(name, args, test_span(), &security)
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
            let m = map.lock().unwrap();
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
    let dirname = call_fn("pathDirname", &[joined.clone()]).unwrap();

    // Get filename
    let basename = call_fn("pathBasename", &[joined.clone()]).unwrap();

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
            let m = map.lock().unwrap();
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
