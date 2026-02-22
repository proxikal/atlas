# Phase 17: Full Build Clean

**Block:** 1 (Memory Model)
**Depends on:** Phases 01–16 all complete

---

## Objective

After all migration phases (01–16), achieve a clean build across the entire workspace:
`cargo build --workspace` passes with zero errors. Fix any remaining compile errors not
addressed by earlier phases.

---

## What to Expect

After Phases 01–16, the most likely residual errors are:
1. Edge cases in stdlib modules not fully covered by Phases 12–15
2. Trait bound issues (e.g., new CoW types need `Clone + Debug + PartialEq` in places
   where the compiler requires them)
3. `Display` or `Debug` formatting for new types in error messages
4. Any `From`/`Into` conversions that reference old `Arc<Mutex<Vec<Value>>>` types

---

## Execution Steps

1. `cargo build --workspace 2>&1 | head -50` — see first batch of errors
2. Fix errors in batches by file
3. Repeat until clean
4. `cargo build --workspace` — zero errors required before proceeding

---

## Common Residual Fixes

### Missing `PartialEq` on inner collection types

If `AtlasHashMap`, `AtlasHashSet`, `AtlasQueue`, `AtlasStack` don't derive `PartialEq`,
the `ValueXxx::PartialEq` impls from Phase 03 won't compile.

Fix: add `#[derive(PartialEq)]` to each inner type, or implement it manually.

### `Arc<Mutex<Vec<Value>>>` still referenced in type aliases or docs

Search:
```bash
grep -rn "Arc<Mutex<Vec<Value" crates/atlas-runtime/src/ --include="*.rs"
```
Should return zero results (excluding comments). Fix any remaining.

### AtlasJIT crate

`atlas-jit` crate references `Value` from `atlas-runtime`. If `atlas-jit` directly
pattern-matches on `Value::Array(arr)` expecting `Arc<Mutex<Vec<Value>>>`, it will fail.

Check:
```bash
grep -rn "Value::Array" crates/atlas-jit/src/ --include="*.rs"
```
Update any sites found.

### atlas-lsp crate

Similar audit for `atlas-lsp`.

---

## Acceptance Criteria

- [ ] `cargo build --workspace` exits with code 0
- [ ] Zero compile errors across all crates
- [ ] Zero `Arc<Mutex<Vec<Value>>>` in production code (excluding intentional uses in async_runtime/security)
- [ ] Zero `Arc<Mutex<AtlasHashMap/HashSet/Queue/Stack>>` in production code
