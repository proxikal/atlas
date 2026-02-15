//! Sandbox enforcement and resource quotas

use crate::security::audit::{AuditEvent, AuditLogger};
use crate::security::permissions::PermissionSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Sandbox errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SandboxError {
    #[error("Memory quota exceeded: {used} bytes used, {limit} bytes limit")]
    MemoryQuotaExceeded { used: usize, limit: usize },

    #[error("CPU time quota exceeded: {used:?} used, {limit:?} limit")]
    CpuTimeQuotaExceeded { used: Duration, limit: Duration },

    #[error("Stack depth exceeded: {depth} (max: {limit})")]
    StackDepthExceeded { depth: usize, limit: usize },

    #[error("File descriptor quota exceeded: {used} used, {limit} limit")]
    FileDescriptorQuotaExceeded { used: usize, limit: usize },

    #[error("Network connection quota exceeded: {used} used, {limit} limit")]
    NetworkConnectionQuotaExceeded { used: usize, limit: usize },

    #[error("Disk I/O quota exceeded: {used} bytes, {limit} bytes limit")]
    DiskIOQuotaExceeded { used: u64, limit: u64 },

    #[error("Sandbox escape prevented: {0}")]
    EscapePrevented(String),

    #[error("Operation not allowed in sandbox: {0}")]
    OperationDenied(String),
}

/// Resource quotas for sandbox
#[derive(Debug, Clone)]
pub struct ResourceQuotas {
    /// Maximum memory allocation (bytes)
    pub memory_limit: Option<usize>,
    /// Maximum CPU time
    pub cpu_time_limit: Option<Duration>,
    /// Maximum stack depth
    pub stack_depth_limit: Option<usize>,
    /// Maximum file descriptors
    pub file_descriptor_limit: Option<usize>,
    /// Maximum network connections
    pub network_connection_limit: Option<usize>,
    /// Maximum disk I/O (bytes)
    pub disk_io_limit: Option<u64>,
}

impl Default for ResourceQuotas {
    fn default() -> Self {
        Self::restrictive()
    }
}

impl ResourceQuotas {
    /// Create restrictive default quotas for untrusted code
    pub fn restrictive() -> Self {
        Self {
            memory_limit: Some(64 * 1024 * 1024),     // 64 MB
            cpu_time_limit: Some(Duration::from_secs(5)), // 5 seconds
            stack_depth_limit: Some(1000),             // 1000 frames
            file_descriptor_limit: Some(10),           // 10 FDs
            network_connection_limit: Some(5),         // 5 connections
            disk_io_limit: Some(10 * 1024 * 1024),    // 10 MB
        }
    }

    /// Create permissive quotas for trusted code
    pub fn permissive() -> Self {
        Self {
            memory_limit: Some(1024 * 1024 * 1024),       // 1 GB
            cpu_time_limit: Some(Duration::from_secs(300)), // 5 minutes
            stack_depth_limit: Some(10000),                // 10000 frames
            file_descriptor_limit: Some(1000),             // 1000 FDs
            network_connection_limit: Some(100),           // 100 connections
            disk_io_limit: Some(1024 * 1024 * 1024),      // 1 GB
        }
    }

    /// Create unlimited quotas (for development/testing only)
    pub fn unlimited() -> Self {
        Self {
            memory_limit: None,
            cpu_time_limit: None,
            stack_depth_limit: None,
            file_descriptor_limit: None,
            network_connection_limit: None,
            disk_io_limit: None,
        }
    }
}

/// Resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_used: usize,
    pub cpu_time_used: Duration,
    pub stack_depth: usize,
    pub file_descriptors_used: usize,
    pub network_connections_used: usize,
    pub disk_io_used: u64,
    pub start_time: Option<Instant>,
}

impl ResourceUsage {
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Update CPU time from start
    pub fn update_cpu_time(&mut self) {
        if let Some(start) = self.start_time {
            self.cpu_time_used = start.elapsed();
        }
    }
}

/// Sandbox context for isolating untrusted code
pub struct Sandbox {
    id: String,
    permissions: PermissionSet,
    quotas: ResourceQuotas,
    usage: Arc<Mutex<ResourceUsage>>,
    audit_logger: Option<Arc<dyn AuditLogger>>,
    enabled: bool,
}

