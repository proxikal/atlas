# BLOCKER 06: Security Model Implementation

**Category:** Foundation - Security Infrastructure
**Blocks:** Stdlib Phase 5 (File I/O), Phase 10 (Network HTTP), all I/O operations
**Estimated Effort:** 2-3 weeks
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Security model documentation must be complete and reviewed.

**Verification:**
```bash
ls docs/reference/io-security-model.md
grep -n "Permission" docs/reference/io-security-model.md
cargo test --lib runtime
```

**What's needed:**
- Security model documented (exists at docs/reference/io-security-model.md)
- Runtime stable
- Config system ready (BLOCKER 05)

**If missing:** Complete BLOCKER 05 first, review security docs.

---

## Objective

Implement the security model documented in `docs/reference/io-security-model.md`. Enforce permission checks for all I/O operations (filesystem, network, process). Enable safe execution of untrusted Atlas code. Foundation for secure AI agent workflows.

**Core principle:** Deny by default, explicit grants required, user consent for sensitive operations.

---

## Background

Atlas is designed for AI agents that execute code autonomously. Without security, agents could:
- Read/write arbitrary files
- Make network requests to any endpoint
- Execute system commands
- Access environment variables

**Security model:** Three-tier permission system (allow, deny, prompt) enforced at runtime. Permissions configurable via config system. All I/O operations go through security layer.

**Design reference:** Deno's permission system (explicit, capability-based, auditable).

---

## Files

### Create
- `crates/atlas-runtime/src/security/mod.rs` (~300 lines) - Security core
- `crates/atlas-runtime/src/security/permissions.rs` (~400 lines) - Permission types and checks
- `crates/atlas-runtime/src/security/policies.rs` (~300 lines) - Policy enforcement
- `crates/atlas-runtime/src/security/sandbox.rs` (~400 lines) - Sandboxing implementation
- `crates/atlas-runtime/src/security/audit.rs` (~200 lines) - Audit logging

### Modify (~15 files)
- `crates/atlas-runtime/src/lib.rs` - Export security module
- `crates/atlas-runtime/src/interpreter/mod.rs` - Add SecurityContext
- `crates/atlas-runtime/src/vm/mod.rs` - Add SecurityContext
- `crates/atlas-cli/src/main.rs` - Initialize security from config
- All I/O builtins - Add permission checks

### Tests
- `crates/atlas-runtime/tests/security_tests.rs` (~600 lines)
- `crates/atlas-runtime/tests/security_sandbox_tests.rs` (~500 lines)

**Minimum test count:** 80+ tests

---

## Implementation

### Step 1: Permission Types
Define Permission enum with variants for each capability. Filesystem (read/write with paths), Network (request with URLs), Process (spawn/exec), Environment (read env vars), FFI (call foreign functions).

