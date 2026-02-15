# Phase 15: Security and Permissions Model

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Runtime API (phase-01), Embedding API (phase-02), and FFI (phase-10) must be complete.

**Verification Steps:**
1. Check STATUS.md: Foundation section
   - phase-01 (Runtime API) should be ✅
   - phase-02 (Embedding API) should be ✅
   - phase-10 (FFI) should be ✅

2. Verify phase-01 (Runtime API) complete:
   ```bash
   ls crates/atlas-runtime/src/api/runtime.rs
   grep -n "pub struct Runtime" crates/atlas-runtime/src/api/runtime.rs
   cargo test api_tests 2>&1 | grep "test result"
   ```

3. Verify phase-02 (Embedding API) complete:
   ```bash
   ls crates/atlas-runtime/src/api/native.rs
   grep -n "RuntimeConfig\|sandbox" crates/atlas-runtime/src/api/runtime.rs
   cargo test api_native_functions_tests 2>&1 | grep "test result"
   ```

4. Verify phase-10 (FFI) complete:
   ```bash
   ls crates/atlas-runtime/src/ffi/mod.rs
   cargo test ffi_tests 2>&1 | grep "test result"
   ```

**Expected from phase-01 (per acceptance criteria):**
- Runtime struct with eval/call methods
- Execution mode support (interpreter/VM)
- 80+ tests passing

**Expected from phase-02 (per acceptance criteria):**
- RuntimeConfig with sandboxing options
- Native function registration
- Sandboxed runtime constructor
- 60+ tests passing

**Expected from phase-10 (per acceptance criteria):**
- FFI infrastructure for foreign calls
- Permission checks will gate FFI operations
- 80+ tests passing

**Decision Tree:**

a) If all 3 phases complete (STATUS.md ✅, files exist, tests pass):
   → Proceed with phase-15
   → Build on existing RuntimeConfig from phase-02
   → Add permission checks to FFI from phase-10

b) If phase-01 or phase-02 incomplete:
   → STOP immediately
   → Report: "Foundation phases 01, 02, and 10 required before phase-15"
   → Complete missing phases in order: 01 → 02 → 10
   → Then return to phase-15

c) If phase-10 incomplete:
   → STOP immediately
   → Report: "Foundation phase-10 (FFI) required before phase-15"
   → Complete phase-10 first
   → Then return to phase-15

d) If any phase marked complete but tests failing:
   → That phase is not actually complete
   → Fix the failing phase first
   → Verify all tests pass
   → Then proceed with phase-15

**No user questions needed:** All prerequisite phases are verifiable via STATUS.md, file existence, and cargo test.

---

## Objective
Implement comprehensive security and permissions model controlling access to system resources, enabling safe execution of untrusted code, and providing capability-based security - making Atlas safe for embedding and running third-party code.

## Files
**Create:** `crates/atlas-runtime/src/security/mod.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/security/permissions.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/security/sandbox.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/security/policy.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/api/runtime.rs` (~200 lines security integration)
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~150 lines permission checks)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~150 lines permission checks)
**Create:** `docs/security-model.md` (~1000 lines)
**Tests:** `crates/atlas-runtime/tests/security_tests.rs` (~800 lines)

## Dependencies
- Runtime API with configuration
- Sandboxing infrastructure
- FFI system for resource access
- Standard library with I/O operations

## Implementation

### Permission System Design
Define granular permission types for resource access. FileReadPermission for reading files. FileWritePermission for writing files. NetworkPermission for network access. FFIPermission for foreign function calls. ProcessPermission for spawning processes. EnvironmentPermission for env vars. ReflectionPermission for reflection API. Each permission with optional scope restrictions. Path-based scoping for file permissions. Domain-based scoping for network. Function-based scoping for FFI.

### Capability-Based Security
Implement capability-based access control. Capabilities are unforgeable tokens granting permissions. Runtime creates capabilities for granted permissions. Operations require presenting valid capability. Capabilities can be attenuated reducing permissions. Capabilities passed explicitly to functions. No ambient authority everything requires capability. Revocable capabilities for dynamic permission changes.

### Security Policy Definition
Define security policies declaratively. Policy files in TOML or JSON format. Specify allowed and denied permissions. Permission scopes and restrictions. Time-based permission grants. Policy inheritance for modularity. Default-deny policy for safety. Whitelist trusted code paths. Blacklist dangerous operations. Policy validation at load time.

