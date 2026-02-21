# Phase v02-completion-05: JIT Formal Status + Closure Semantic Foundations

## Dependencies

**Required:** v02-completion-04 complete (stdlib hardened)

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -3   # must pass
cargo nextest run -p atlas-jit 2>&1 | tail -3        # must pass
grep "atlas-jit" Cargo.toml                          # workspace member
```

---

## Objective

Two cleanup items remain to give v0.2 a truly clean close:

**1. JIT (atlas-jit):** The crate exists with 1,522 lines of Cranelift code and 48 tests. It correctly compiles arithmetic-only functions (Constant, Add, Sub, Mul, Div, Mod, Negate, Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual, Not, GetLocal, SetLocal, Pop, Dup, Return/Halt). It does NOT support: GetGlobal, SetGlobal, Call, Jump, JumpIfFalse, or any collection/object opcodes. It is NOT wired to the VM profiler/hotspot path. This is consistent with the "foundation" phase that created it — but the crate currently has no documentation explaining this. Without clarity, the next AI session will either try to wire it (wrong — missing control flow support) or ignore it (wrong — it has real value). This phase adds a clear crate-level document and ensures the JIT tests are comprehensive for the opcodes it does support.

**2. Closure semantics:** The spec states "closures cannot reference outer scope" as a current limitation, but this contradicts how the interpreter actually works — function closures DO capture the environment through the symbol table lookup chain. The actual limitation is that `var` mutation in closures is unreliable. This phase clarifies the spec, adds tests that document the ACTUAL behavior (not the incorrect spec), and fixes the spec to match reality.

---

## Files

**Create:**
- `crates/atlas-jit/JIT_STATUS.md` — ~60 lines, formal capability matrix + roadmap
- `crates/atlas-runtime/tests/closures.rs` — ~200 lines, 30+ tests documenting actual closure behavior

**Update:**
- `crates/atlas-jit/src/lib.rs` — add crate-level doc comment with capability summary
- `docs/specification/types.md` — fix "Current Limitations" for Function Types (closures section)
- `crates/atlas-jit/src/codegen.rs` — add tests for all supported opcodes if any missing (target: 100% opcode coverage for supported set)

**Total new code:** ~300 lines tests + ~80 lines docs
**Minimum test count:** 30 new closure tests + full opcode coverage in JIT tests

---

## Implementation Notes

**JIT Status Document (`JIT_STATUS.md`):**
```markdown
# Atlas JIT Status

**Status:** Foundation complete — not yet wired to production execution path
**Last updated:** v0.2

## Supported Opcodes
[List all opcodes that compile and run correctly — from codegen.rs]

## Unsupported Opcodes
[All opcodes that return UnsupportedOpcode — and why]

## Integration Requirements (v0.3)
To wire the JIT to the VM hotspot profiler:
1. Implement Jump, JumpIfFalse, Call, Return in codegen
2. Add GetGlobal/SetGlobal with the global value array
3. Wire hotspot.rs to VM profiler — threshold = 1000 executions
4. Replace interpret loop for hot functions with JIT-compiled native fn ptr

## What Works Today
[Example: arithmetic-only functions with local variables can be JIT-compiled
and produce correct results — see tests/jit_tests.rs]
```

**Closure behavior audit (BEFORE writing tests — critical):**
Read `interpreter/expr.rs` function call handling — specifically how closures capture:
- Functions defined in outer scope: how does inner function access outer `let` variables?
- Are closures captured by value or reference to the symbol table?
- What ACTUALLY happens with `var` mutation inside a closure?
- Test empirically: write code in Atlas, run it, observe the result

Do NOT guess. Run code. Document what actually happens.

**Likely actual closure behavior (verify):**
- `let x = 5; fn get_x() -> number { return x; }` — works (captures outer let via lookup)
- `var counter = 0; fn increment() { counter = counter + 1; }` — may or may not work
- Functions passed as arguments: captured as values, not as references
- The spec says "no closure capture" but the implementation may be more permissive

**Spec fix for Function Types section:**
Replace current limitation text:
```
// WRONG (current spec):
- No closure capture (functions cannot reference outer scope)

// CORRECT (after this phase):
- Let-bound outer variables are accessible from inner functions
- Var-bound outer variables: mutation from inner functions is not guaranteed
  (behavior is implementation-defined; use return values instead)
- No anonymous function syntax (fn must be named)
```

**Critical requirements:**
- Document reality, not aspiration
- If `var` mutation in closures is broken — that's a KNOWN LIMITATION, not a bug to fix here (fixing it requires scope redesign → v0.3)
- JIT status doc must be factually accurate — list exactly which opcodes work
- Closure tests must pass as-is; tests for broken behavior should be marked `#[ignore]` with a comment explaining the v0.3 fix plan

**Error handling:**
- No new error codes needed for this phase (documentation phase primarily)

---

## Tests (TDD Approach)

**Closure behavior tests:** (30 tests)
1. Let-bound outer accessed from inner fn → works
2. Let-bound outer used in returned function → works
3. Multiple levels of nesting (3 deep)
4. Function that returns a function that closes over its arg
5. Two inner functions sharing same outer let
6. Outer let shadowed by inner parameter — inner param wins
7. Var outer accessed from inner fn (read) — document result
8. Var outer mutated from inner fn — document result (may be `#[ignore]` if broken)
9. Function stored in variable, then called — closure still valid
10. Function passed as argument — closure still valid after passing
11. Array of functions — each closure independent
12. Closure over loop variable (if for-in closures work)
13. Recursive inner function referencing outer scope
14. Higher-order function: takes fn, calls it — closure behavior preserved
15. Closure is NOT a live reference: outer var changes don't affect closed-over let
16-30: Edge cases of the above, parity tests (interpreter vs VM)

**JIT opcode coverage tests (add to `crates/atlas-jit/src/codegen.rs` or `jit_tests.rs`):**
- Verify every opcode in the supported list has at least one test
- Verify UnsupportedOpcode is correctly returned for every opcode NOT in the supported list
- Numerical correctness: `a + b`, `a - b`, `a * b`, `a / b`, `a % b` for various inputs

**Minimum test count:** 30 closure tests + opcode coverage tests

**Parity requirement:** Closure tests run in both interpreter and VM with identical results.

---

## Acceptance Criteria

- ✅ `crates/atlas-jit/JIT_STATUS.md` created with accurate capability matrix
- ✅ `atlas-jit` crate-level doc updated to reflect actual status
- ✅ All JIT-supported opcodes have dedicated tests in `jit_tests.rs`
- ✅ `docs/specification/types.md` Function Types / Current Limitations section corrected
- ✅ `crates/atlas-runtime/tests/closures.rs` created with 30+ tests
- ✅ Closure tests document actual behavior accurately (broken behavior marked `#[ignore]` with explanation)
- ✅ All non-ignored tests pass in both interpreter and VM
- ✅ No clippy warnings
- ✅ `cargo nextest run -p atlas-runtime` passes
- ✅ `cargo nextest run -p atlas-jit` passes

---

## References

**Specifications:** `docs/specification/types.md` (Function Types section)
**JIT crate:** `crates/atlas-jit/src/` (codegen.rs, hotspot.rs, lib.rs)
**Related phases:** v0.3 — JIT wiring will be a v0.3 correctness phase once control flow opcodes are supported
