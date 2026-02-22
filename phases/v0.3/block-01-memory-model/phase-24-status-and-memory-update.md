# Phase 24: STATUS.md + Memory Update

**Block:** 1 (Memory Model)
**Depends on:** Phase 23 complete (all acceptance criteria verified)

---

## Objective

Update project status tracking and agent memory to reflect Block 1 completion.
Prepare for Block 2 scaffolding.

---

## STATUS.md Updates

Read `STATUS.md` before editing. Update:

1. **Mark Block 1 as complete** in the block table
2. **Update current phase** to "Block 2 scaffolding" or "Awaiting Block 2 start"
3. **Update test count** — record the actual test count after Block 1
4. **Update the "No Arc<Mutex<Vec<Value>>>" milestone** as achieved
5. **Record Block 1 actual phase count** — how many phases did Block 1 actually take vs. estimated 25?

---

## Memory Updates (Claude Auto-Memory)

### Update `decisions/runtime.md`

Add Block 1 implementation decisions:
```markdown
## DR-B01-01: ValueArray Representation
**Decision:** Arc<Vec<Value>> with Arc::make_mut for CoW
**Rationale:** Zero new deps, proven Swift/Rust pattern, in-place mutation for exclusive owners
**Date:** 2026-02-21 (locked pre-scaffolding)

## DR-B01-02: ValueMap Representation
**Decision:** Arc<HashMap<String, Value>> with Arc::make_mut
**Rationale:** Consistent with ValueArray, simple ownership story

## DR-B01-03: Shared<T> Implementation
**Decision:** Arc<Mutex<T>> for explicit reference semantics
**Rationale:** Arc<Mutex> is the correct Rust pattern for shared mutable state

## DR-B01-04: Equality Semantics
**Decision:**
- CoW types (Array, HashMap, etc.): content equality
- Reference types (NativeFunction, async runtime): pointer equality
- Shared<T>: pointer equality (reference semantics by design)
- Regex: pattern string equality
```

### Update `patterns.md`

Add CoW patterns to avoid repeating the Phase 07 analysis:
```markdown
## CoW Array Update Loop (stdlib functions)
When a stdlib function receives &[Value] and needs to mutate an array:
1. Clone the Value: `let mut val = args[0].clone()`  (cheap — refcount bump)
2. Match mut: `if let Value::Array(ref mut arr) = val`
3. Call mutation: `arr.push(x)` (CoW triggers if refcount > 1)
4. Return modified value: `Ok(val)` (caller stores back to variable)
DO NOT try to get &mut from &[Value] directly — it won't work.

## Method Call Return Value Rule (interpreter/VM)
After calling a mutating stdlib method on a collection, store the return
value back to the receiver variable. Mutating methods always return the
modified collection (not Null).
Mutating array methods: push, pop, insert, remove, set, sort, reverse, fill, extend, truncate
Non-mutating: len, get, includes, find, join, slice, map, filter, reduce (return new value)
```

---

## Acceptance Criteria

- [ ] `STATUS.md` updated with Block 1 completion, actual phase count, test count
- [ ] `decisions/runtime.md` updated with DR-B01-01 through DR-B01-04
- [ ] `patterns.md` updated with CoW update loop and method return value patterns
- [ ] Memory files within size limits (check MEMORY.md ≤ 50 lines, others ≤ 150)
