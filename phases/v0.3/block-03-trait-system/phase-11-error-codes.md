# Phase 11 — Error Codes: AT3xxx Diagnostic Registry

**Block:** 3 (Trait System)
**Depends on:** Phase 10 complete
**Estimated tests added:** 10–15

---

## Objective

Audit all AT3xxx error codes emitted across Phases 06–10. Ensure every code is:
1. Registered in `diagnostic.rs`
2. Has a clear, actionable error message
3. Has at least one test that triggers it
4. Appears in the diagnostic system's code registry (for `--explain` support if applicable)

This is a consolidation phase — no new feature code, only hardening.

---

## Current State (verified after Phase 10)

Codes introduced across Phases 06–10:

| Code | Meaning | Phase introduced |
|------|---------|-----------------|
| AT3001 | Redefines built-in trait | Phase 06 |
| AT3002 | Trait already defined | Phase 06 |
| AT3003 | Trait not found | Phase 06 |
| AT3004 | Impl missing required method | Phase 07 |
| AT3005 | Impl method signature mismatch | Phase 07 |
| AT3006 | Type does not implement trait (call site) | Phase 08 |
| AT3007 | Copy type required | Phase 09 (AT3007 alias for Copy-specific case) |
| AT3008 | Trait bound not satisfied | Phase 10 |
| AT3009 | Impl already exists for (type, trait) | Phase 07 |
| AT3010 | Move type passed without ownership annotation (warning) | Phase 09 |

---

## Audit Tasks

### 1. Verify all codes are in `diagnostic.rs`

```bash
grep -n "AT3" crates/atlas-runtime/src/diagnostic.rs
```

Expected: 10 constants defined. Add any that are missing.

### 2. Verify all codes have tests

For each AT3xxx code, confirm there is at least one test in `typesystem.rs` that:
- Triggers the code
- Asserts the error contains the code string OR contains the expected message
- Is labeled with the code in a comment

If any code is untested, add a test.

### 3. Error message quality audit

Review each diagnostic emission site and ensure messages follow the pattern:
```
"[Brief description]: [specific detail]. [Hint if helpful]"
```

Good: `"Trait 'Display' is not defined. Did you forget to declare it with 'trait Display { ... }'?"`
Bad: `"Unknown trait"`

Update messages that are too terse.

### 4. AT3006 vs AT3008 distinction

These two may overlap:
- AT3006: Calling a method on a type that doesn't implement the relevant trait
- AT3008: Trait bound on a generic type parameter not satisfied

They are distinct situations — ensure they emit different codes with different messages.
AT3006: `"Type 'string' does not implement trait 'Display'"` (method call context)
AT3008: `"Type 'string' does not satisfy trait bound 'Display' for type parameter 'T'"` (generic context)

### 5. Warning vs error classification

AT3010 is intentionally a **warning** (not error) in Block 3. Verify the diagnostic
level is `Diagnostic::warning(...)` not `Diagnostic::error(...)`.

All other AT3xxx codes should be `Diagnostic::error(...)`.

---

## Diagnostic Registry Format

Check `diagnostic.rs` for how codes are structured. The format from Block 2:

```rust
pub const TRAIT_NOT_FOUND: &str = "AT3003";
// ... etc
```

Ensure each constant has a doc comment explaining when it fires:

```rust
/// Fired when an `impl` block references a trait that has not been declared.
/// Check that the trait is declared with `trait TraitName { ... }` before the impl.
pub const TRAIT_NOT_FOUND: &str = "AT3003";
```

---

## Tests

Add to `crates/atlas-runtime/tests/typesystem.rs` (one per untested code):

```rust
// AT3001 — test in Phase 06 (redefines built-in)
// AT3002 — test in Phase 06 (duplicate trait)
// AT3003 — test in Phase 06 (impl unknown trait)

#[test]
fn test_at3004_impl_missing_required_method() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait HasArea { fn area(self: HasArea) -> number; }
        impl HasArea for number { }  // missing 'area'
    ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("AT3004"));
}

#[test]
fn test_at3005_impl_method_wrong_return_type() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Stringify { fn to_str(self: Stringify) -> string; }
        impl Stringify for number {
            fn to_str(self: number) -> number { return self; }  // should return string
        }
    ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("AT3005"));
}

#[test]
fn test_at3006_method_call_on_non_implementing_type() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Flippable { fn flip(self: Flippable) -> bool; }
        impl Flippable for bool { fn flip(self: bool) -> bool { return !self; } }
        let n: number = 42;
        n.flip();  // number doesn't implement Flippable
    ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("AT3006"));
}

#[test]
fn test_at3007_copy_type_required_context() {
    // AT3007 is for contexts where Copy is explicitly required but type isn't Copy
    // This fires when ownership checking detects a Copy-required context
    // (exact trigger depends on Phase 09 implementation)
    // Add test once Phase 09 behavior is concrete
}

#[test]
fn test_at3008_generic_bound_not_satisfied() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Saveable { fn save(self: Saveable) -> void; }
        fn persist<T: Saveable>(x: T) -> void { x.save(); }
        persist(42);  // number doesn't implement Saveable
    ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("AT3008"));
}

#[test]
fn test_at3009_duplicate_impl() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Marker { }
        impl Marker for string { }
        impl Marker for string { }  // duplicate
    ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("AT3009"));
}

#[test]
fn test_at3010_move_type_warning_not_error() {
    // AT3010 is a WARNING — the program should still run
    // (This is hard to test without accessing diagnostics directly —
    //  if the runtime returns Ok, it means the warning didn't become a hard error)
    let atlas = Atlas::new();
    // If we could access diagnostics: check that warning is emitted but eval succeeds
    // For now, verify the code runs (warning doesn't block execution)
}
```

Also add to `crates/atlas-runtime/tests/diagnostics.rs`:

```rust
#[test]
fn test_all_at3xxx_codes_are_registered() {
    // Verify the code strings exist (compile-time check via constants)
    let codes = [
        error_codes::TRAIT_REDEFINES_BUILTIN,       // AT3001
        error_codes::TRAIT_ALREADY_DEFINED,          // AT3002
        error_codes::TRAIT_NOT_FOUND,                // AT3003
        error_codes::IMPL_METHOD_MISSING,            // AT3004
        error_codes::IMPL_METHOD_SIGNATURE_MISMATCH, // AT3005
        error_codes::TYPE_DOES_NOT_IMPLEMENT_TRAIT,  // AT3006
        error_codes::COPY_TYPE_REQUIRED,             // AT3007
        error_codes::TRAIT_BOUND_NOT_SATISFIED,      // AT3008
        error_codes::IMPL_ALREADY_EXISTS,            // AT3009
        error_codes::MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION, // AT3010
    ];
    // All codes should start with "AT3"
    for code in &codes {
        assert!(code.starts_with("AT3"), "Code '{}' should be in AT3xxx range", code);
    }
}
```

---

## Acceptance Criteria

- [ ] All 10 AT3xxx codes registered in `diagnostic.rs` with doc comments
- [ ] Every code has at least one test that asserts it fires
- [ ] Error messages are clear and actionable (not one-word descriptions)
- [ ] AT3010 is a `warning`, all others are `error`
- [ ] AT3006 and AT3008 are distinct with distinct messages
- [ ] `cargo test` on `typesystem.rs` passes all AT3xxx tests
- [ ] All existing tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- This phase is a safety net — it's easy to emit a code in one phase and forget to
  register the constant, or emit the wrong code by copy-paste. The audit catches these.
- The `--explain AT3004` CLI feature (if it exists) should return the doc comment text.
  Check if Atlas CLI has this feature. If yes, ensure doc comments are complete sentences.
