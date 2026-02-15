//! Comprehensive security and permission tests

use atlas_runtime::security::{
    Permission, PermissionSet, PolicyManager, ResourceQuotas, Sandbox, SecurityContext,
    SecurityError, SecurityPolicy,
};
use atlas_runtime::security::policy::{PolicyAction, PolicyRule, ResourceType};
use std::path::{Path, PathBuf};

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
        .check_filesystem_read(Path::new("/data/file.txt"))
        .is_err());
    assert!(ctx
        .check_filesystem_write(Path::new("/output/file.txt"))
        .is_err());
    assert!(ctx.check_network("api.example.com").is_err());
    assert!(ctx.check_process("git").is_err());
    assert!(ctx.check_environment("PATH").is_err());
}

#[test]
fn test_security_context_grant_filesystem_read() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_read(Path::new("/data"), true);

    assert!(ctx
        .check_filesystem_read(Path::new("/data/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(Path::new("/data/subdir/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(Path::new("/other/file.txt"))
        .is_err());
}

#[test]
fn test_security_context_grant_filesystem_write() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_write(Path::new("/output"), true);

    assert!(ctx
        .check_filesystem_write(Path::new("/output/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_write(Path::new("/output/logs/app.log"))
        .is_ok());
    assert!(ctx
        .check_filesystem_write(Path::new("/other/file.txt"))
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

    assert!(ctx.check_filesystem_read(Path::new("/any/path")).is_ok());
    assert!(ctx.check_filesystem_write(Path::new("/any/path")).is_ok());
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
    let result = ctx.check_filesystem_read(Path::new("/data/file.txt"));

    assert!(matches!(
        result,
        Err(SecurityError::FilesystemReadDenied { .. })
    ));
}

#[test]
fn test_security_error_filesystem_write() {
    let ctx = SecurityContext::new();
    let result = ctx.check_filesystem_write(Path::new("/output/file.txt"));

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
    ctx.grant_filesystem_read(Path::new("/data"), true);
    ctx.grant_filesystem_read(Path::new("/config"), true);

    assert!(ctx
        .check_filesystem_read(Path::new("/data/file.txt"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(Path::new("/config/app.toml"))
        .is_ok());
    assert!(ctx
        .check_filesystem_read(Path::new("/other/file.txt"))
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
