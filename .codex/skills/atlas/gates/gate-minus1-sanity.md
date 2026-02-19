# GATE -1: Sanity Check

**Purpose:** Verify before work starts. AI is Lead Developer — makes technical decisions using spec/docs. User is Architect — strategic decisions only.

---

## Action

1. **Communication check:** Am I making assumptions? → Verify using spec/docs FIRST, not user
2. **Read phase blockers:** Check `🚨 BLOCKERS` section in phase file
3. **Verify each dependency:** Check spec → check codebase → decide autonomously
4. **Sanity check:** `cargo clean && cargo check -p atlas-runtime`
5. **Evaluate:** Version scope? Dependencies met? Parity impact? Workload reasonable?

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
