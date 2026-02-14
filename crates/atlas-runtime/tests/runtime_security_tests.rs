//! Runtime security enforcement tests
//!
//! Tests that security checks are enforced at runtime for I/O operations.

use atlas_runtime::{Atlas, DiagnosticLevel, SecurityContext};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Runtime Integration Tests
// ============================================================================

#[test]
fn test_default_runtime_denies_file_reads() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 42;").unwrap();

    // Default runtime should deny file reads
    let runtime = Atlas::new();
    let result = runtime.eval_file(test_file.to_str().unwrap());

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "AT0300");
    assert!(diagnostics[0].message.contains("Permission denied"));
    assert!(diagnostics[0].message.contains("file read"));
}

#[test]
fn test_allow_all_runtime_permits_file_reads() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 42;").unwrap();

    // Runtime with allow_all security should permit file reads
    let runtime = Atlas::new_with_security(SecurityContext::allow_all());
    let result = runtime.eval_file(test_file.to_str().unwrap());

    assert!(result.is_ok());
}

#[test]
fn test_granular_permission_allows_specific_path() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_file = temp_dir.path().join("allowed.atl");
    let denied_file = temp_dir.path().join("denied.atl");

    fs::write(&allowed_file, "let x = 1;").unwrap();
    fs::write(&denied_file, "let y = 2;").unwrap();

    // Grant read permission only to allowed_file
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(&allowed_file, false);

    let runtime = Atlas::new_with_security(security);

    // Should allow reading allowed_file
    let result = runtime.eval_file(allowed_file.to_str().unwrap());
    assert!(result.is_ok());

    // Should deny reading denied_file
    let result = runtime.eval_file(denied_file.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_recursive_permission_allows_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let test_file = subdir.join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    // Grant recursive read permission to temp_dir
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);

    let runtime = Atlas::new_with_security(security);

    // Should allow reading file in subdirectory
    let result = runtime.eval_file(test_file.to_str().unwrap());
    assert!(result.is_ok());
}

#[test]
fn test_non_recursive_permission_denies_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let test_file = subdir.join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    // Grant non-recursive read permission to temp_dir
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), false);

    let runtime = Atlas::new_with_security(security);

    // Should deny reading file in subdirectory
    let result = runtime.eval_file(test_file.to_str().unwrap());
    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

#[test]
fn test_permission_error_includes_path() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("secret.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let runtime = Atlas::new();
    let result = runtime.eval_file(test_file.to_str().unwrap());

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert!(diagnostics[0].message.contains("secret.atl"));
}

#[test]
fn test_eval_does_not_require_filesystem_permission() {
    // eval() should work without filesystem permissions since it doesn't touch files
    let runtime = Atlas::new();
    let result = runtime.eval("1 + 2");
    assert!(result.is_ok());
}

// ============================================================================
// Security Context Configuration Tests
// ============================================================================

#[test]
fn test_runtime_with_new_has_deny_all_security() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let runtime = Atlas::new();

    // Should deny file access
    let result = runtime.eval_file(test_file.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_runtime_with_allow_all_permits_all_operations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let runtime = Atlas::new_with_security(SecurityContext::allow_all());

    // Should allow file access
    let result = runtime.eval_file(test_file.to_str().unwrap());
    assert!(result.is_ok());
}

#[test]
fn test_runtime_respects_custom_security_context() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);

    let runtime = Atlas::new_with_security(security);

    // Should allow file access within granted path
    let result = runtime.eval_file(test_file.to_str().unwrap());
    assert!(result.is_ok());
}

// ============================================================================
// Error Code Tests
// ============================================================================

#[test]
fn test_filesystem_permission_denied_uses_at0300() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let runtime = Atlas::new();
    let result = runtime.eval_file(test_file.to_str().unwrap());

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

