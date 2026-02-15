//! Security and Permission System
//!
//! Implements Atlas's secure-by-default I/O model with capability-based permissions.
//!
//! # Overview
//!
//! All I/O operations (filesystem, network, process, environment) are denied by default
//! and require explicit permission grants. Permissions can be granted via:
//! - Configuration files (atlas.toml, global config)
//! - Environment variables
//! - CLI flags (highest priority)
//!
//! # Architecture
//!
//! - **Permission**: Types of capabilities that can be granted
//! - **SecurityContext**: Manages active permissions and enforces checks
//! - **Policy**: Rules for matching and allowing/denying operations
//!
//! # Example
//!
//! ```
//! use atlas_runtime::security::{SecurityContext, Permission};
//! use std::path::Path;
//!
//! let mut ctx = SecurityContext::new();
//! ctx.grant_filesystem_read(Path::new("/data"), true); // Allow /data and subdirs
//!
//! // Check permission before I/O
//! assert!(ctx.check_filesystem_read(Path::new("/data/file.txt")).is_ok());
//! assert!(ctx.check_filesystem_read(Path::new("/etc/passwd")).is_err());
//! ```

pub mod audit;
pub mod permissions;
pub mod policy;
pub mod sandbox;

pub use audit::{AuditEntry, AuditEvent, AuditLogger, MemoryAuditLogger, NullAuditLogger};
pub use permissions::{Permission, PermissionSet, SecurityContext, SecurityError};
pub use policy::{PolicyError, PolicyManager, SecurityPolicy};
pub use sandbox::{ResourceQuotas, ResourceUsage, Sandbox, SandboxError};
