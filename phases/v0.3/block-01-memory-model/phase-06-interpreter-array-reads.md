# Phase 06: Interpreter — Array Read Paths

**Block:** 1 (Memory Model)
**Depends on:** Phase 02 complete (Value::Array is ValueArray)

---

## Objective

Fix all array *read* operations in the interpreter (`interpreter/expr.rs`,
`interpreter/stmt.rs`, `interpreter/mod.rs`). Read operations are any site that:
- Pattern-matches on `Value::Array(arr)` and then calls `.lock().unwrap()` to read
- Calls `.get()`, `.len()`, `.iter()`, `.is_empty()`, indexing

These are the simpler cases — no CoW trigger needed. Remove `.lock().unwrap()`, call
the `ValueArray` read API directly.

---

## Current State (verified 2026-02-21)

Interpreter has 35 `Value::Array`/`.lock()` call sites total across 3 files.
Read sites are the majority. Mutation sites are covered in Phase 07.

---

## Approach

### Pattern to replace

Old (read pattern):
```rust
if let Value::Array(arr) = &value {
    let arr = arr.lock().unwrap();
    let len = arr.len();
    let first = arr.get(0);
    // ...
}
```

New:
```rust
if let Value::Array(arr) = &value {
    let len = arr.len();
    let first = arr.get(0);
    // ...
}
```

### Common read patterns in interpreter

1. **Array index access** (`a[i]`):
   ```rust
   // Old
   let arr = arr.lock().unwrap();
   let val = arr.get(index).cloned().unwrap_or(Value::Null);
   // New
   let val = arr.get(index).cloned().unwrap_or(Value::Null);
   ```

2. **Array length** (used in loops, conditions):
   ```rust
   // Old: arr.lock().unwrap().len()
   // New: arr.len()
   ```

3. **Array iteration** (for-in loops):
   ```rust
   // Old
   let arr = arr.lock().unwrap();
   for item in arr.iter() { ... }
   // New
   for item in arr.iter() { ... }
   ```

4. **Spread/destructuring**:
   ```rust
   // Old: let elements: Vec<Value> = arr.lock().unwrap().iter().cloned().collect()
   // New: let elements: Vec<Value> = arr.iter().cloned().collect()
   ```

5. **Pattern matching on array contents** (match arms):
   ```rust
   // Old: match arr.lock().unwrap().as_slice() { ... }
   // New: match arr.as_slice() { ... }
   ```

---

## Execution Steps

1. Run `cargo check -p atlas-runtime 2>&1 | grep "interpreter/" | grep "lock\|Mutex"` to get exact list
2. Fix each error — read-only sites first (safe, mechanical, no behavior change)
3. After each file, run `cargo check -p atlas-runtime` to verify no new errors introduced
4. Do NOT fix mutation sites in this phase (Phase 07 handles those)

---

## How to Distinguish Read vs. Mutation Sites

A site is a **read** if it only calls:
- `.len()`, `.is_empty()`, `.get()`, `.iter()`, `.as_slice()`, `.contains()`, indexing

A site is a **mutation** if it calls:
- `.push()`, `.pop()`, `.insert()`, `.remove()`, `.set()`, `.clear()`, `.extend()`
- Or reassigns elements via `arr[i] = value`

Mutation sites: leave the `.lock().unwrap()` in place with a `// TODO Phase 07` comment
so the compiler doesn't complain but the intent is clear.

**Wait** — after Phase 02, `Arc<Mutex<Vec<Value>>>` is gone. There is no `.lock()` to
keep. Instead, any mutation site that previously got a `MutexGuard` needs to be updated
to call `ValueArray`'s mutation API. For this phase:
- If it's a read → fix it now
- If it's a mutation → fix the pattern-match syntax to use `Value::Array(ref mut arr)` +
  leave a `// TODO Phase 07: mutation` comment. The code won't compile, but at least
  the shape is right.

---

## Tests

After this phase, the following interpreter tests must pass:
- All array read tests: index access, length, iteration, spread
- `cargo nextest run -p atlas-runtime` — expected to fail on mutation tests, pass on reads

Note: full test suite won't pass until Phase 07 (mutations) and Phase 15 (stdlib) complete.
Track which test categories pass after each phase.

---

## Acceptance Criteria

- [ ] All read-only array sites in `interpreter/` updated to `ValueArray` API
- [ ] No `.lock().unwrap()` remains on `Value::Array` in `interpreter/` for read sites
- [ ] Mutation sites are marked and shaped correctly for Phase 07
- [ ] `cargo check` reports no errors in files touched by this phase (only mutation TODOs remain)
- [ ] Array index, length, iteration tests pass in interpreter
