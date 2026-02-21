# GATE -1: Sanity Check

**Purpose:** Verify before work starts. AI is Lead Developer — makes technical decisions using spec/docs. User is Architect — strategic decisions only.

---

## Action

1. **Read phase blockers:** Check `🚨 BLOCKERS` section in phase file
2. **Verify each dependency:** Check spec → check codebase → decide autonomously
3. **Git check:** Ensure on feature branch (not main), working directory clean
4. **Full build:** `cargo build --workspace` — MUST succeed before any work begins
5. **Sanity check:** `cargo check -p atlas-runtime`
6. **Security scan:** `cargo audit` (check for known vulnerabilities)
   - If vulnerabilities found in direct deps → STOP, alert user
   - If vulnerabilities in transitive deps only → note and continue
7. **Evaluate:** Version scope? Dependencies met? Parity impact? Workload reasonable?

---

## Security Scanning

```bash
# Install if needed (one-time)
cargo install cargo-audit

# Run in GATE -1
cargo audit
```

**Note:** `cargo deny` is optional but recommended for stricter checks:
```bash
cargo install cargo-deny
cargo deny check
```

---

## Decision Tree: Technical vs Architectural

**TECHNICAL (AI decides using spec/docs):**
- How should feature X work? → Check spec
- What approach to use? → Check best practices, decide
- Is dependency X ready? → Verify in codebase

**ARCHITECTURAL (ask user):**
- Should we prioritize performance or memory?
- Should feature X be in v0.2 or v0.3?
- What's the project priority right now?

---

**If concerns found:** Present with evidence, suggest spec-compliant alternatives
**If no concerns:** Proceed to GATE 0

**Next:** GATE 0