### Permission Checking Infrastructure
Check permissions before privileged operations. File operation checks before open, read, write. Network operation checks before connect, listen. FFI call checks before dynamic library load. Process operation checks before spawn. Environment checks before getenv, setenv. Reflection checks before introspection. Fast permission lookup with caching. Clear error messages on permission denial. Audit log for permission checks.

### Sandbox Enforcement
Enforce sandboxing at runtime boundaries. Isolate untrusted code in sandbox. Restrict resource access via permissions. Memory limits enforced. CPU time limits enforced. Stack depth limits to prevent overflow. Heap allocation limits to prevent exhaustion. No access to filesystem without permission. No network access without permission. No FFI without permission. Escape prevention strict boundary checks.

### Trusted and Untrusted Code Separation
Distinguish trusted and untrusted code execution. Trusted code runs with full permissions. Untrusted code runs in sandbox with minimal permissions. Transition boundaries marked explicitly. Privilege escalation prevented. Sandbox escape detection. Stack inspection for caller validation. Code signing for trust verification (future).

### Resource Quotas and Limits
Enforce resource usage quotas preventing abuse. Maximum memory allocation per sandbox. Maximum file descriptors. Maximum network connections. Maximum CPU time. Maximum disk I/O. Maximum network bandwidth. Quota exhaustion errors. Quota monitoring and reporting. Configurable quota policies.

### Security Audit and Logging
Log security-relevant events for audit. Permission grant and denial events. Sandbox creation and destruction. Policy violations. Resource quota violations. Attempted privilege escalation. Suspicious behavior detection. Audit log rotation. Structured logging for analysis. Privacy considerations in logging.

## Tests (TDD - Use rstest)

**Permission system tests:**
1. Grant file read permission
2. Grant file write permission
3. Grant network permission
4. Grant FFI permission
5. Deny permission on scope mismatch
6. Permission with path scope
7. Permission with domain scope
8. Permission check performance

**Capability tests:**
1. Create capability for permission
2. Attentuate capability reducing scope
3. Revoke capability
4. Capability presentation required
5. Invalid capability rejected
6. Capability unforgeable
7. Capability passing to functions

**Security policy tests:**
1. Load policy from file
2. Parse policy configuration
3. Default-deny policy
4. Whitelist specific paths
5. Blacklist dangerous operations
6. Policy inheritance
7. Invalid policy detection
8. Policy validation

**Permission checking tests:**
1. Check file read permission
2. Check file write permission
3. Check network permission
4. Check FFI permission
5. Deny operation without permission
6. Permission error message
7. Audit log entry created

**Sandbox enforcement tests:**
1. Create sandbox with limits
2. File access denied in sandbox
3. Network access denied in sandbox
4. FFI denied in sandbox
5. Memory limit enforced
6. CPU time limit enforced
7. Sandbox escape prevented
8. Stack depth limit enforced

**Trust boundary tests:**
1. Trusted code runs unrestricted
2. Untrusted code runs in sandbox
3. Transition boundary crossing
4. Privilege escalation prevented
5. Caller validation via stack inspection

**Resource quota tests:**
1. Memory quota enforcement
2. File descriptor quota
3. Network connection quota
4. CPU time quota
5. Disk I/O quota
6. Quota exhaustion error
7. Quota monitoring

**Audit logging tests:**
1. Log permission grant
2. Log permission denial
3. Log sandbox creation
4. Log policy violation
5. Log quota violation
6. Audit log structured format
7. Audit log rotation

**Integration tests:**
1. Run untrusted code in sandbox
2. Sandbox file access denied
3. Sandbox network access denied
4. Trusted code accessing resources
5. Permission-based API usage
6. Security policy enforcement
7. Complete sandbox lifecycle

**Minimum test count:** 90 tests

## Integration Points
- Uses: Runtime API from phase-01
- Uses: Sandboxing from phase-02
- Uses: FFI from phase-10
- Uses: Stdlib I/O operations
- Updates: Runtime with permission checks
- Updates: Interpreter with security
- Updates: VM with security
- Creates: Security infrastructure
- Output: Safe execution of untrusted code

## Acceptance
- Permission system grants and denies access
- Capability-based security prevents ambient authority
- Security policies define permissions declaratively
- Permission checks enforce before operations
- Sandbox prevents resource access
- Resource quotas prevent abuse
- Audit logging tracks security events
- Untrusted code runs safely in sandbox
- Trusted code runs without restrictions
- 90+ tests pass
- Security model documentation comprehensive
- Threat model documented
- Best practices guide included
- No clippy warnings
- cargo test passes
