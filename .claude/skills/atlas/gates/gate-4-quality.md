# GATE 4: Quality Gates

**Condition:** Implementation complete, parity verified

---

## Action

1. **Run clippy:**
   ```bash
   cargo clippy -p atlas-runtime -- -D warnings
   ```
   **MUST:** Zero warnings

2. **Run formatter:**
   ```bash
   cargo fmt -p atlas-runtime -- --check
   ```
   **MUST:** All files formatted

---

**Note:** Full test suite runs at GATE 6 (handoff), not here. See auto-memory `testing-patterns.md` for the complete testing protocol.

---

**BLOCKING:** Both must pass. No exceptions.

---

## Decision

- All pass → GATE 5
- Any fail → Fix → Retry

---

**Next:** GATE 5
