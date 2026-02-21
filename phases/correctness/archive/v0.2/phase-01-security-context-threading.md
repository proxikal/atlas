# Phase Correctness-01: Safe Security Context Threading

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Infra phases complete. Suite green. Build passes.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
cargo nextest run -p atlas-runtime 2>&1 | tail -3
```

---

## Objective

Both the interpreter and VM currently store the active `SecurityContext` as a raw C pointer (`*const SecurityContext`), cast it back to a reference with `unsafe`, and dereference it across every builtin call. This is incorrect Rust: the pointer is dangling after `eval()` returns, it bypasses lifetime guarantees, and it requires `unsafe` scattered across production code in three separate call sites.

Every professional Rust codebase treats raw pointer workarounds for lifetime problems as a design bug, not a feature. The correct pattern for shared, read-only state threaded through a call tree is `Arc<T>`. This phase replaces every raw pointer usage with `Arc<SecurityContext>`, eliminating all `unsafe` blocks that exist solely to work around this design flaw.

---

## Files Changed

- `crates/atlas-runtime/src/security/mod.rs` — derive/implement `Clone` on `SecurityContext` (needed for Arc construction)
- `crates/atlas-runtime/src/interpreter/mod.rs` — change field type, update `eval()` and `eval_with_config()`
- `crates/atlas-runtime/src/interpreter/expr.rs` — remove all three `unsafe { &*self.current_security... }` blocks
- `crates/atlas-runtime/src/vm/mod.rs` — same: change field, remove `unsafe` dereferences
- `crates/atlas-runtime/src/api/runtime.rs` — update construction to pass `Arc<SecurityContext>`

---

## Dependencies

- Infra phases complete (clean build required)
- No other correctness phases are prerequisites

---

## Implementation

### Step 1: Audit SecurityContext for Clone

Read `security/mod.rs` and `security/permissions.rs`. Determine if `SecurityContext` can derive `Clone` or if it needs a manual implementation. If it contains `Mutex` fields, those can be wrapped in `Arc` before cloning. Document in this phase's commit message what was required. Do not skip this — if `SecurityContext` cannot be cloned cheaply, use `Arc<SecurityContext>` everywhere so the clone cost is a reference count increment only.

### Step 2: Change the field type in Interpreter

In `interpreter/mod.rs`, change:
```rust
pub(super) current_security: Option<*const crate::security::SecurityContext>,
```
to:
```rust
pub(super) current_security: Option<Arc<crate::security::SecurityContext>>,
```

In `eval()`, change the assignment from raw-pointer cast to `Arc::clone` of the passed context. The caller already passes `&SecurityContext` — construct an `Arc` from it or require the caller to pass `Arc<SecurityContext>` directly. Choose the approach that requires fewer API surface changes: accepting `Arc<SecurityContext>` at `eval()` is the cleanest.

### Step 3: Remove unsafe blocks from interpreter/expr.rs

All three occurrences of:
```rust
let security = unsafe { &*self.current_security.expect("Security context not set") };
```
become:
```rust
let security = self.current_security.as_ref().expect("Security context not set");
```

No `unsafe`. No raw pointer. The `Arc<SecurityContext>` dereferences to `&SecurityContext` automatically. The `.as_ref()` converts `Option<Arc<T>>` to `Option<&Arc<T>>`; `.expect()` unwraps safely.

### Step 4: Apply the same change to VM

`vm/mod.rs` has the same field and the same two `unsafe` dereference patterns. Apply identical changes. The VM's `run()` and `execute_step()` methods set `current_security` — update both to store `Arc<SecurityContext>` rather than cast a raw pointer.

### Step 5: Update api/runtime.rs

The `Runtime` struct constructs security contexts and passes them to interpreter/VM. Update those construction paths to produce `Arc<SecurityContext>`. The public API surface (`eval()`, `run()`) should accept either `Arc<SecurityContext>` or remain unchanged depending on what is cleanest — prefer keeping the public API identical if possible by constructing the `Arc` internally.

### Step 6: Verify zero unsafe remains for this pattern

```bash
grep -rn "current_security.*\*const\|unsafe.*current_security" \
    crates/atlas-runtime/src/
```
Must return no results. The only remaining `unsafe` blocks in the runtime after this phase should be FFI calls and the VM's hot-path unchecked indexing (which are intentional and performance-justified).

---

## Tests

No new tests are required — this is a pure refactor with identical runtime semantics. All existing tests must pass unchanged. The absence of `unsafe` for security context access is verified by code inspection (`grep` above).

Add a compile-time check: ensure `SecurityContext` implements `Send + Sync` (required for `Arc<SecurityContext>`). Add to `security/mod.rs`:
```rust
fn _assert_send_sync() {
    fn check<T: Send + Sync>() {}
    check::<SecurityContext>();
}
```
This function is never called — it exists only to produce a compile error if the invariant is broken.

---

## Integration Points

- `security/mod.rs` — SecurityContext must implement Clone (or be proven clonable via Arc)
- `interpreter/mod.rs` + `interpreter/expr.rs` — field type + 3 unsafe blocks removed
- `vm/mod.rs` — field type + 2 unsafe blocks removed
- `api/runtime.rs` — construction updated

---

## Acceptance

- Zero raw pointer casts for security context in `interpreter/mod.rs`, `interpreter/expr.rs`, `vm/mod.rs`
- Zero `unsafe` blocks introduced by this change (FFI and hot-path VM unsafe untouched)
- `grep -rn "current_security.*\*const"` returns no results
- `SecurityContext` is `Send + Sync` (compile-time verified)
- All existing tests pass: `cargo nextest run -p atlas-runtime`
- Zero clippy warnings: `cargo clippy -p atlas-runtime -- -D warnings`
- Commit: `refactor(runtime): Replace raw *const SecurityContext with Arc<SecurityContext>`
