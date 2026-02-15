//! Permission types and security context
//!
//! Defines the permission system for controlling I/O operations.

use crate::security::audit::{AuditEvent, AuditLogger, NullAuditLogger};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

/// Security errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SecurityError {
    #[error("Permission denied: filesystem read access to {path}")]
    FilesystemReadDenied { path: PathBuf },

    #[error("Permission denied: filesystem write access to {path}")]
    FilesystemWriteDenied { path: PathBuf },

    #[error("Permission denied: network access to {host}")]
    NetworkDenied { host: String },

    #[error("Permission denied: process execution of {command}")]
    ProcessDenied { command: String },

    #[error("Permission denied: environment variable {var}")]
    EnvironmentDenied { var: String },

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid permission pattern: {0}")]
    InvalidPattern(String),
}

/// Permission types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Filesystem read access
    FilesystemRead { path: PathBuf, recursive: bool },

    /// Filesystem write access
    FilesystemWrite { path: PathBuf, recursive: bool },

    /// Network access
    Network { host: String },

    /// Process execution
    Process { command: String },

    /// Environment variable access
    Environment { var: String },
}

impl Permission {
    /// Check if this permission allows the requested operation
    pub fn allows(&self, requested: &Permission) -> bool {
        match (self, requested) {
            // Filesystem read: check path
            (
                Permission::FilesystemRead {
                    path: allowed_path,
                    recursive: allowed_recursive,
                },
                Permission::FilesystemRead {
                    path: requested_path,
                    ..
                },
            ) => {
                // Exact match
                if allowed_path == requested_path {
                    return true;
                }

                // If recursive, check if requested is under allowed
                if *allowed_recursive {
                    return requested_path.starts_with(allowed_path);
                }

                false
            }

            // Filesystem write: check path
            (
                Permission::FilesystemWrite {
                    path: allowed_path,
                    recursive: allowed_recursive,
                },
                Permission::FilesystemWrite {
                    path: requested_path,
                    ..
                },
            ) => {
                // Exact match
                if allowed_path == requested_path {
                    return true;
                }

                // If recursive, check if requested is under allowed
                if *allowed_recursive {
                    return requested_path.starts_with(allowed_path);
                }

                false
            }

            // Network: check host (exact match or wildcard)
            (Permission::Network { host: allowed }, Permission::Network { host: requested }) => {
                // Exact match
                if allowed == requested {
                    return true;
                }

                // Wildcard: *.example.com matches api.example.com
                if let Some(domain) = allowed.strip_prefix("*.") {
                    if requested.ends_with(domain) {
                        // Check it's a proper subdomain match
                        if requested == domain {
                            return true;
                        }
                        if let Some(prefix) = requested.strip_suffix(domain) {
                            return prefix.ends_with('.');
                        }
                    }
                }

                // Wildcard: allow all
                allowed == "*"
            }

            // Process: exact command name match
            (
                Permission::Process {
                    command: allowed_cmd,
                },
                Permission::Process {
                    command: requested_cmd,
                },
            ) => allowed_cmd == requested_cmd || allowed_cmd == "*",

            // Environment: exact variable name match
            (
                Permission::Environment { var: allowed_var },
                Permission::Environment { var: requested_var },
            ) => allowed_var == requested_var || allowed_var == "*",

            // Different permission types never match
            _ => false,
        }
    }
}

/// Set of permissions
#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    /// Grant a permission
    pub fn grant(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Check if a permission is granted
    pub fn is_granted(&self, requested: &Permission) -> bool {
        self.permissions.iter().any(|p| p.allows(requested))
    }

    /// Get all permissions
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }

    /// Get number of permissions
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Check if permission set is empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Merge permissions from another set
    pub fn merge(&mut self, other: &PermissionSet) {
        for perm in &other.permissions {
            self.permissions.insert(perm.clone());
        }
    }
}

/// Security context managing permissions
#[derive(Clone)]
pub struct SecurityContext {
    filesystem_read: PermissionSet,
    filesystem_write: PermissionSet,
    network: PermissionSet,
    process: PermissionSet,
    environment: PermissionSet,
    audit_logger: Arc<dyn AuditLogger>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            filesystem_read: PermissionSet::new(),
            filesystem_write: PermissionSet::new(),
            network: PermissionSet::new(),
            process: PermissionSet::new(),
            environment: PermissionSet::new(),
            audit_logger: Arc::new(NullAuditLogger::new()),
        }
    }
}

impl SecurityContext {
    /// Create a new security context with default (deny all) permissions
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new security context with audit logging enabled
    pub fn with_audit_logger(logger: Arc<dyn AuditLogger>) -> Self {
        Self {
            filesystem_read: PermissionSet::new(),
            filesystem_write: PermissionSet::new(),
            network: PermissionSet::new(),
            process: PermissionSet::new(),
            environment: PermissionSet::new(),
            audit_logger: logger,
        }
    }

