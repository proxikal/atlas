# Phase 10: VM — Array Mutation Paths + CoW Trigger

**Block:** 1 (Memory Model)
**Depends on:** Phase 09 complete (VM array reads done)

---

## Objective

Fix all array *mutation* operations in the VM. This is the VM-side mirror of Phase 07.
The VM's mutation model differs from the interpreter: the VM operates on a stack, and
values on the stack may be aliases of values in other stack frames or globals.

---

## VM Mutation Model

The VM stack holds `Value` objects. When a VM opcode mutates an array, it must:
1. Pop the array from the stack (or peek at it if SetIndex needs the array back)
2. Get a `&mut ValueArray`
3. Call the mutation method (triggers CoW if shared)
4. Push the result back if needed

### Critical: SetIndex opcode

```rust
// OLD:
Opcode::SetIndex => {
    let value = self.pop()?;
    let index = self.pop()?.as_number()? as usize;
    let array = self.pop()?;
    if let Value::Array(arr) = &array {
        arr.lock().unwrap()[index] = value; // mutates in-place through Arc
    }
    self.push(array);
}

// NEW (CoW-correct):
Opcode::SetIndex => {
    let value = self.pop()?;
    let index = self.pop()?.as_number()? as usize;
    let mut array = self.pop()?;   // move out of stack
    if let Value::Array(ref mut arr) = array {
        arr.set(index, value);     // CoW triggers if arr is shared
    }
    self.push(array);              // push back (may be a new allocation)
}
```

### ArrayPush opcode

```rust
// NEW:
Opcode::ArrayPush => {
    let item = self.pop()?;
    let mut array = self.pop()?;
    if let Value::Array(ref mut arr) = array {
        arr.push(item);
    }
    self.push(array);
}
```

### Important: local variable mutation via SetLocal

When `SetLocal` stores a new value into a stack slot, the previous value is dropped.
If the previous value was an `Arc<Vec<Value>>` with refcount > 1, the drop decrements
refcount but doesn't free memory (other aliases exist). This is correct and automatic.

---

## Parity Requirement

Every mutation opcode must produce the same post-mutation state as the interpreter:
- After `arr.push(x)` in VM: `arr` has length N+1
- After `arr[i] = v` in VM: `arr[i] == v`
- After either: aliased copies (other stack slots pointing to same original Arc) are unaffected

---

## Tests

```atlas
// VM mutation CoW - must match interpreter behavior exactly:
let a = [1, 2, 3]
let b = a
b.push(4)
assert(a.len() == 3)    // a unaffected
assert(b.len() == 4)

// VM index assignment CoW:
let x = [10, 20, 30]
let y = x
y[1] = 99
assert(x[1] == 20)      // x unaffected
assert(y[1] == 99)
```

Run interpreter and VM on same program, compare output. Zero divergence allowed.

---

## Acceptance Criteria

- [ ] All array mutation opcodes in `vm/` use `ValueArray` mutation API
- [ ] No `.lock().unwrap()` remains on mutation sites in `vm/`
- [ ] VM CoW push test matches interpreter
- [ ] VM CoW index-assignment test matches interpreter
- [ ] `cargo nextest run -p atlas-runtime` passes all VM array mutation tests
