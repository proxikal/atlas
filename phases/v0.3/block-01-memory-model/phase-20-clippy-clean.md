# Phase 20: Clippy Clean

**Block:** 1 (Memory Model)
**Depends on:** Phase 19 complete (parity verified)

---

## Objective

Achieve zero Clippy warnings across `atlas-runtime` after all Block 1 changes.
Clippy must pass with `-D warnings` (warnings as errors).

---

## Expected Warning Categories

After the Block 1 migration, common Clippy warnings will be:

### `clippy::arc_with_non_send_sync`
If `ValueArray` or `ValueMap` inner types are not `Send + Sync`, Clippy warns.
Fix: ensure `Value: Send + Sync`. If `Value` already has this bound (check current code),
all CoW wrapper types will inherit it.

### `clippy::needless_pass_by_ref_mut`
Stdlib functions that take `&mut ValueArray` but could take `&ValueArray` for reads.
Fix: change to `&ValueArray` where the mutation is triggered via `Arc::make_mut` internally.

### `clippy::clone_on_ref_ptr`
If code does `arc.clone()` where Arc deref would suffice.
Fix: use `Arc::clone(&arc)` (explicit is clippy-preferred) or restructure.

### `clippy::mutex_atomic`
If any `Mutex<bool>` or `Mutex<usize>` exists in the new code (unlikely but check).

### `clippy::redundant_clone`
After CoW changes, some `.clone()` calls may be provably redundant.
Remove them — they add cognitive overhead and clippy finds them reliably.

---

## Execution

```bash
cargo clippy -p atlas-runtime -- -D warnings 2>&1 | head -50
```

Fix in batches. Run after each batch. Repeat until clean.

```bash
cargo clippy --workspace -- -D warnings
```

Run workspace-wide at the end — ensure atlas-jit and atlas-lsp haven't regressed.

---

## Acceptance Criteria

- [ ] `cargo clippy -p atlas-runtime -- -D warnings` exits 0
- [ ] `cargo clippy --workspace -- -D warnings` exits 0
- [ ] No `#[allow(clippy::...)]` attributes added as suppressions (fix, don't suppress)
- [ ] Exception: existing `#[allow(...)]` that predated Block 1 may remain if unchanged