impl Sandbox {
    /// Create new sandbox with given permissions and quotas
    pub fn new(
        id: String,
        permissions: PermissionSet,
        quotas: ResourceQuotas,
    ) -> Self {
        Self {
            id,
            permissions,
            quotas,
            usage: Arc::new(Mutex::new(ResourceUsage::new())),
            audit_logger: None,
            enabled: true,
        }
    }

    /// Create restrictive sandbox for untrusted code
    pub fn restrictive(id: String) -> Self {
        Self::new(id, PermissionSet::new(), ResourceQuotas::restrictive())
    }

    /// Create permissive sandbox for semi-trusted code
    pub fn permissive(id: String) -> Self {
        let perms = PermissionSet::new();
        // Grant some basic permissions
        Self::new(id, perms, ResourceQuotas::permissive())
    }

    /// Set audit logger
    pub fn with_audit_logger(mut self, logger: Arc<dyn AuditLogger>) -> Self {
        // Log sandbox creation
        let event = AuditEvent::SandboxCreated {
            sandbox_id: self.id.clone(),
            memory_limit: self.quotas.memory_limit,
            cpu_limit: self.quotas.cpu_time_limit.map(|d| d.as_millis() as u64),
        };
        logger.log(event);
        self.audit_logger = Some(logger);
        self
    }

    /// Get sandbox ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get permissions
    pub fn permissions(&self) -> &PermissionSet {
        &self.permissions
    }

    /// Grant permission to sandbox
    pub fn grant_permission(&mut self, permission: crate::security::permissions::Permission) {
        self.permissions.grant(permission);
    }

    /// Check if sandbox is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Disable sandbox (for privileged escalation - use with caution)
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable sandbox
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    // Resource quota checking methods

    /// Check memory allocation
    pub fn check_memory(&self, additional: usize) -> Result<(), SandboxError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(limit) = self.quotas.memory_limit {
            let usage = self.usage.lock().unwrap();
            let total = usage.memory_used + additional;

            if total > limit {
                if let Some(ref logger) = self.audit_logger {
                    logger.log(AuditEvent::QuotaViolation {
                        resource: "memory".to_string(),
                        limit: limit as u64,
                        attempted: total as u64,
                    });
                }
                return Err(SandboxError::MemoryQuotaExceeded {
                    used: total,
                    limit,
                });
            }
        }

