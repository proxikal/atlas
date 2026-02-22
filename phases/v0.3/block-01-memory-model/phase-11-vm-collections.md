# Phase 11: VM — Collection Operations (HashMap/Queue/Stack/HashSet)

**Block:** 1 (Memory Model)
**Depends on:** Phase 10 complete (VM array mutations done)

---

## Objective

Fix all HashMap, HashSet, Queue, and Stack operations in the VM. Mirror of Phase 08
for the VM engine. Same API change: `.lock().unwrap()` → `.inner()` / `.inner_mut()`.

---

## Implementation

### VM opcode patterns for collections

```rust
// Map read opcode (GetMapKey):
// OLD: map.lock().unwrap().get(key)
// NEW: map.inner().get(key)

// Map write opcode (SetMapKey):
// OLD:
let mut map = self.pop()?;
if let Value::HashMap(m) = &map { m.lock().unwrap().insert(k, v); }
self.push(map);
// NEW:
let mut map = self.pop()?;
if let Value::HashMap(ref mut m) = map { m.inner_mut().insert(k, v); }
self.push(map);
```

### Queue and Stack specific opcodes

If the VM has dedicated Queue/Stack opcodes (check `vm/dispatch.rs`):
- `QueueEnqueue` / `QueueDequeue` / `QueuePeek`
- `StackPush` / `StackPop` / `StackPeek`

Update each to use `inner_mut()` for mutations, `inner()` for reads.

---

## Parity Requirement

VM collection operations must be identical to interpreter collection operations.
Run the same test programs against both engines and diff the output.

---

## Tests

```atlas
// Map CoW — VM:
let m1 = {"a": 1}
let m2 = m1
m2["b"] = 2
assert(m1.len() == 1)
assert(m2.len() == 2)

// Queue CoW — VM:
let q = Queue.new()
q.enqueue(1)
let q2 = q
q2.dequeue()
assert(q.size() == 1)
assert(q2.size() == 0)
```

---

## Acceptance Criteria

- [ ] All HashMap/HashSet/Queue/Stack opcodes in `vm/` use CoW wrapper API
- [ ] No `.lock().unwrap()` remains on any collection variant in `vm/`
- [ ] Map, Queue, Stack CoW tests pass in VM
- [ ] VM collection output matches interpreter output (parity)
- [ ] `cargo nextest run -p atlas-runtime` passes all VM collection tests
