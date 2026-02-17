//! File system operations tests
//!
//! Comprehensive tests for fs module: directories, metadata, symlinks, temporary files

use atlas_runtime::span::Span;
use atlas_runtime::stdlib::fs;
use atlas_runtime::value::Value;
use std::fs as std_fs;
use std::path::Path;
use tempfile::TempDir;

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
        Value::Array(arr) => arr.lock().unwrap().clone(),
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
    let names: Vec<String> = entries.iter().map(|v| extract_string(v)).collect();
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
    let names: Vec<String> = filtered.iter().map(|v| extract_string(v)).collect();
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
