# BLOCKER 06-B: Security Runtime Enforcement

**Part:** 2 of 3 (Runtime Enforcement)
**Category:** Foundation - Security Infrastructure
**Estimated Effort:** 1 week
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** BLOCKER 06-A complete.

**Verification:**
```bash
cargo test security_permission_tests --no-fail-fast
grep -n "SecurityContext" crates/atlas-runtime/src/security/permissions.rs
```

**What's needed:**
- ✅ Permission system defined
- ✅ SecurityContext works
- ✅ Policies load

**If missing:** Complete BLOCKER 06-A first.

---

## Objective

**THIS PHASE:** Integrate security checks into I/O operations. Sandbox modes work. Permission checks enforced at runtime.

**Success criteria:** File/network/process operations check permissions. Denials work correctly.

---

## Implementation

### Step 1-3: Runtime Integration (Days 1-6)

Add permission checks to all I/O operations. Implement sandbox modes. Return SecurityError when denied.

### Step 4: Testing (Day 7)

30+ tests for runtime enforcement.

---

## Acceptance Criteria

- ✅ I/O operations check permissions
- ✅ Sandbox modes work
- ✅ Denials handled correctly
- ✅ 30+ tests pass

**Next:** blocker-06-c-audit-configuration.md
