# GATE 3: Verify Parity (100% REQUIRED)

**Condition:** Implementation and tests complete

---

## ⚠️ PARITY IS NON-NEGOTIABLE

Interpreter and VM MUST produce **identical output** for the same Atlas program. No exceptions.

---

## Verification

### Use `assert_parity()` helper

The preferred pattern is a single test that runs both engines:

```rust
#[test]
fn test_feature_parity() {
    assert_parity(r#"let x = 42; print(x);"#, "42");
}
```

This runs the code in both interpreter and VM and asserts identical output. See auto-memory `testing-patterns.md` for details.

### Run parity tests

```bash
# Run all parity tests
cargo nextest run -p atlas-runtime -E 'test(parity)'

# Run domain file that contains parity tests
cargo nextest run -p atlas-runtime --test <domain_file>
```

### Common parity violations

- Missing intrinsic in one engine
- Different error messages between engines
- Different argument validation order
- Method dispatch divergence (see Correctness phase-05)

---

## Decision

- All parity tests pass → GATE 4
- Any fail → **BLOCKING** → Fix → Retry

**100% parity or the phase is INCOMPLETE.**

---

**Next:** GATE 4
