# Phase 18: Fix Aliasing Tests

**Block:** 1 (Memory Model)
**Depends on:** Phase 17 complete (build clean)

---

## Objective

Identify and fix all existing tests that assumed the OLD aliasing behavior (reference
semantics via `Arc<Mutex>`). These tests are EXPECTED to fail after Block 1's value
semantics changes. They must be updated to reflect correct CoW behavior.

---

## Background

Under the old model, `let b = a` where `a` is an array made `b` an alias: mutations to
`b` were visible through `a`. Some tests were written to verify this (now-wrong) behavior.
Under the new model, `let b = a` creates an independent copy (CoW).

These tests don't represent bugs in the new implementation — they represent tests written
for the old, incorrect semantics. They must be rewritten.

---

## Finding Aliasing Tests

```bash
# In test files, look for patterns that assert old aliasing behavior
grep -rn "ptr_eq\|same.*array\|alias\|shared.*array\|reference.*semantics" \
    crates/atlas-runtime/tests/ --include="*.rs"

# Also look for Atlas source test files:
grep -rn "let b = a" crates/atlas-runtime/tests/ --include="*.rs" -A 5 | \
    grep -B 2 "assert.*b.*a\|assert.*a.*b"
```

Also run the full test suite and note which tests fail — these are the aliasing tests:
```bash
cargo nextest run -p atlas-runtime 2>&1 | grep "FAILED"
```

---

## Categories of Tests to Fix

### Category 1: Tests that verify aliasing (must be inverted)
```rust
// OLD TEST (wrong behavior):
let a = Value::Array(Arc::new(Mutex::new(vec![Value::Number(1.0)])));
let b = a.clone();
// Push to b via Arc
b_guard.push(Value::Number(2.0));
assert_eq!(a_guard.len(), 2); // a was also affected ← this was the BUG

// NEW TEST (correct behavior):
let mut a = ValueArray::from_vec(vec![Value::Number(1.0)]);
let mut b = a.clone(); // CoW clone
b.push(Value::Number(2.0));
assert_eq!(a.len(), 1); // a is NOT affected ← correct
assert_eq!(b.len(), 2);
```

### Category 2: Tests using `Value::Array(Arc::new(Mutex::new(...)))`
All such test setup code must be rewritten:
```rust
// OLD:
Value::Array(Arc::new(Mutex::new(vec![...])))
// NEW:
Value::Array(ValueArray::from_vec(vec![...]))
```

### Category 3: Atlas-language integration tests
If any `.atlas` test files test aliasing, update them:
```atlas
// OLD TEST (expected old aliasing — now wrong):
let a = [1, 2, 3]
let b = a
push(b, 4)
assert(a.len() == 4)  // ← was testing the bug

// NEW TEST:
let a = [1, 2, 3]
let b = a
b.push(4)
assert(a.len() == 3)  // value semantics: a unaffected
assert(b.len() == 4)
```

---

## Tests to ADD (verify new semantics)

After fixing broken tests, add explicit value-semantics tests:

```rust
// Regression tests that verify CoW semantics are permanent:
#[test]
fn array_value_semantics_regression() {
    // Any test marked @regression in aliasing behavior
    // This suite should never regress to reference semantics
    let program = r#"
        let a = [1, 2, 3]
        let b = a
        b.push(4)
        assert(a == [1, 2, 3])
        assert(b == [1, 2, 3, 4])
    "#;
    assert_interpreter_output(program, "");
    assert_vm_output(program, "");
}
```

---

## Acceptance Criteria

- [ ] `cargo nextest run -p atlas-runtime` — zero failures related to aliasing
- [ ] No test uses `Arc::new(Mutex::new(vec![...]))` for `Value::Array` setup
- [ ] All array aliasing tests updated to assert CoW behavior
- [ ] At least 5 explicit regression tests for value semantics added
- [ ] Both engines pass all aliasing regression tests