    /// Create from security configuration
    ///
    /// Note: This requires atlas-config to be available. For now, this is a placeholder
    /// that will be implemented when integrating with the runtime.
    #[allow(dead_code)] // TODO: Implement full config integration (future phase)
    pub fn from_config(_config: &atlas_config::SecurityConfig) -> Self {
        // TODO: Implement full config integration
        Self::new()
    }

    /// Create a permissive context that allows all operations
    ///
    /// **WARNING**: Only use for development/testing. Not secure!
    pub fn allow_all() -> Self {
        let mut ctx = Self::new();

        // Grant wildcard permissions for everything
        ctx.filesystem_read.grant(Permission::FilesystemRead {
            path: PathBuf::from("/"),
            recursive: true,
        });
        ctx.filesystem_write.grant(Permission::FilesystemWrite {
            path: PathBuf::from("/"),
            recursive: true,
        });
        ctx.network.grant(Permission::Network {
            host: "*".to_string(),
        });
        ctx.process.grant(Permission::Process {
            command: "*".to_string(),
        });
        ctx.environment.grant(Permission::Environment {
            var: "*".to_string(),
        });

        ctx
    }

    // Permission granting methods

    /// Grant filesystem read permission
    pub fn grant_filesystem_read(&mut self, path: &Path, recursive: bool) {
        let path = canonicalize_path_safe(path);
        self.filesystem_read
            .grant(Permission::FilesystemRead { path, recursive });
    }

    /// Grant filesystem write permission
    pub fn grant_filesystem_write(&mut self, path: &Path, recursive: bool) {
        let path = canonicalize_path_safe(path);
        self.filesystem_write
            .grant(Permission::FilesystemWrite { path, recursive });
    }

    /// Grant network permission
    pub fn grant_network(&mut self, host: impl Into<String>) {
        self.network
            .grant(Permission::Network { host: host.into() });
    }

    /// Grant process execution permission
    pub fn grant_process(&mut self, command: impl Into<String>) {
        self.process.grant(Permission::Process {
            command: command.into(),
        });
    }

    /// Grant environment variable access permission
    pub fn grant_environment(&mut self, var: impl Into<String>) {
        self.environment
            .grant(Permission::Environment { var: var.into() });
    }

    // Permission checking methods

    /// Check filesystem read permission
    pub fn check_filesystem_read(&self, path: &Path) -> Result<(), SecurityError> {
        let path = canonicalize_path_safe(path);
        let requested = Permission::FilesystemRead {
            path: path.clone(),
            recursive: false,
        };

        if self.filesystem_read.is_granted(&requested) {
            self.audit_logger.log(AuditEvent::PermissionCheck {
                operation: "file read".to_string(),
                target: path.display().to_string(),
                granted: true,
            });
            Ok(())
        } else {
            self.audit_logger
                .log(AuditEvent::FilesystemReadDenied { path: path.clone() });
            Err(SecurityError::FilesystemReadDenied { path })
        }
    }

    /// Check filesystem write permission
    pub fn check_filesystem_write(&self, path: &Path) -> Result<(), SecurityError> {
        let path = canonicalize_path_safe(path);
        let requested = Permission::FilesystemWrite {
            path: path.clone(),
            recursive: false,
        };

        if self.filesystem_write.is_granted(&requested) {
            self.audit_logger.log(AuditEvent::PermissionCheck {
                operation: "file write".to_string(),
                target: path.display().to_string(),
                granted: true,
            });
            Ok(())
        } else {
            self.audit_logger
                .log(AuditEvent::FilesystemWriteDenied { path: path.clone() });
            Err(SecurityError::FilesystemWriteDenied { path })
        }
    }

    /// Check network permission
    pub fn check_network(&self, host: &str) -> Result<(), SecurityError> {
        let requested = Permission::Network {
            host: host.to_string(),
        };

        if self.network.is_granted(&requested) {
            self.audit_logger.log(AuditEvent::PermissionCheck {
                operation: "network".to_string(),
                target: host.to_string(),
                granted: true,
            });
            Ok(())
        } else {
            self.audit_logger.log(AuditEvent::NetworkDenied {
                host: host.to_string(),
            });
            Err(SecurityError::NetworkDenied {
                host: host.to_string(),
            })
        }
    }

