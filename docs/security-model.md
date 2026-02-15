# Atlas Security Model

## Table of Contents

1. [Overview](#overview)
2. [Threat Model](#threat-model)
3. [Permission System](#permission-system)
4. [Capability-Based Security](#capability-based-security)
5. [Security Policies](#security-policies)
6. [Sandbox Enforcement](#sandbox-enforcement)
7. [Resource Quotas](#resource-quotas)
8. [Trust Boundaries](#trust-boundaries)
9. [Audit Logging](#audit-logging)
10. [Best Practices](#best-practices)
11. [API Reference](#api-reference)

## Overview

Atlas implements a comprehensive capability-based security model designed to safely execute untrusted code while protecting system resources. The security system provides:

- **Default-deny policy**: All operations denied unless explicitly permitted
- **Granular permissions**: File, network, process, FFI, environment, reflection access control
- **Capability-based access**: Unforgeable tokens grant permissions without ambient authority
- **Resource quotas**: Memory, CPU, I/O limits prevent resource exhaustion
- **Sandbox isolation**: Untrusted code runs in restricted environments
- **Security policies**: Declarative configuration for permission management
- **Audit logging**: Comprehensive event tracking for security monitoring

## Threat Model

### Threats Addressed

**Untrusted Code Execution**
- Malicious scripts attempting file system access
- Network attacks (data exfiltration, DDoS participation)
- Resource exhaustion (memory bombs, CPU loops)
- Privilege escalation attempts
- Information disclosure through reflection/introspection

**Resource Abuse**
- Memory exhaustion
- CPU time abuse
- Disk I/O flooding
- Network bandwidth consumption
- File descriptor leaks

**System Compromise**
- Arbitrary file read/write
- Unauthorized network connections
- Process spawning
- Environment variable manipulation
- FFI calls to dangerous functions

### Out of Scope

- Side-channel attacks (timing, cache)
- Hardware vulnerabilities
- Kernel exploits
- Physical access attacks
- Cryptographic weaknesses

## Permission System

### Permission Types

```rust
pub enum Permission {
    FilesystemRead { path: PathBuf, recursive: bool },
    FilesystemWrite { path: PathBuf, recursive: bool },
    Network { host: String },
    Process { command: String },
    Environment { var: String },
}
```

### Granting Permissions

```rust
use atlas_runtime::security::SecurityContext;
use std::path::Path;

let mut ctx = SecurityContext::new();

// File access
ctx.grant_filesystem_read(Path::new("/data"), true); // recursive
ctx.grant_filesystem_write(Path::new("/output"), false); // exact path

// Network access
ctx.grant_network("api.example.com"); // exact domain
ctx.grant_network("*.example.com");   // wildcard subdomain

// Process execution
ctx.grant_process("git");
ctx.grant_process("*"); // allow all (use with caution)

// Environment variables
ctx.grant_environment("PATH");
ctx.grant_environment("*"); // allow all (development only)
```

### Checking Permissions

```rust
// Permissions are checked automatically before operations
ctx.check_filesystem_read(Path::new("/data/file.txt"))?;
ctx.check_network("api.example.com")?;
ctx.check_process("git")?;
ctx.check_environment("HOME")?;
```

## Capability-Based Security

Capabilities are unforgeable tokens that grant specific permissions without relying on ambient authority.

### Creating Capabilities

Capabilities are managed internally by the security system. The `SecurityContext` acts as a capability manager, granting permissions that form capability tokens.

### Capability Properties

1. **Unforgeable**: Cannot be created or duplicated without authorization
2. **Transferable**: Can be passed explicitly to functions
3. **Attenuatable**: Can be reduced in scope but not expanded
4. **Revocable**: Can be invalidated dynamically
5. **No Ambient Authority**: All access requires explicit capability presentation

### Attenuation

```rust
// Reduce permissions by removing specific grants
// This would be done by creating a new SecurityContext
// with a subset of permissions
```

## Security Policies

Policies define security rules in TOML or JSON format.

### Policy Structure

```toml
# security-policy.toml
name = "application-policy"
description = "Policy for web application"
default_action = "deny"

[[allow]]
resource = "file-read"
pattern = "/app/data/*"
scope = "recursive"
description = "Allow reading application data"

[[allow]]
resource = "network-connect"
pattern = "*.example.com"
description = "Allow connections to example.com subdomains"

[[deny]]
resource = "file-write"
pattern = "/etc/*"
description = "Deny writes to system configuration"

[features]
default = { dependencies = [], default = true }
```

### Loading Policies

```rust
use atlas_runtime::security::{SecurityPolicy, PolicyManager};

// From TOML string
let policy = SecurityPolicy::from_toml(toml_content)?;

// Using policy manager
let mut manager = PolicyManager::new();
manager.load_policy(policy)?;

// Get permission set from policy
let permissions = manager.get_permissions("application-policy")?;
```

### Policy Inheritance

```toml
name = "derived-policy"
inherits = ["base-policy", "network-policy"]

[[allow]]
resource = "file-write"
pattern = "/tmp/*"
```

### Time-Based Permissions

```toml
[[time_based]]
permission = "network-connect:api.example.com"
start = 1640995200  # Unix timestamp
end = 1672531200
days_of_week = [1, 2, 3, 4, 5]  # Monday-Friday
hours_of_day = [9, 10, 11, 12, 13, 14, 15, 16, 17]  # Business hours
```

## Sandbox Enforcement

Sandboxes isolate untrusted code execution with strict resource limits.

### Creating Sandboxes

```rust
use atlas_runtime::security::{Sandbox, ResourceQuotas, PermissionSet};

// Restrictive sandbox for untrusted code
let sandbox = Sandbox::restrictive("untrusted-code".to_string());

// Custom sandbox with specific quotas
let quotas = ResourceQuotas {
    memory_limit: Some(64 * 1024 * 1024),  // 64 MB
    cpu_time_limit: Some(Duration::from_secs(5)),
    stack_depth_limit: Some(1000),
    file_descriptor_limit: Some(10),
    network_connection_limit: Some(5),
    disk_io_limit: Some(10 * 1024 * 1024),  // 10 MB
};

let sandbox = Sandbox::new(
    "custom-sandbox".to_string(),
    PermissionSet::new(),
    quotas
);
```

### Sandbox Operations

```rust
// Check quotas before operations
sandbox.check_memory(1024)?;
sandbox.allocate_memory(1024)?;

sandbox.check_file_descriptor()?;
sandbox.allocate_file_descriptor()?;

sandbox.check_network_connection()?;
sandbox.allocate_network_connection()?;

// Monitor resource usage
let usage = sandbox.usage();
println!("Memory used: {} bytes", usage.memory_used);
println!("CPU time: {:?}", usage.cpu_time_used);
```

### Sandbox Lifecycle

```rust
// Create sandbox with audit logging
let logger = Arc::new(MemoryAuditLogger::new());
let sandbox = Sandbox::restrictive("test".to_string())
    .with_audit_logger(logger as Arc<dyn AuditLogger>);

// Use sandbox...

// Sandbox automatically logs destruction when dropped
drop(sandbox); // Triggers SandboxDestroyed audit event
```

## Resource Quotas

### Quota Types

**Memory Quota**
- Tracks heap allocations
- Prevents memory exhaustion
- Configurable per sandbox

**CPU Time Quota**
- Limits execution time
- Prevents infinite loops
- Measured from sandbox creation

**Stack Depth Quota**
- Prevents stack overflow
- Limits recursion depth
- Per-call tracking

**File Descriptor Quota**
- Limits open files
- Prevents descriptor leaks
- Tracked across sandbox lifetime

**Network Connection Quota**
- Limits concurrent connections
- Prevents connection flooding
- Tracked per sandbox

**Disk I/O Quota**
- Limits bytes read/written
- Prevents I/O flooding
- Cumulative tracking

### Quota Presets

```rust
// Restrictive (untrusted code)
ResourceQuotas::restrictive()
// memory: 64MB, cpu: 5s, stack: 1000, fds: 10, network: 5, disk: 10MB

// Permissive (semi-trusted code)
ResourceQuotas::permissive()
// memory: 1GB, cpu: 5min, stack: 10000, fds: 1000, network: 100, disk: 1GB

// Unlimited (development/testing only)
ResourceQuotas::unlimited()
// No limits
```

## Trust Boundaries

### Trusted vs Untrusted Code

**Trusted Code**
- System libraries
- Core runtime
- Verified packages
- User's own code (with appropriate permissions)

**Untrusted Code**
- Third-party packages
- User-submitted scripts
- Downloaded code
- Dynamically loaded modules

### Privilege Levels

1. **System**: Full unrestricted access (runtime internals only)
2. **Application**: Configurable permissions (user code)
3. **Sandboxed**: Restricted environment (untrusted code)
4. **Isolated**: No permissions (pure computation)

### Transition Rules

- Trusted → Untrusted: Automatic sandboxing
- Untrusted → Trusted: Explicitly denied (no privilege escalation)
- Same level: Permission checks still apply
- Cross-boundary calls: Permission verification at boundary

## Audit Logging

### Audit Events

```rust
pub enum AuditEvent {
    PermissionCheck { operation: String, target: String, granted: bool },
    FilesystemReadDenied { path: PathBuf },
    FilesystemWriteDenied { path: PathBuf },
    NetworkDenied { host: String },
    ProcessDenied { command: String },
    EnvironmentDenied { var: String },
    SandboxCreated { sandbox_id: String, memory_limit: Option<usize>, cpu_limit: Option<u64> },
    SandboxDestroyed { sandbox_id: String },
    PolicyViolation { policy: String, violation: String },
    QuotaViolation { resource: String, limit: u64, attempted: u64 },
    PrivilegeEscalation { context: String },
    CapabilityGranted { capability_id: String, permissions: String },
    CapabilityRevoked { capability_id: String },
}
```

### Logging Backends

**MemoryAuditLogger**: In-memory storage (testing/debugging)
```rust
let logger = Arc::new(MemoryAuditLogger::new());
let entries = logger.entries();
```

**NullAuditLogger**: No-op logger (production performance)
```rust
let logger = Arc::new(NullAuditLogger::new());
```

**Custom Logger**: Implement `AuditLogger` trait
```rust
impl AuditLogger for CustomLogger {
    fn log(&self, event: AuditEvent) {
        // Custom logging implementation
    }
    
    fn entries(&self) -> Vec<AuditEntry> {
        // Return logged entries
    }
    
    fn clear(&self) {
        // Clear logged entries
    }
}
```

### Audit Log Analysis

```rust
let logger = Arc::new(MemoryAuditLogger::new());
// ... use logger ...

// Analyze audit trail
let entries = logger.entries();
for entry in entries {
    match entry.event {
        AuditEvent::QuotaViolation { resource, limit, attempted } => {
            println!("Quota violation: {} (limit: {}, attempted: {})", 
                     resource, limit, attempted);
        }
        AuditEvent::PolicyViolation { policy, violation } => {
            println!("Policy violation in {}: {}", policy, violation);
        }
        _ => {}
    }
}
```

## Best Practices

### Security Configuration

1. **Always use default-deny**: Start with no permissions, grant only what's needed
2. **Principle of least privilege**: Grant minimum permissions required
3. **Scope restrictions**: Use path scoping, domain wildcards appropriately
4. **Audit everything**: Enable audit logging in production
5. **Review policies regularly**: Security requirements change over time

### Sandbox Usage

1. **Untrusted code always sandboxed**: Never run untrusted code without sandbox
2. **Appropriate quotas**: Match quotas to expected resource usage
3. **Monitor resource usage**: Track quota consumption
4. **Handle quota errors gracefully**: Provide user feedback on quota violations
5. **Clean up resources**: Release quotas when operations complete

### Permission Management

1. **Validate inputs**: Check paths, domains, commands before permission checks
2. **Canonicalize paths**: Resolve symlinks to prevent bypass
3. **Wildcard caution**: Avoid overly broad wildcards like `*`
4. **Document permissions**: Comment why each permission is needed
5. **Test permission denial**: Verify operations fail without permission

### Policy Design

1. **Explicit deny rules**: Use deny rules for sensitive resources
2. **Policy inheritance**: Factor common permissions into base policies
3. **Environment-specific**: Different policies for dev/staging/production
4. **Version policies**: Track policy changes with version control
5. **Policy validation**: Always validate before loading

## API Reference

### Core Types

```rust
// Security context
pub struct SecurityContext { /* ... */ }
impl SecurityContext {
    pub fn new() -> Self;
    pub fn allow_all() -> Self; // WARNING: Development only
    pub fn grant_filesystem_read(&mut self, path: &Path, recursive: bool);
    pub fn grant_filesystem_write(&mut self, path: &Path, recursive: bool);
    pub fn grant_network(&mut self, host: impl Into<String>);
    pub fn grant_process(&mut self, command: impl Into<String>);
    pub fn grant_environment(&mut self, var: impl Into<String>);
    pub fn check_filesystem_read(&self, path: &Path) -> Result<(), SecurityError>;
    pub fn check_filesystem_write(&self, path: &Path) -> Result<(), SecurityError>;
    pub fn check_network(&self, host: &str) -> Result<(), SecurityError>;
    pub fn check_process(&self, command: &str) -> Result<(), SecurityError>;
    pub fn check_environment(&self, var: &str) -> Result<(), SecurityError>;
}

// Permission set
pub struct PermissionSet { /* ... */ }
impl PermissionSet {
    pub fn new() -> Self;
    pub fn grant(&mut self, permission: Permission);
    pub fn is_granted(&self, requested: &Permission) -> bool;
    pub fn len(&self) -> usize;
    pub fn merge(&mut self, other: &PermissionSet);
}

// Sandbox
pub struct Sandbox { /* ... */ }
impl Sandbox {
    pub fn new(id: String, permissions: PermissionSet, quotas: ResourceQuotas) -> Self;
    pub fn restrictive(id: String) -> Self;
    pub fn permissive(id: String) -> Self;
    pub fn with_audit_logger(self, logger: Arc<dyn AuditLogger>) -> Self;
    pub fn id(&self) -> &str;
    pub fn permissions(&self) -> &PermissionSet;
    pub fn grant_permission(&mut self, permission: Permission);
    pub fn check_memory(&self, additional: usize) -> Result<(), SandboxError>;
    pub fn allocate_memory(&self, size: usize) -> Result<(), SandboxError>;
    pub fn deallocate_memory(&self, size: usize);
    pub fn check_cpu_time(&self) -> Result<(), SandboxError>;
    pub fn check_stack_depth(&self, depth: usize) -> Result<(), SandboxError>;
    pub fn allocate_file_descriptor(&self) -> Result<(), SandboxError>;
    pub fn deallocate_file_descriptor(&self);
    pub fn allocate_network_connection(&self) -> Result<(), SandboxError>;
    pub fn deallocate_network_connection(&self);
    pub fn track_disk_io(&self, bytes: u64) -> Result<(), SandboxError>;
    pub fn usage(&self) -> ResourceUsage;
}

// Security policy
pub struct SecurityPolicy { /* ... */ }
impl SecurityPolicy {
    pub fn new(name: String) -> Self;
    pub fn from_toml(content: &str) -> Result<Self, PolicyError>;
    pub fn from_json(content: &str) -> Result<Self, PolicyError>;
    pub fn validate(&self) -> Result<(), PolicyError>;
    pub fn to_permission_set(&self) -> PermissionSet;
    pub fn allows(&self, resource: &ResourceType, pattern: &str) -> bool;
}

// Policy manager
pub struct PolicyManager { /* ... */ }
impl PolicyManager {
    pub fn new() -> Self;
    pub fn load_policy(&mut self, policy: SecurityPolicy) -> Result<(), PolicyError>;
    pub fn get_policy(&self, name: &str) -> Option<&SecurityPolicy>;
    pub fn get_permissions(&self, policy_name: &str) -> Result<PermissionSet, PolicyError>;
}
```

### Error Types

```rust
pub enum SecurityError {
    FilesystemReadDenied { path: PathBuf },
    FilesystemWriteDenied { path: PathBuf },
    NetworkDenied { host: String },
    ProcessDenied { command: String },
    EnvironmentDenied { var: String },
}

pub enum SandboxError {
    MemoryQuotaExceeded { used: usize, limit: usize },
    CpuTimeQuotaExceeded { used: Duration, limit: Duration },
    StackDepthExceeded { depth: usize, limit: usize },
    FileDescriptorQuotaExceeded { used: usize, limit: usize },
    NetworkConnectionQuotaExceeded { used: usize, limit: usize },
    DiskIOQuotaExceeded { used: u64, limit: u64 },
    EscapePrevented(String),
    OperationDenied(String),
}

pub enum PolicyError {
    ParseError(String),
    ValidationError(String),
    InvalidField { field: String, reason: String },
    NotFound(String),
}
```

## Security Considerations

### Known Limitations

1. **Symbolic link following**: Paths are canonicalized but race conditions possible
2. **Time-based checks**: System time can be manipulated
3. **Resource measurement**: CPU time tracking has overhead
4. **Policy complexity**: Complex policies may have unintended interactions

### Future Enhancements

- Code signing for trust verification
- Cryptographic capability tokens
- Hardware-backed security (TPM, SGX)
- Fine-grained FFI permissions (per-function)
- Network traffic inspection
- Filesystem virtualization

## See Also

- [Runtime API Documentation](api/runtime-api.md)
- [FFI Security Guide](features/ffi-guide.md)
- [Configuration Reference](config/README.md)
- [Atlas Security Principles](philosophy/ai-manifesto.md)
