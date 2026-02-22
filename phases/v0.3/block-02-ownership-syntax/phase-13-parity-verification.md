# Phase 13: Parity Verification — Ownership Semantics

**Block:** 2 (Ownership Syntax)
**Depends on:** Phases 08, 09, 11, 12 (both engines enforcing ownership)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-runtime/tests/` (new parity test file or existing parity domain file)

## Summary

Verify that the interpreter and VM produce identical output for every ownership annotation
scenario. Zero divergence is required. This is the parity gate before Block 2 can be
declared complete.

## Current State

Verified: Block 1 established 32+ parity tests in `tests/`. There is an existing parity
test infrastructure for running identical source through both engines and asserting equal
output. After Phase 12, both engines implement ownership enforcement — now verify they match.

Check existing test files for the correct domain file to add ownership parity tests to
(do NOT create a new test file if a suitable domain file exists — see testing-patterns.md).

## Requirements

Write parity tests covering all ownership annotation scenarios. Each test runs the same
Atlas source through both interpreter and VM and asserts identical output or identical error.

**Required scenarios (minimum 20 parity tests):**

1. Unannotated function call — both engines: same output as before Block 2 (no regression)
2. `own` param — callee receives value: both engines produce same result
3. `own` param — caller uses consumed binding (debug): both engines produce same error message
4. `borrow` param — caller retains value after call: both engines: same output
5. `shared` param with `SharedValue` argument: both engines accept
6. `shared` param with plain value argument (debug): both engines produce same error message
7. Mixed annotations: `fn f(own a: ..., borrow b: ..., c: ...)` — all three in one call
8. `own` with literal argument — no binding consumed: both engines OK
9. `own` return type annotation — parsed and ignored at runtime (v0.3): both engines
10. `borrow` return type annotation — same
11. Nested function calls with ownership propagation
12. `own` param where argument is a function call result (no binding to consume)
13. Multiple `borrow` calls to same value — all succeed
14. `own` followed by `borrow` of same value — error on own-use, not on borrow
15. Ownership annotations on recursive functions
16. Function value stored in variable with `own` annotation
17. `shared` value passed through multiple function calls
18. `own` annotation on `void` function (no return): both engines OK
19. Ownership in nested scope (inner function calling outer with `own` param)
20. Error message format is identical between engines (not just error/success, but message text)

## Acceptance Criteria

- [ ] All 20+ parity tests pass
- [ ] Zero divergence between interpreter and VM for all ownership scenarios
- [ ] Error messages are identical (not engine-dependent wording)
- [ ] No regression in existing parity tests from Block 1
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

See requirements above. Template:
```rust
fn assert_ownership_parity(source: &str) {
    let interp_result = eval_interpreter(source);
    let vm_result = eval_vm(source);
    assert_eq!(interp_result, vm_result,
        "Ownership parity failure for source:\n{}", source);
}

#[test]
fn test_parity_own_param_consumes_binding() {
    assert_ownership_parity(r#"
        fn consume(own data: array<number>) -> void { }
        let arr = [1, 2, 3];
        consume(arr);
        arr;
    "#);
    // Both engines return Err with same message
}
```

## Notes

- If a parity divergence is found, it is a BLOCKING issue — do not declare this phase
  complete until divergence is resolved.
- The error message format established here becomes the spec that v0.4's compile-time
  verifier must replicate as a compile error.
