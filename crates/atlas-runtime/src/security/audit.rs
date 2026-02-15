//! Security audit logging
//!
//! Provides structured logging of all security events (permission checks,
//! grants, denials) for security monitoring and compliance.

use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Security audit event types
#[derive(Debug, Clone, PartialEq)]
pub enum AuditEvent {
    /// Permission check requested
    PermissionCheck {
        operation: String,
        target: String,
        granted: bool,
    },
    /// Filesystem read permission denied
    FilesystemReadDenied { path: PathBuf },
    /// Filesystem write permission denied
    FilesystemWriteDenied { path: PathBuf },
    /// Network access denied
    NetworkDenied { host: String },
    /// Process execution denied
    ProcessDenied { command: String },
    /// Environment variable access denied
    EnvironmentDenied { var: String },
    /// Sandbox created
    SandboxCreated {
        sandbox_id: String,
        memory_limit: Option<usize>,
        cpu_limit: Option<u64>,
    },
    /// Sandbox destroyed
    SandboxDestroyed { sandbox_id: String },
    /// Security policy violation
    PolicyViolation {
        policy: String,
        violation: String,
    },
    /// Resource quota exceeded
    QuotaViolation {
        resource: String,
        limit: u64,
        attempted: u64,
    },
    /// Privilege escalation attempt
    PrivilegeEscalation { context: String },
    /// Capability granted
    CapabilityGranted {
        capability_id: String,
        permissions: String,
    },
    /// Capability revoked
    CapabilityRevoked { capability_id: String },
}

impl fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditEvent::PermissionCheck {
                operation,
                target,
                granted,
            } => {
                let status = if *granted { "GRANTED" } else { "DENIED" };
                write!(
                    f,
                    "Permission {}: {} access to {}",
                    status, operation, target
                )
            }
            AuditEvent::FilesystemReadDenied { path } => {
                write!(
                    f,
                    "Permission denied: file read access to {}",
                    path.display()
                )
            }
            AuditEvent::FilesystemWriteDenied { path } => {
                write!(
                    f,
                    "Permission denied: file write access to {}",
                    path.display()
                )
            }
            AuditEvent::NetworkDenied { host } => {
                write!(f, "Permission denied: network access to {}", host)
            }
            AuditEvent::ProcessDenied { command } => {
                write!(f, "Permission denied: process execution of {}", command)
            }
            AuditEvent::EnvironmentDenied { var } => {
                write!(f, "Permission denied: environment variable {}", var)
            }
            AuditEvent::SandboxCreated {
                sandbox_id,
                memory_limit,
                cpu_limit,
            } => {
                write!(
                    f,
                    "Sandbox created: {} (memory: {:?}, cpu: {:?})",
                    sandbox_id, memory_limit, cpu_limit
                )
            }
            AuditEvent::SandboxDestroyed { sandbox_id } => {
                write!(f, "Sandbox destroyed: {}", sandbox_id)
            }
            AuditEvent::PolicyViolation { policy, violation } => {
                write!(f, "Policy violation: {} - {}", policy, violation)
            }
            AuditEvent::QuotaViolation {
                resource,
                limit,
                attempted,
            } => {
                write!(
                    f,
                    "Quota violation: {} (limit: {}, attempted: {})",
                    resource, limit, attempted
                )
            }
            AuditEvent::PrivilegeEscalation { context } => {
                write!(f, "Privilege escalation attempt: {}", context)
            }
            AuditEvent::CapabilityGranted {
                capability_id,
                permissions,
            } => {
                write!(
                    f,
                    "Capability granted: {} (permissions: {})",
                    capability_id, permissions
                )
            }
            AuditEvent::CapabilityRevoked { capability_id } => {
                write!(f, "Capability revoked: {}", capability_id)
            }
        }
    }
}

/// Audit log entry with timestamp
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Event timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,
    /// Audit event
    pub event: AuditEvent,
}

impl AuditEntry {
    /// Create a new audit entry with current timestamp
    pub fn new(event: AuditEvent) -> Self {
        Self {
            timestamp: current_timestamp_ms(),
            event,
        }
    }

    /// Format as log line
    pub fn to_log_line(&self) -> String {
        format!("[{}] {}", format_timestamp(self.timestamp), self.event)
    }
}

/// Get current Unix timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_millis() as u64
}

/// Format timestamp as ISO 8601 datetime
fn format_timestamp(timestamp_ms: u64) -> String {
    // Simple formatting: convert ms to seconds since epoch
    // For production, would use chrono or time crate for proper formatting
    let seconds = timestamp_ms / 1000;
    let millis = timestamp_ms % 1000;

    // Basic UTC datetime formatting (simplified)
    // In production, use proper datetime library
    format!("{}+{:03}ms", seconds, millis)
}