    /// Check process execution permission
    pub fn check_process(&self, command: &str) -> Result<(), SecurityError> {
        let requested = Permission::Process {
            command: command.to_string(),
        };

        if self.process.is_granted(&requested) {
            self.audit_logger.log(AuditEvent::PermissionCheck {
                operation: "process".to_string(),
                target: command.to_string(),
                granted: true,
            });
            Ok(())
        } else {
            self.audit_logger.log(AuditEvent::ProcessDenied {
                command: command.to_string(),
            });
            Err(SecurityError::ProcessDenied {
                command: command.to_string(),
            })
        }
    }

    /// Check environment variable access permission
    pub fn check_environment(&self, var: &str) -> Result<(), SecurityError> {
        let requested = Permission::Environment {
            var: var.to_string(),
        };

        if self.environment.is_granted(&requested) {
            self.audit_logger.log(AuditEvent::PermissionCheck {
                operation: "environment".to_string(),
                target: var.to_string(),
                granted: true,
            });
            Ok(())
        } else {
            self.audit_logger.log(AuditEvent::EnvironmentDenied {
                var: var.to_string(),
            });
            Err(SecurityError::EnvironmentDenied {
                var: var.to_string(),
            })
        }
    }

    /// Get the audit logger (for testing)
    pub fn audit_logger(&self) -> Arc<dyn AuditLogger> {
        Arc::clone(&self.audit_logger)
    }
}

/// Safely canonicalize a path
///
/// If canonicalization fails (path doesn't exist), returns the absolute path
/// without resolving symlinks. This allows permission checks on paths that
/// don't exist yet (for write operations).
fn canonicalize_path_safe(path: &Path) -> PathBuf {
    // Try to canonicalize (resolve symlinks, .., etc.)
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }

    // If that fails, make it absolute at least
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_filesystem_read_exact() {
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
    fn test_permission_filesystem_read_recursive() {
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
    fn test_permission_filesystem_read_not_recursive() {
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
    fn test_permission_network_exact() {
        let allowed = Permission::Network {
            host: "api.example.com".to_string(),
        };
        let requested = Permission::Network {
            host: "api.example.com".to_string(),
        };

        assert!(allowed.allows(&requested));
    }

    #[test]
    fn test_permission_network_wildcard() {
        let allowed = Permission::Network {
            host: "*.example.com".to_string(),
        };
        let requested = Permission::Network {
            host: "api.example.com".to_string(),
        };

        assert!(allowed.allows(&requested));
    }

    #[test]
    fn test_permission_network_wildcard_no_match() {
        let allowed = Permission::Network {
            host: "*.example.com".to_string(),
        };
        let requested = Permission::Network {
            host: "other.com".to_string(),
        };

        assert!(!allowed.allows(&requested));
    }

    #[test]
    fn test_permission_process_exact() {
        let allowed = Permission::Process {
            command: "git".to_string(),
        };
        let requested = Permission::Process {
            command: "git".to_string(),
        };

        assert!(allowed.allows(&requested));
    }

    #[test]
    fn test_permission_process_wildcard() {
        let allowed = Permission::Process {
            command: "*".to_string(),
        };
        let requested = Permission::Process {
            command: "git".to_string(),
        };

        assert!(allowed.allows(&requested));
    }

    #[test]
    fn test_security_context_default_denies() {
        let ctx = SecurityContext::new();

        assert!(ctx
            .check_filesystem_read(Path::new("/data/file.txt"))
            .is_err());
        assert!(ctx
            .check_filesystem_write(Path::new("/data/file.txt"))
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
            .check_filesystem_read(Path::new("/other/file.txt"))
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
    fn test_security_context_allow_all() {
        let ctx = SecurityContext::allow_all();

        assert!(ctx.check_filesystem_read(Path::new("/any/path")).is_ok());
        assert!(ctx.check_filesystem_write(Path::new("/any/path")).is_ok());
        assert!(ctx.check_network("any.host.com").is_ok());
        assert!(ctx.check_process("any-command").is_ok());
        assert!(ctx.check_environment("ANY_VAR").is_ok());
    }

    #[test]
    fn test_permission_set_multiple_grants() {
        let mut set = PermissionSet::new();
        set.grant(Permission::FilesystemRead {
            path: PathBuf::from("/data"),
            recursive: true,
        });
        set.grant(Permission::FilesystemRead {
            path: PathBuf::from("/config"),
            recursive: false,
        });

        // Check /data/file.txt (allowed via /data recursive)
        assert!(set.is_granted(&Permission::FilesystemRead {
            path: PathBuf::from("/data/file.txt"),
            recursive: false,
        }));

        // Check /config (allowed via exact match)
        assert!(set.is_granted(&Permission::FilesystemRead {
            path: PathBuf::from("/config"),
            recursive: false,
        }));

        // Check /other (not allowed)
        assert!(!set.is_granted(&Permission::FilesystemRead {
            path: PathBuf::from("/other"),
            recursive: false,
        }));
    }
}
