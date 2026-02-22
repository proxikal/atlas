# Phase 08: Interpreter — Collection Operations (HashMap/Queue/Stack/HashSet)

**Block:** 1 (Memory Model)
**Depends on:** Phase 07 complete (array mutation paths in interpreter done)

---

## Objective

Fix all HashMap, HashSet, Queue, and Stack operations in the interpreter. These collection
types had `Arc<Mutex<AtlasXxx>>` wrappers in Phase 03 — now they use `ValueHashMap`,
`ValueHashSet`, `ValueQueue`, `ValueStack`. Update every interpreter site that accesses
these collections to use the `.inner()` / `.inner_mut()` API instead of `.lock().unwrap()`.

---

## Current State

After Phase 03, `Value::HashMap(ValueHashMap)` etc. are in place. The interpreter
has compile errors wherever it previously called `.lock().unwrap()` on these variants.

Scope: `interpreter/expr.rs`, `interpreter/stmt.rs`, `interpreter/mod.rs`.
Count: check with `cargo check 2>&1 | grep "interpreter/" | grep -v "Array"` after Phase 07.

---

## Implementation

### Read pattern
```rust
// OLD:
if let Value::HashMap(map) = &val {
    let guard = map.lock().unwrap();
    let v = guard.get("key");
}
// NEW:
if let Value::HashMap(map) = &val {
    let v = map.inner().get("key");
}
```

### Mutation pattern
```rust
// OLD:
if let Value::HashMap(map) = &val {
    map.lock().unwrap().insert("key".to_string(), new_val);
}
// NEW (requires &mut):
if let Value::HashMap(map) = &mut val {
    map.inner_mut().insert("key".to_string(), new_val);
}
```

### Queue/Stack specific
Queue has `enqueue`/`dequeue`, Stack has `push`/`pop`. Update accordingly:
```rust
// OLD: queue.lock().unwrap().enqueue(val)
// NEW: queue.inner_mut().enqueue(val)  (on &mut ValueQueue)
```

---

## CoW Note for Collections

The same CoW update loop from Phase 07 applies:
```rust
let mut val = env.get("my_map").clone();
if let Value::HashMap(ref mut map) = val {
    map.inner_mut().insert(key, value); // CoW triggers if shared
}
env.set("my_map", val);
```

---

## Tests

```atlas
// Test: HashMap mutation does not affect alias
let a = {"x": 1}
let b = a
b["y"] = 2
assert(a.keys() == ["x"])
assert(b.keys().len() == 2)
```

```atlas
// Test: Queue dequeue does not affect alias
let q = Queue.new()
q.enqueue(1)
q.enqueue(2)
let q2 = q
q2.dequeue()
assert(q.size() == 2)   // original unaffected
assert(q2.size() == 1)
```

---

## Acceptance Criteria

- [ ] All HashMap/HashSet/Queue/Stack sites in `interpreter/` use CoW wrapper API
- [ ] No `.lock().unwrap()` remains on any collection variant in interpreter
- [ ] Map mutation CoW test passes
- [ ] Queue mutation CoW test passes
- [ ] `cargo nextest run -p atlas-runtime` passes all interpreter collection tests