/// Audit logger trait for customizable logging backends
pub trait AuditLogger: Send + Sync {
    /// Log an audit event
    fn log(&self, event: AuditEvent);

    /// Get all logged entries (for testing)
    fn entries(&self) -> Vec<AuditEntry>;

    /// Clear all logged entries (for testing)
    fn clear(&self);
}

/// In-memory audit logger (default implementation)
#[derive(Debug, Clone, Default)]
pub struct MemoryAuditLogger {
    entries: Arc<Mutex<Vec<AuditEntry>>>,
}

impl MemoryAuditLogger {
    /// Create a new in-memory audit logger
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl AuditLogger for MemoryAuditLogger {
    fn log(&self, event: AuditEvent) {
        let entry = AuditEntry::new(event);
        self.entries.lock().unwrap().push(entry);
    }

    fn entries(&self) -> Vec<AuditEntry> {
        self.entries.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

/// Null audit logger (no-op, for performance)
#[derive(Debug, Clone, Copy, Default)]
pub struct NullAuditLogger;

impl NullAuditLogger {
    /// Create a new null audit logger
    pub fn new() -> Self {
        Self
    }
}

impl AuditLogger for NullAuditLogger {
    fn log(&self, _event: AuditEvent) {
        // No-op
    }

    fn entries(&self) -> Vec<AuditEntry> {
        Vec::new()
    }

    fn clear(&self) {
        // No-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_display() {
        let event = AuditEvent::FilesystemReadDenied {
            path: PathBuf::from("/etc/passwd"),
        };
        assert_eq!(
            event.to_string(),
            "Permission denied: file read access to /etc/passwd"
        );
    }

    #[test]
    fn test_audit_entry_creation() {
        let event = AuditEvent::NetworkDenied {
            host: "evil.com".to_string(),
        };
        let entry = AuditEntry::new(event.clone());

        assert!(entry.timestamp > 0);
        assert_eq!(entry.event, event);
    }

    #[test]
    fn test_audit_entry_log_line_format() {
        let event = AuditEvent::ProcessDenied {
            command: "rm".to_string(),
        };
        let entry = AuditEntry::new(event);
        let log_line = entry.to_log_line();

        assert!(log_line.contains("Permission denied: process execution of rm"));
        assert!(log_line.starts_with('[')); // Has timestamp
    }

    #[test]
    fn test_memory_logger_stores_events() {
        let logger = MemoryAuditLogger::new();

        logger.log(AuditEvent::FilesystemReadDenied {
            path: PathBuf::from("/data/file.txt"),
        });
        logger.log(AuditEvent::NetworkDenied {
            host: "api.example.com".to_string(),
        });

        let entries = logger.entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_memory_logger_clear() {
        let logger = MemoryAuditLogger::new();

        logger.log(AuditEvent::FilesystemReadDenied {
            path: PathBuf::from("/file.txt"),
        });

        assert_eq!(logger.entries().len(), 1);

        logger.clear();
        assert_eq!(logger.entries().len(), 0);
    }

    #[test]
    fn test_null_logger_no_op() {
        let logger = NullAuditLogger::new();

        logger.log(AuditEvent::FilesystemReadDenied {
            path: PathBuf::from("/file.txt"),
        });

        assert_eq!(logger.entries().len(), 0);
    }

    #[test]
    fn test_permission_check_event_granted() {
        let event = AuditEvent::PermissionCheck {
            operation: "file read".to_string(),
            target: "/data/file.txt".to_string(),
            granted: true,
        };

        assert!(event.to_string().contains("GRANTED"));
        assert!(event.to_string().contains("file read"));
    }

    #[test]
    fn test_permission_check_event_denied() {
        let event = AuditEvent::PermissionCheck {
            operation: "file write".to_string(),
            target: "/etc/passwd".to_string(),
            granted: false,
        };

        assert!(event.to_string().contains("DENIED"));
        assert!(event.to_string().contains("file write"));
    }

    #[test]
    fn test_timestamp_is_monotonic() {
        let entry1 = AuditEntry::new(AuditEvent::NetworkDenied {
            host: "test1.com".to_string(),
        });

        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));

        let entry2 = AuditEntry::new(AuditEvent::NetworkDenied {
            host: "test2.com".to_string(),
        });

        assert!(entry2.timestamp >= entry1.timestamp);
    }
}
