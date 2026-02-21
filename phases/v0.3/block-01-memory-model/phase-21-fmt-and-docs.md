# Phase 21: Fmt Clean + Documentation Update

**Block:** 1 (Memory Model)
**Depends on:** Phase 20 complete (clippy clean)

---

## Objective

1. `cargo fmt --check` passes — all code is formatted to rustfmt standard
2. Update inline documentation to reflect new types
3. Update `docs/specification/memory-model.md` to reflect the implementation

---

## Formatting

```bash
cargo fmt --all
cargo fmt --check --all  # verify no remaining unformatted files
```

Rustfmt may reformat some of the new CoW types in `value.rs`. Review the diff to ensure
no logic was accidentally changed (rustfmt shouldn't change logic, but verify).

---

## value.rs Documentation Updates

Update the module doc comment at the top of `value.rs`:

```rust
//! Runtime value representation
//!
//! All Atlas values use **value semantics** — copying a value creates an independent
//! copy. Mutations to a copy never affect the original.
//!
//! ## Value Categories
//!
//! ### Immediate (stack-allocated, always copied)
//! - `Number(f64)` — IEEE 754 double
//! - `Bool(bool)`
//! - `Null`
//!
//! ### Copy-on-write (cheap to copy, independent on mutation)
//! - `String(Arc<String>)` — immutable, shared until reassigned
//! - `Array(ValueArray)` — `Arc<Vec<Value>>` with `Arc::make_mut` CoW
//! - `HashMap(ValueHashMap)` — `Arc<AtlasHashMap>` with CoW
//! - `HashSet(ValueHashSet)`, `Queue(ValueQueue)`, `Stack(ValueStack)` — CoW
//!
//! ### Reference semantics (explicit, opt-in)
//! - `SharedValue(Shared<Box<Value>>)` — `Arc<Mutex<Value>>`, mutations visible to all aliases
//!   Used only when the program explicitly annotates a value as `shared<T>`.
//!
//! ### Identity types (always compared by reference)
//! - `NativeFunction`, `Future`, `TaskHandle`, `ChannelSender`, `ChannelReceiver`, `AsyncMutex`
```

---

## memory-model.md Update

Read `docs/specification/memory-model.md` before editing. Update the "Implementation"
section to document:
1. `ValueArray` = `Arc<Vec<Value>>` with `Arc::make_mut` — implementation decision logged
2. `ValueMap` = `Arc<HashMap<String, Value>>` with `Arc::make_mut`
3. `Shared<T>` = `Arc<Mutex<T>>` — explicit reference semantics
4. Equality semantics table (content vs. reference)

Do NOT change the semantic spec (ownership rules, etc.) — only update the implementation notes.

---

## Acceptance Criteria

- [ ] `cargo fmt --check --all` exits 0
- [ ] `value.rs` module doc comment updated to describe CoW architecture
- [ ] `docs/specification/memory-model.md` implementation section updated
- [ ] No other spec files modified (spec is locked, only implementation notes change)
