// security.rs — Security model, runtime enforcement, and audit logging tests

use atlas_runtime::security::policy::{PolicyAction, PolicyRule, ResourceType};
use atlas_runtime::security::{
    Permission, PermissionSet, PolicyManager, ResourceQuotas, Sandbox, SecurityError,
    SecurityPolicy,
};
use atlas_runtime::{
    Atlas, AuditEvent, AuditLogger, DiagnosticLevel, MemoryAuditLogger, SecurityContext,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

/// Returns a platform-appropriate absolute path for testing.
/// On Unix: returns the path as-is (e.g., "/data/file.txt")
/// On Windows: prepends drive letter (e.g., "C:\data\file.txt")
#[cfg(windows)]
fn test_path(path: &str) -> PathBuf {
    let system_drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
    let windows_path = path.replace('/', "\\");
    PathBuf::from(format!("{}{}", system_drive, windows_path))
}

#[cfg(not(windows))]
fn test_path(path: &str) -> PathBuf {
    PathBuf::from(path)
}

// --- Permission model ---

// Comprehensive security and permission tests

// ============================================================================
// Permission Matching Tests
// ============================================================================

#[test]
fn test_filesystem_read_exact_match() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_recursive_allows_subdirs() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/subdir/file.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_recursive_allows_nested() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/a/b/c/file.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_non_recursive_denies_subdirs() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: false,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_different_paths() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/other/file.txt"),
        recursive: false,
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_filesystem_write_exact_match() {
    let allowed = Permission::FilesystemWrite {
        path: PathBuf::from("/output/result.txt"),
        recursive: false,
    };
    let requested = Permission::FilesystemWrite {
        path: PathBuf::from("/output/result.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_write_recursive() {
    let allowed = Permission::FilesystemWrite {
        path: PathBuf::from("/output"),
        recursive: true,
    };
    let requested = Permission::FilesystemWrite {
        path: PathBuf::from("/output/logs/app.log"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_exact_host_match() {
    let allowed = Permission::Network {
        host: "api.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "api.example.com".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_wildcard_subdomain() {
    let allowed = Permission::Network {
        host: "*.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "api.example.com".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_wildcard_nested_subdomain() {
    let allowed = Permission::Network {
        host: "*.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "api.v2.example.com".to_string(),
    };
    // *.example.com matches any subdomains (including nested)
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_wildcard_all() {
    let allowed = Permission::Network {
        host: "*".to_string(),
    };
    let requested = Permission::Network {
        host: "any.host.com".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_different_hosts() {
    let allowed = Permission::Network {
        host: "api.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "other.com".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_process_exact_command() {
    let allowed = Permission::Process {
        command: "git".to_string(),
    };
    let requested = Permission::Process {
        command: "git".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_process_wildcard() {
    let allowed = Permission::Process {
        command: "*".to_string(),
    };
    let requested = Permission::Process {
        command: "git".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_process_different_commands() {
    let allowed = Permission::Process {
        command: "git".to_string(),
    };
    let requested = Permission::Process {
        command: "npm".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_environment_exact_var() {
    let allowed = Permission::Environment {
        var: "PATH".to_string(),
    };
    let requested = Permission::Environment {
        var: "PATH".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_environment_wildcard() {
    let allowed = Permission::Environment {
        var: "*".to_string(),
    };
    let requested = Permission::Environment {
        var: "HOME".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_environment_different_vars() {
    let allowed = Permission::Environment {
        var: "PATH".to_string(),
    };
    let requested = Permission::Environment {
        var: "HOME".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

// ============================================================================
// Permission Type Mismatch Tests
// ============================================================================

#[test]
fn test_permission_type_mismatch_fs_vs_network() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::Network {
        host: "example.com".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_permission_type_mismatch_read_vs_write() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemWrite {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(!allowed.allows(&requested));
}

// ============================================================================
// PermissionSet Tests
// ============================================================================

#[test]
fn test_permission_set_empty_denies_all() {
    let set = PermissionSet::new();
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(!set.is_granted(&requested));
}

#[test]
fn test_permission_set_grant_and_check() {
    let mut set = PermissionSet::new();
    set.grant(Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    });

    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(set.is_granted(&requested));
}

#[test]
fn test_permission_set_multiple_permissions() {
    let mut set = PermissionSet::new();
    set.grant(Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    });
    set.grant(Permission::FilesystemRead {
        path: PathBuf::from("/config.txt"),
        recursive: false,
    });

    assert!(set.is_granted(&Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    }));
    assert!(set.is_granted(&Permission::FilesystemRead {
        path: PathBuf::from("/config.txt"),
        recursive: false,
    }));
    assert!(!set.is_granted(&Permission::FilesystemRead {
        path: PathBuf::from("/other.txt"),
        recursive: false,
    }));
}

// ============================================================================
// SecurityContext Tests
// ============================================================================

#[test]
fn test_security_context_default_denies_everything() {
    let ctx = SecurityContext::new();

    assert!(ctx
        .check_filesystem_read(&test_path("/data/file.txt"))
        .is_err());
    assert!(ctx
        .check_filesystem_write(&test_path("/output/file.txt"))
        .is_err());
    assert!(ctx.check_network("api.example.com").is_err());
    assert!(ctx.check_process("git").is_err());
    assert!(ctx.check_environment("PATH").is_err());
}

#[test]
fn test_security_context_grant_filesystem_read() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_read(&test_path("/data"), true);

    assert!(ctx
        .check_filesystem_read(&test_path("/data/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(&test_path("/data/subdir/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(&test_path("/other/file.txt"))
        .is_err());
}

#[test]
fn test_security_context_grant_filesystem_write() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_write(&test_path("/output"), true);

    assert!(ctx
        .check_filesystem_write(&test_path("/output/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_write(&test_path("/output/logs/app.log"))
        .is_ok());
    assert!(ctx
        .check_filesystem_write(&test_path("/other/file.txt"))
        .is_err());
}

#[test]
fn test_security_context_grant_network() {
    let mut ctx = SecurityContext::new();
    ctx.grant_network("api.example.com");

    assert!(ctx.check_network("api.example.com").is_ok());
    assert!(ctx.check_network("other.com").is_err());
}

#[test]
fn test_security_context_grant_network_wildcard() {
    let mut ctx = SecurityContext::new();
    ctx.grant_network("*.example.com");

    assert!(ctx.check_network("api.example.com").is_ok());
    assert!(ctx.check_network("cdn.example.com").is_ok());
    assert!(ctx.check_network("other.com").is_err());
}

#[test]
fn test_security_context_grant_process() {
    let mut ctx = SecurityContext::new();
    ctx.grant_process("git");

    assert!(ctx.check_process("git").is_ok());
    assert!(ctx.check_process("npm").is_err());
}

#[test]
fn test_security_context_grant_environment() {
    let mut ctx = SecurityContext::new();
    ctx.grant_environment("PATH");

    assert!(ctx.check_environment("PATH").is_ok());
    assert!(ctx.check_environment("HOME").is_err());
}

#[test]
fn test_security_context_allow_all() {
    let ctx = SecurityContext::allow_all();

    assert!(ctx.check_filesystem_read(&test_path("/any/path")).is_ok());
    assert!(ctx.check_filesystem_write(&test_path("/any/path")).is_ok());
    assert!(ctx.check_network("any.host.com").is_ok());
    assert!(ctx.check_process("any-command").is_ok());
    assert!(ctx.check_environment("ANY_VAR").is_ok());
}

// ============================================================================
// SecurityError Tests
// ============================================================================

#[test]
fn test_security_error_filesystem_read() {
    let ctx = SecurityContext::new();
    let result = ctx.check_filesystem_read(&test_path("/data/file.txt"));

    assert!(matches!(
        result,
        Err(SecurityError::FilesystemReadDenied { .. })
    ));
}

#[test]
fn test_security_error_filesystem_write() {
    let ctx = SecurityContext::new();
    let result = ctx.check_filesystem_write(&test_path("/output/file.txt"));

    assert!(matches!(
        result,
        Err(SecurityError::FilesystemWriteDenied { .. })
    ));
}

#[test]
fn test_security_error_network() {
    let ctx = SecurityContext::new();
    let result = ctx.check_network("api.example.com");

    assert!(matches!(result, Err(SecurityError::NetworkDenied { .. })));
}

#[test]
fn test_security_error_process() {
    let ctx = SecurityContext::new();
    let result = ctx.check_process("git");

    assert!(matches!(result, Err(SecurityError::ProcessDenied { .. })));
}

#[test]
fn test_security_error_environment() {
    let ctx = SecurityContext::new();
    let result = ctx.check_environment("PATH");

    assert!(matches!(
        result,
        Err(SecurityError::EnvironmentDenied { .. })
    ));
}

// ============================================================================
// Edge Cases and Security Tests
// ============================================================================

#[test]
fn test_empty_path_handling() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_read(Path::new(""), true);

    // Empty path should be treated as current directory
    assert!(ctx.check_filesystem_read(Path::new("")).is_ok());
}

#[test]
fn test_multiple_grants_same_type() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_read(&test_path("/data"), true);
    ctx.grant_filesystem_read(&test_path("/config"), true);

    assert!(ctx
        .check_filesystem_read(&test_path("/data/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(&test_path("/config/app.toml"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(&test_path("/other/file.txt"))
        .is_err());
}

#[test]
fn test_network_base_domain_match() {
    let allowed = Permission::Network {
        host: "*.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "example.com".to_string(),
    };
    // *.example.com should also match example.com (base domain)
    assert!(allowed.allows(&requested));
}

#[test]
fn test_security_context_isolation() {
    let mut ctx1 = SecurityContext::new();
    let ctx2 = SecurityContext::new();

    ctx1.grant_filesystem_read(Path::new("/data"), true);

    // ctx2 should not have ctx1's permissions
    assert!(ctx1
        .check_filesystem_read(Path::new("/data/file.txt"))
        .is_ok());
    assert!(ctx2
        .check_filesystem_read(Path::new("/data/file.txt"))
        .is_err());
}
// ============================================================================
// Phase-15 Integration Tests: Sandbox, Policy, Capabilities
// ============================================================================

#[test]
fn test_sandbox_complete_lifecycle() {
    let sandbox = Sandbox::restrictive("test-lifecycle".to_string());

    assert_eq!(sandbox.id(), "test-lifecycle");
    assert!(sandbox.is_enabled());

    // Sandbox is dropped at end of scope, triggering audit log
}

#[test]
fn test_sandbox_resource_quotas_enforced() {
    let quotas = ResourceQuotas {
        memory_limit: Some(1000),
        file_descriptor_limit: Some(5),
        ..ResourceQuotas::default()
    };

    let sandbox = Sandbox::new("test-quotas".to_string(), PermissionSet::new(), quotas);

    // Memory quota
    assert!(sandbox.allocate_memory(500).is_ok());
    assert!(sandbox.allocate_memory(600).is_err());

    // File descriptor quota
    for _ in 0..5 {
        assert!(sandbox.allocate_file_descriptor().is_ok());
    }
    assert!(sandbox.allocate_file_descriptor().is_err());
}

#[test]
fn test_security_policy_enforcement() {
    let mut policy = SecurityPolicy::new("test-policy".to_string());
    policy.default_action = PolicyAction::Deny;

    policy.allow.push(PolicyRule {
        resource: ResourceType::FileRead,
        pattern: "/allowed/*".to_string(),
        scope: None,
        description: None,
    });

    assert!(policy.allows(&ResourceType::FileRead, "/allowed/file.txt"));
    assert!(!policy.allows(&ResourceType::FileRead, "/denied/file.txt"));
}

#[test]
fn test_policy_manager_inheritance() {
    let mut manager = PolicyManager::new();

    // Base policy
    let mut base = SecurityPolicy::new("base".to_string());
    base.allow.push(PolicyRule {
        resource: ResourceType::FileRead,
        pattern: "/shared/*".to_string(),
        scope: None,
        description: None,
    });
    manager.load_policy(base).unwrap();

    // Derived policy
    let mut derived = SecurityPolicy::new("derived".to_string());
    derived.inherits.push("base".to_string());
    derived.allow.push(PolicyRule {
        resource: ResourceType::FileWrite,
        pattern: "/temp/*".to_string(),
        scope: None,
        description: None,
    });
    manager.load_policy(derived).unwrap();

    // Derived should have both base and its own permissions
    let perms = manager.get_permissions("derived").unwrap();
    assert!(perms.len() >= 2);
}

#[test]
fn test_permission_set_operations() {
    let mut set1 = PermissionSet::new();
    set1.grant(Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    });

    let mut set2 = PermissionSet::new();
    set2.grant(Permission::FilesystemWrite {
        path: PathBuf::from("/output"),
        recursive: true,
    });

    // Merge sets
    set1.merge(&set2);

    // set1 should now have both permissions
    assert_eq!(set1.len(), 2);
    assert!(set1.is_granted(&Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    }));
    assert!(set1.is_granted(&Permission::FilesystemWrite {
        path: PathBuf::from("/output/file.txt"),
        recursive: false,
    }));
}

#[test]
fn test_sandbox_permission_integration() {
    let mut sandbox = Sandbox::restrictive("test-perms".to_string());

    // Grant specific permission
    sandbox.grant_permission(Permission::FilesystemRead {
        path: PathBuf::from("/allowed"),
        recursive: true,
    });

    assert_eq!(sandbox.permissions().len(), 1);
}

#[test]
fn test_resource_quota_monitoring() {
    let sandbox = Sandbox::new(
        "test-monitoring".to_string(),
        PermissionSet::new(),
        ResourceQuotas::permissive(),
    );

    sandbox.allocate_memory(1000).unwrap();
    sandbox.allocate_file_descriptor().unwrap();
    sandbox.allocate_network_connection().unwrap();

    let usage = sandbox.usage();
    assert_eq!(usage.memory_used, 1000);
    assert_eq!(usage.file_descriptors_used, 1);
    assert_eq!(usage.network_connections_used, 1);
}

#[test]
fn test_policy_deny_rules_override_allow() {
    let mut policy = SecurityPolicy::new("test-deny-priority".to_string());
    policy.default_action = PolicyAction::Allow;

    // Allow all files
    policy.allow.push(PolicyRule {
        resource: ResourceType::FileRead,
        pattern: "/*".to_string(),
        scope: None,
        description: None,
    });

    // But deny /etc specifically
    policy.deny.push(PolicyRule {
        resource: ResourceType::FileRead,
        pattern: "/etc/*".to_string(),
        scope: None,
        description: None,
    });

    assert!(policy.allows(&ResourceType::FileRead, "/home/file.txt"));
    assert!(!policy.allows(&ResourceType::FileRead, "/etc/passwd"));
}

#[test]
fn test_sandbox_disabled_bypasses_checks() {
    let quotas = ResourceQuotas {
        memory_limit: Some(100),
        ..ResourceQuotas::default()
    };

    let mut sandbox = Sandbox::new("test-disable".to_string(), PermissionSet::new(), quotas);

    // Should fail with sandbox enabled
    assert!(sandbox.allocate_memory(200).is_err());

    // Disable sandbox
    sandbox.disable();

    // Should succeed now
    assert!(sandbox.allocate_memory(200).is_ok());
}

// --- Runtime security enforcement ---

// Runtime security enforcement tests
//
// Tests that security checks are enforced at runtime for I/O operations.

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

// --- Audit logging ---

// Audit logging tests
//
// Tests that all security events are properly logged for monitoring and compliance.

// ============================================================================
// Audit Logging Integration Tests
// ============================================================================

#[test]
fn test_audit_logger_logs_filesystem_read_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_read(&test_path("/etc/passwd"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::FilesystemReadDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_filesystem_read_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_read(&test_path("/data"), true);
    let _ = ctx.check_filesystem_read(&test_path("/data/file.txt"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_filesystem_write_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_write(&test_path("/etc/passwd"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::FilesystemWriteDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_filesystem_write_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_write(&test_path("/output"), true);
    let _ = ctx.check_filesystem_write(&test_path("/output/file.txt"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_network_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::NetworkDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_network_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_network("api.example.com");
    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_process_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_process("git");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::ProcessDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_process_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_process("git");
    let _ = ctx.check_process("git");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_environment_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_environment("PATH");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::EnvironmentDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_environment_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_environment("PATH");
    let _ = ctx.check_environment("PATH");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

// ============================================================================
// Multiple Events Tests
// ============================================================================

#[test]
fn test_audit_logger_logs_multiple_events() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_read(&test_path("/data"), true);

    // Multiple permission checks
    let _ = ctx.check_filesystem_read(&test_path("/data/file1.txt"));
    let _ = ctx.check_filesystem_read(&test_path("/data/file2.txt"));
    let _ = ctx.check_filesystem_read(&test_path("/etc/passwd"));
    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 4);
}

#[test]
fn test_audit_logger_logs_granted_and_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_read(&test_path("/data"), true);

    // Granted
    let _ = ctx.check_filesystem_read(&test_path("/data/file.txt"));

    // Denied
    let _ = ctx.check_filesystem_read(&test_path("/etc/passwd"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 2);

    // First should be granted
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));

    // Second should be denied
    assert!(matches!(
        &entries[1].event,
        AuditEvent::FilesystemReadDenied { .. }
    ));
}

// ============================================================================
// Audit Entry Format Tests
// ============================================================================

#[test]
fn test_audit_entry_has_timestamp() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].timestamp > 0);
}

#[test]
fn test_audit_entry_log_line_format() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_read(&test_path("/etc/passwd"));

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();

    assert!(log_line.contains("Permission denied"));
    assert!(log_line.contains("file read"));
    // Path format varies by platform (Unix: /etc/passwd, Windows: C:\etc\passwd)
    assert!(log_line.contains("passwd"));
    assert!(log_line.starts_with('[')); // Has timestamp
}

#[test]
fn test_audit_event_display_filesystem_read_denied() {
    let event = AuditEvent::FilesystemReadDenied {
        path: Path::new("/etc/passwd").to_path_buf(),
    };
    let display = event.to_string();

    assert!(display.contains("Permission denied"));
    assert!(display.contains("file read"));
    assert!(display.contains("/etc/passwd"));
}

#[test]
fn test_audit_event_display_network_denied() {
    let event = AuditEvent::NetworkDenied {
        host: "evil.com".to_string(),
    };
    let display = event.to_string();

    assert!(display.contains("Permission denied"));
    assert!(display.contains("network access"));
    assert!(display.contains("evil.com"));
}

#[test]
fn test_audit_event_display_permission_check_granted() {
    let event = AuditEvent::PermissionCheck {
        operation: "file read".to_string(),
        target: "/data/file.txt".to_string(),
        granted: true,
    };
    let display = event.to_string();

    assert!(display.contains("GRANTED"));
    assert!(display.contains("file read"));
    assert!(display.contains("/data/file.txt"));
}

#[test]
fn test_audit_event_display_permission_check_denied() {
    let event = AuditEvent::PermissionCheck {
        operation: "network".to_string(),
        target: "evil.com".to_string(),
        granted: false,
    };
    let display = event.to_string();

    assert!(display.contains("DENIED"));
    assert!(display.contains("network"));
    assert!(display.contains("evil.com"));
}

// ============================================================================
// Audit Logger Clear Tests
// ============================================================================

#[test]
fn test_audit_logger_clear() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api.example.com");
    assert_eq!(logger.entries().len(), 1);

    logger.clear();
    assert_eq!(logger.entries().len(), 0);
}

#[test]
fn test_audit_logger_clear_and_continue() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api1.example.com");
    logger.clear();

    let _ = ctx.check_network("api2.example.com");
    assert_eq!(logger.entries().len(), 1);
}

// ============================================================================
// Default Context (No Audit Logger) Tests
// ============================================================================

#[test]
fn test_default_context_has_null_logger() {
    // Default context should have NullAuditLogger (no logging overhead)
    let ctx = SecurityContext::new();

    // Should not panic or cause errors
    let _ = ctx.check_filesystem_read(&test_path("/etc/passwd"));
    let _ = ctx.check_network("api.example.com");
}

#[test]
fn test_default_context_audit_logger_is_null() {
    let ctx = SecurityContext::new();

    // Perform some checks
    let _ = ctx.check_filesystem_read(&test_path("/etc/passwd"));
    let _ = ctx.check_network("api.example.com");

    // Get audit logger and verify it returns no entries
    let logger = ctx.audit_logger();
    assert_eq!(logger.entries().len(), 0); // NullAuditLogger returns empty
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[test]
fn test_audit_logger_thread_safe() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = Arc::new(SecurityContext::with_audit_logger(
        logger.clone() as Arc<dyn AuditLogger>
    ));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let ctx = Arc::clone(&ctx);
            thread::spawn(move || {
                let _ = ctx.check_network(&format!("api{}.example.com", i));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // All 10 events should be logged
    assert_eq!(logger.entries().len(), 10);
}

#[test]
fn test_security_context_clone_shares_logger() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx1 = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let ctx2 = ctx1.clone();

    let _ = ctx1.check_network("api1.example.com");
    let _ = ctx2.check_network("api2.example.com");

    // Both contexts share the same logger
    assert_eq!(logger.entries().len(), 2);
}

// ============================================================================
// Event Details Tests
// ============================================================================

#[test]
fn test_filesystem_read_denied_event_includes_path() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_read(&test_path("/secret/data.txt"));

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    // Path format varies by platform, but should contain filename
    assert!(log_line.contains("data.txt"));
}

#[test]
fn test_network_denied_event_includes_host() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("malicious.com");

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    assert!(log_line.contains("malicious.com"));
}

#[test]
fn test_process_denied_event_includes_command() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_process("rm");

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    assert!(log_line.contains("rm"));
}

#[test]
fn test_environment_denied_event_includes_var() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_environment("SECRET_KEY");

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    assert!(log_line.contains("SECRET_KEY"));
}