#[test]
fn test_permission_error_message_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let runtime = Atlas::new();
    let result = runtime.eval_file(test_file.to_str().unwrap());

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();

    let message = &diagnostics[0].message;
    assert!(message.contains("Permission denied"));
    assert!(message.contains("file read"));
    assert!(message.contains("test.atl"));
}

// ============================================================================
// Path Traversal Protection Tests
// ============================================================================

#[test]
fn test_path_traversal_protection() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("allowed");
    fs::create_dir(&subdir).unwrap();

    let allowed_file = subdir.join("test.atl");
    fs::write(&allowed_file, "let x = 1;").unwrap();

    // Create a file outside the allowed directory
    let parent_file = temp_dir.path().join("parent.atl");
    fs::write(&parent_file, "let y = 2;").unwrap();

    // Grant permission only to subdir
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(&subdir, true);

    let runtime = Atlas::new_with_security(security);

    // Should allow reading allowed_file
    let result = runtime.eval_file(allowed_file.to_str().unwrap());
    assert!(result.is_ok());

    // Should deny reading parent_file (path traversal attempt)
    let result = runtime.eval_file(parent_file.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_exact_path_match_security() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("exact.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    // Grant permission to exact file (non-recursive)
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(&test_file, false);

    let runtime = Atlas::new_with_security(security);

    // Should allow reading exact file
    let result = runtime.eval_file(test_file.to_str().unwrap());
    assert!(result.is_ok());

    // Create another file in same directory
    let other_file = temp_dir.path().join("other.atl");
    fs::write(&other_file, "let y = 2;").unwrap();

    // Should deny reading other file
    let result = runtime.eval_file(other_file.to_str().unwrap());
    assert!(result.is_err());
}

// ============================================================================
// Module System Integration Tests
// ============================================================================

#[test]
fn test_module_imports_respect_permissions() {
    let temp_dir = TempDir::new().unwrap();

    // Create helper module
    let helper_file = temp_dir.path().join("helper.atl");
    fs::write(&helper_file, "export let value = 42;").unwrap();

    // Create main module that imports helper
    let main_file = temp_dir.path().join("main.atl");
    fs::write(&main_file, "import { value } from \"./helper\";").unwrap();

    // Grant permission to entire temp_dir
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);

    let runtime = Atlas::new_with_security(security);

    // Should allow reading main file and imported modules
    let result = runtime.eval_file(main_file.to_str().unwrap());
    assert!(result.is_ok());
}

// NOTE: Module executor security integration is a future enhancement
// For now, only eval_file has security checks. Module imports are not yet secured.
#[test]
#[ignore = "ModuleExecutor security integration not yet implemented"]
fn test_module_imports_denied_without_permission() {
    let temp_dir = TempDir::new().unwrap();

    // Create helper module
    let helper_file = temp_dir.path().join("helper.atl");
    fs::write(&helper_file, "export let value = 42;").unwrap();

    // Create main module that imports helper
    let main_file = temp_dir.path().join("main.atl");
    fs::write(&main_file, "import { value } from \"./helper\";").unwrap();

    // Grant permission only to main file (not helper)
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(&main_file, false);

    let runtime = Atlas::new_with_security(security);

    // Should deny because helper.atl cannot be read
    let result = runtime.eval_file(main_file.to_str().unwrap());
    // Note: This might fail at main file read or helper file read
    // depending on how module executor handles permissions
    assert!(result.is_err());
}

// ============================================================================
// Diagnostic Quality Tests
// ============================================================================

#[test]
fn test_permission_denied_diagnostic_is_error_level() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let runtime = Atlas::new();
    let result = runtime.eval_file(test_file.to_str().unwrap());

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
}

#[test]
fn test_permission_denied_has_code_and_message() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.atl");
    fs::write(&test_file, "let x = 1;").unwrap();

    let runtime = Atlas::new();
    let result = runtime.eval_file(test_file.to_str().unwrap());

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();

    assert!(!diagnostics[0].code.is_empty());
    assert!(!diagnostics[0].message.is_empty());
}