**Permission granularity:**
- Filesystem: per-path or pattern (e.g., allow read ./data/*)
- Network: per-host or pattern (e.g., allow api.example.com)
- Process: per-command (e.g., allow git)
- Environment: per-variable (e.g., allow HOME)

**Permission states:**
- Allow: Always permit
- Deny: Always reject
- Prompt: Ask user at runtime (interactive mode only)

### Step 2: Security Context
Create SecurityContext struct holding active permissions. Passed to interpreter/VM at initialization. Checked before each I/O operation. Methods: check_filesystem_read(path), check_network_request(url), check_process_spawn(cmd), check_env_read(var).

**Context initialization:**
```rust
pub struct SecurityContext {
    filesystem_read: PermissionSet,
    filesystem_write: PermissionSet,
    network: PermissionSet,
    process: PermissionSet,
    environment: PermissionSet,
}

impl SecurityContext {
    pub fn from_config(config: &Config) -> Self { ... }
    pub fn check_filesystem_read(&self, path: &Path) -> Result<(), SecurityError> { ... }
    // ... other checks
}
```

### Step 3: Permission Policies
Implement policy evaluation. Policies loaded from config (BLOCKER 05). Support wildcards and patterns. Path normalization (resolve .., symlinks). URL parsing and host matching. Clear precedence rules (most specific wins).

**Policy matching:**
- Exact match: `/home/user/project/data.txt`
- Prefix match: `/home/user/project/*`
- Pattern match: `/home/user/**/*.txt`
- Deny overrides allow (security-first)

### Step 4: Runtime Enforcement
Integrate security checks into all I/O operations. Before file read/write, call check_filesystem. Before network request, call check_network. Before process spawn, call check_process. Return SecurityError if denied. Prompt user if policy is "prompt" (CLI only).

**Integration points:**
- File I/O builtins (read_file, write_file, etc.)
- Network builtins (http_request, etc.)
- Process builtins (exec, spawn, etc.)
- Environment builtins (getenv, etc.)

### Step 5: Sandboxing
Implement sandbox mode for untrusted code. Sandbox: deny all by default, must explicitly grant. Filesystem access limited to specific directories. No network by default. No process spawning. No environment access. Enforced in both interpreter and VM.

**Sandbox levels:**
- None: No restrictions (development mode)
- Standard: Deny dangerous operations, allow safe ones
- Strict: Deny all by default, explicit grants only
- Maximum: No I/O allowed at all (pure computation)

### Step 6: Audit Logging
Log all permission checks and decisions. Include: operation type, resource (path/URL/command), decision (allow/deny), timestamp, call site. Configurable verbosity. Write to audit log file. Enable post-execution review.

**Audit format:**
```
[2026-02-13T10:30:45Z] ALLOW filesystem_read /home/user/data.txt (policy: allow /home/user/*)
[2026-02-13T10:30:46Z] DENY network_request https://evil.com (policy: default deny)
```

### Step 7: Configuration Integration
Security policies defined in config files. Support both global and project-level policies. CLI flags override config (--allow-read, --deny-network). Environment variables for defaults. Clear error messages when permission denied.

**Config example:**
```toml
[security]
mode = "standard"

[security.filesystem]
read = ["./data/*", "./config.toml"]
write = ["./output/*"]

[security.network]
allow = ["api.github.com", "api.openai.com"]
deny = ["*"]

[security.process]
deny = ["*"]
```

### Step 8: Error Handling
SecurityError type with clear messages. Include: what was attempted, why it was denied, how to grant permission. Example: "Permission denied: network request to https://example.com. Add to allowed hosts in security.network.allow or run with --allow-net=example.com".

### Step 9: Interactive Prompts
For "prompt" mode, implement interactive consent. CLI shows: operation, resource, risk level, remember choice option. User responds: allow once, allow always, deny once, deny always. Choice saved to config if "always".

### Step 10: Comprehensive Testing
Test all permission types. Test policy matching (exact, prefix, pattern). Test sandbox modes. Test audit logging. Test config integration. Test error messages. Test interactive prompts (mocked). Test security bypasses fail (negative testing).

---

## Architecture Notes

**Defense in depth:** Multiple layers - policy check, sandbox enforcement, audit logging.

**Fail secure:** If permission check errors, deny by default. Never fail open.

**Capability-based:** Permissions are capabilities - explicit grants, not ambient authority.

**Auditable:** All decisions logged. Can review what code did after execution.

**Configuration-driven:** Security policies in config files, not hardcoded.

**Deny by default:** Safer to require explicit grants than rely on denials.

---

## Acceptance Criteria

**Functionality:**
- ✅ Permission types defined (filesystem, network, process, env)
- ✅ SecurityContext initialized from config
- ✅ Permission checks enforced at runtime
- ✅ Sandbox modes work (standard, strict, maximum)
- ✅ Audit logging captures all decisions
- ✅ Config integration works
- ✅ Interactive prompts work (CLI)
- ✅ Clear error messages for denials

**Quality:**
- ✅ 80+ tests pass
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ No security bypasses
- ✅ Comprehensive negative testing
- ✅ Audit logs parseable

**Documentation:**
- ✅ Update io-security-model.md with implementation notes
- ✅ Security guide in docs/guides/security.md
- ✅ Config documentation updated
- ✅ Examples for common patterns
- ✅ Migration guide for new security features

---

## Dependencies

**Requires:**
- BLOCKER 05: Configuration System (for policy config)
- Stable runtime (interpreter + VM)
- CLI infrastructure

**Blocks:**
- Stdlib Phase 5: File I/O API (needs filesystem permissions)
- Stdlib Phase 10: Network HTTP (needs network permissions)
- Any phase with I/O operations
- FFI implementation (needs FFI permissions)
- Process spawning features

---

## Rollout Plan

1. Define permission types (2 days)
2. Implement SecurityContext (2 days)
3. Policy evaluation (3 days)
4. Runtime enforcement (4 days)
5. Sandboxing (3 days)
6. Audit logging (2 days)
7. Config integration (2 days)
8. Interactive prompts (2 days)
9. Testing (4 days)
10. Documentation (2 days)

**Total: ~26 days (4 weeks with thorough testing)**

Security cannot be rushed. Allocate proper time.

---

## Known Limitations

**No OS-level sandboxing yet:** Uses runtime checks, not OS isolation (containers, chroot, etc.). OS-level sandboxing deferred to later.

**No capability tokens:** Permissions are context-based, not token-based. Capability tokens (unforgeable references) deferred.

**No time-based permissions:** Cannot grant temporary access (expires after 1 hour). Time-based permissions deferred.

**No rate limiting:** Cannot limit operations per second. Rate limiting deferred.

These are acceptable for initial security. Provides strong baseline protection.

---

## Examples

**Allow file read for specific directory:**
```toml
[security.filesystem]
read = ["./data/*"]
```

**Deny all network access:**
```toml
[security.network]
deny = ["*"]
```

**Prompt for process spawning:**
```toml
[security.process]
prompt = ["git", "npm"]
```

**Runtime permission denial:**
```
Error: Permission denied: filesystem write to /etc/passwd
  Reason: Path not in allowed write list
  Fix: Add to security.filesystem.write in atlas.toml or run with --allow-write=/etc/passwd

  Note: Writing to /etc/passwd is dangerous. Are you sure?
```

**Audit log entry:**
```
[2026-02-13T10:30:45Z] DENY filesystem_write /etc/passwd (policy: default deny, attempted from main.atl:15)
```

---

## Risk Assessment

**Very high risk:**
- Security bypasses (catastrophic)
- Policy evaluation bugs (allow when should deny)
- TOCTOU (time-of-check-time-of-use) races
- Path traversal vulnerabilities

**Mitigation:**
- Extensive security testing
- Negative testing (try to bypass)
- Reference Deno implementation
- Security review by multiple people
- Fuzz testing for policy matching
- Path canonicalization before checks
- Threat modeling

**This is security. Test exhaustively. No shortcuts.**

---

## Security Checklist

Before marking complete:
- [ ] All I/O operations checked
- [ ] No bypasses found in negative testing
- [ ] Path canonicalization prevents traversal
- [ ] Symbolic links handled correctly
- [ ] Race conditions considered
- [ ] Error messages don't leak sensitive info
- [ ] Audit logs cannot be disabled by code
- [ ] Config parsing rejects malicious input
- [ ] Prompt UI cannot be spoofed
- [ ] Default deny verified for all operations

**Security is not optional. This must be bulletproof.**