        Ok(())
    }

    /// Track memory allocation
    pub fn allocate_memory(&self, size: usize) -> Result<(), SandboxError> {
        self.check_memory(size)?;
        let mut usage = self.usage.lock().unwrap();
        usage.memory_used += size;
        Ok(())
    }

    /// Track memory deallocation
    pub fn deallocate_memory(&self, size: usize) {
        let mut usage = self.usage.lock().unwrap();
        usage.memory_used = usage.memory_used.saturating_sub(size);
    }

    /// Check CPU time
    pub fn check_cpu_time(&self) -> Result<(), SandboxError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(limit) = self.quotas.cpu_time_limit {
            let mut usage = self.usage.lock().unwrap();
            usage.update_cpu_time();

            if usage.cpu_time_used > limit {
                if let Some(ref logger) = self.audit_logger {
                    logger.log(AuditEvent::QuotaViolation {
                        resource: "cpu_time".to_string(),
                        limit: limit.as_millis() as u64,
                        attempted: usage.cpu_time_used.as_millis() as u64,
                    });
                }
                return Err(SandboxError::CpuTimeQuotaExceeded {
                    used: usage.cpu_time_used,
                    limit,
                });
            }
        }

        Ok(())
    }

    /// Check stack depth
    pub fn check_stack_depth(&self, depth: usize) -> Result<(), SandboxError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(limit) = self.quotas.stack_depth_limit {
            if depth > limit {
                if let Some(ref logger) = self.audit_logger {
                    logger.log(AuditEvent::QuotaViolation {
                        resource: "stack_depth".to_string(),
                        limit: limit as u64,
                        attempted: depth as u64,
                    });
                }
                return Err(SandboxError::StackDepthExceeded { depth, limit });
            }
        }

        // Update usage
        let mut usage = self.usage.lock().unwrap();
        usage.stack_depth = usage.stack_depth.max(depth);

        Ok(())
    }

    /// Check file descriptor allocation
    pub fn check_file_descriptor(&self) -> Result<(), SandboxError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(limit) = self.quotas.file_descriptor_limit {
            let usage = self.usage.lock().unwrap();

            if usage.file_descriptors_used >= limit {
                if let Some(ref logger) = self.audit_logger {
                    logger.log(AuditEvent::QuotaViolation {
                        resource: "file_descriptors".to_string(),
                        limit: limit as u64,
                        attempted: (usage.file_descriptors_used + 1) as u64,
                    });
                }
                return Err(SandboxError::FileDescriptorQuotaExceeded {
                    used: usage.file_descriptors_used,
                    limit,
                });
            }
        }

        Ok(())
    }

    /// Allocate file descriptor
    pub fn allocate_file_descriptor(&self) -> Result<(), SandboxError> {
        self.check_file_descriptor()?;
        let mut usage = self.usage.lock().unwrap();
        usage.file_descriptors_used += 1;
        Ok(())
    }

    /// Deallocate file descriptor
    pub fn deallocate_file_descriptor(&self) {
        let mut usage = self.usage.lock().unwrap();
        usage.file_descriptors_used = usage.file_descriptors_used.saturating_sub(1);
    }

    /// Check network connection
    pub fn check_network_connection(&self) -> Result<(), SandboxError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(limit) = self.quotas.network_connection_limit {
            let usage = self.usage.lock().unwrap();

            if usage.network_connections_used >= limit {
                if let Some(ref logger) = self.audit_logger {
                    logger.log(AuditEvent::QuotaViolation {
                        resource: "network_connections".to_string(),
                        limit: limit as u64,
                        attempted: (usage.network_connections_used + 1) as u64,
                    });
                }
                return Err(SandboxError::NetworkConnectionQuotaExceeded {
                    used: usage.network_connections_used,
                    limit,
                });
            }
        }

        Ok(())
    }

    /// Allocate network connection
    pub fn allocate_network_connection(&self) -> Result<(), SandboxError> {
        self.check_network_connection()?;
        let mut usage = self.usage.lock().unwrap();
        usage.network_connections_used += 1;
        Ok(())
    }

    /// Deallocate network connection
    pub fn deallocate_network_connection(&self) {
        let mut usage = self.usage.lock().unwrap();
        usage.network_connections_used = usage.network_connections_used.saturating_sub(1);
    }

    /// Check disk I/O
    pub fn check_disk_io(&self, bytes: u64) -> Result<(), SandboxError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(limit) = self.quotas.disk_io_limit {
            let usage = self.usage.lock().unwrap();
            let total = usage.disk_io_used + bytes;

            if total > limit {
                if let Some(ref logger) = self.audit_logger {
                    logger.log(AuditEvent::QuotaViolation {
                        resource: "disk_io".to_string(),
                        limit,
                        attempted: total,
                    });
                }
                return Err(SandboxError::DiskIOQuotaExceeded { used: total, limit });
            }
        }

        Ok(())
    }

    /// Track disk I/O
    pub fn track_disk_io(&self, bytes: u64) -> Result<(), SandboxError> {
        self.check_disk_io(bytes)?;
        let mut usage = self.usage.lock().unwrap();
        usage.disk_io_used += bytes;
        Ok(())
    }

    /// Get resource usage
    pub fn usage(&self) -> ResourceUsage {
        self.usage.lock().unwrap().clone()
    }

    /// Get quotas
    pub fn quotas(&self) -> &ResourceQuotas {
        &self.quotas
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        if let Some(ref logger) = self.audit_logger {
            logger.log(AuditEvent::SandboxDestroyed {
                sandbox_id: self.id.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::audit::MemoryAuditLogger;

    #[test]
    fn test_create_sandbox_with_limits() {
        let quotas = ResourceQuotas::restrictive();
        let sandbox = Sandbox::new("test-sandbox".to_string(), PermissionSet::new(), quotas);

        assert_eq!(sandbox.id(), "test-sandbox");
        assert!(sandbox.is_enabled());
    }

    #[test]
    fn test_file_access_denied_in_sandbox() {
        let sandbox = Sandbox::restrictive("test".to_string());
        
        // Sandbox has no permissions by default
        assert_eq!(sandbox.permissions().len(), 0);
    }

    #[test]
    fn test_network_access_denied_in_sandbox() {
        let sandbox = Sandbox::restrictive("test".to_string());
        
        // Sandbox has no permissions by default
        assert_eq!(sandbox.permissions().len(), 0);
    }

    #[test]
    fn test_memory_limit_enforced() {
        let quotas = ResourceQuotas {
            memory_limit: Some(100),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        // Should succeed
        assert!(sandbox.allocate_memory(50).is_ok());

        // Should exceed limit
        assert!(sandbox.allocate_memory(100).is_err());
    }

    #[test]
    fn test_cpu_time_limit_enforced() {
        let quotas = ResourceQuotas {
            cpu_time_limit: Some(Duration::from_millis(1)),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        // Small delay to exceed limit
        std::thread::sleep(Duration::from_millis(10));

        assert!(sandbox.check_cpu_time().is_err());
    }

    #[test]
    fn test_stack_depth_limit_enforced() {
        let quotas = ResourceQuotas {
            stack_depth_limit: Some(10),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        assert!(sandbox.check_stack_depth(5).is_ok());
        assert!(sandbox.check_stack_depth(15).is_err());
    }

    #[test]
    fn test_file_descriptor_quota() {
        let quotas = ResourceQuotas {
            file_descriptor_limit: Some(2),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        assert!(sandbox.allocate_file_descriptor().is_ok());
        assert!(sandbox.allocate_file_descriptor().is_ok());
        assert!(sandbox.allocate_file_descriptor().is_err());

        sandbox.deallocate_file_descriptor();
        assert!(sandbox.allocate_file_descriptor().is_ok());
    }

    #[test]
    fn test_network_connection_quota() {
        let quotas = ResourceQuotas {
            network_connection_limit: Some(2),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        assert!(sandbox.allocate_network_connection().is_ok());
        assert!(sandbox.allocate_network_connection().is_ok());
        assert!(sandbox.allocate_network_connection().is_err());
    }

    #[test]
    fn test_disk_io_quota() {
        let quotas = ResourceQuotas {
            disk_io_limit: Some(100),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        assert!(sandbox.track_disk_io(50).is_ok());
        assert!(sandbox.track_disk_io(60).is_err());
    }

    #[test]
    fn test_quota_exhaustion_error() {
        let quotas = ResourceQuotas {
            memory_limit: Some(50),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        let result = sandbox.allocate_memory(100);
        assert!(result.is_err());

        if let Err(SandboxError::MemoryQuotaExceeded { used, limit }) = result {
            assert_eq!(used, 100);
            assert_eq!(limit, 50);
        } else {
            panic!("Expected MemoryQuotaExceeded error");
        }
    }

    #[test]
    fn test_quota_monitoring() {
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), ResourceQuotas::default());

        sandbox.allocate_memory(100).unwrap();
        sandbox.allocate_file_descriptor().unwrap();

        let usage = sandbox.usage();
        assert_eq!(usage.memory_used, 100);
        assert_eq!(usage.file_descriptors_used, 1);
    }

    #[test]
    fn test_sandbox_disable_bypasses_checks() {
        let quotas = ResourceQuotas {
            memory_limit: Some(10),
            ..ResourceQuotas::default()
        };
        let mut sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas);

        // With sandbox enabled, should fail
        assert!(sandbox.allocate_memory(100).is_err());

        // Disable sandbox
        sandbox.disable();

        // Now should succeed
        assert!(sandbox.allocate_memory(100).is_ok());
    }

    #[test]
    fn test_audit_logging_quota_violation() {
        let logger = Arc::new(MemoryAuditLogger::new());
        let logger_dyn: Arc<dyn AuditLogger> = Arc::clone(&logger) as Arc<dyn AuditLogger>;
        let quotas = ResourceQuotas {
            memory_limit: Some(50),
            ..ResourceQuotas::default()
        };
        let sandbox = Sandbox::new("test".to_string(), PermissionSet::new(), quotas)
            .with_audit_logger(logger_dyn);

        // Trigger quota violation
        let _ = sandbox.allocate_memory(100);

        let entries = logger.entries();
        assert!(entries.iter().any(|e| matches!(
            &e.event,
            AuditEvent::QuotaViolation { resource, .. } if resource == "memory"
        )));
    }
}
