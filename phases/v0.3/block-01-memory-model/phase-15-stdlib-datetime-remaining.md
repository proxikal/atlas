# Phase 15: Stdlib — DateTime + Remaining Modules

**Block:** 1 (Memory Model)
**Depends on:** Phase 02, Phase 03 complete

---

## Objective

Fix the remaining stdlib modules that have `Arc<Mutex<Vec<Value>>>` or `Arc<Mutex<...>>`
references:
1. `stdlib/datetime.rs` — two functions returning `Arc<Mutex<Vec<Value>>>`
2. Any other stdlib module found to have residual lock sites after Phases 12–14

---

## Current State (verified 2026-02-21)

`stdlib/datetime.rs`:
- Line 1199: function returning `Result<Arc<Mutex<Vec<Value>>>, RuntimeError>`
- Line 1218: function returning `Result<Arc<Mutex<AtlasHashMap>>, RuntimeError>`

Remaining stdlib modules to audit (checked: none have `Arc<Mutex` besides those above):
- `string.rs`, `math.rs`, `io.rs`, `fs.rs`, `path.rs`, `json.rs`, `regex.rs`
- `http.rs`, `process.rs`, `reflect.rs`, `compression/`, `async_io.rs`, `async_primitives.rs`

---

## datetime.rs Fix

### Function at line 1199
This function returns an array of `Value` (likely a list of components: year, month, day, etc.)
```rust
// OLD return type:
Result<Arc<Mutex<Vec<Value>>>, RuntimeError>

// NEW return type:
Result<ValueArray, RuntimeError>
```
Or if this function is a stdlib helper rather than a public function, update the caller
to wrap in `Value::Array(ValueArray::from_vec(...))`.

### Function at line 1218
Returns a HashMap. Update to:
```rust
Result<ValueHashMap, RuntimeError>
```

---

## Residual Module Audit

After Phases 12–14, run:
```bash
cargo check -p atlas-runtime 2>&1 | grep "lock\|Mutex" | grep -v "async_runtime\|security\|OutputWriter\|VecWriter\|FutureState"
```

Any remaining errors not in async_runtime or security are in scope for this phase.

---

## Acceptance Criteria

- [ ] `datetime.rs` functions updated (no `Arc<Mutex<Vec<Value>>>` return)
- [ ] All residual stdlib `.lock().unwrap()` sites on collection types fixed
- [ ] `cargo check -p atlas-runtime` has zero errors related to collection locking
  (excluding intentional infrastructure uses: OutputWriter, async runtime, security)
- [ ] `cargo nextest run -p atlas-runtime -- stdlib::datetime` passes
