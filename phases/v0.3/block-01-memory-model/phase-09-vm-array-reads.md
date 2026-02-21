# Phase 09: VM — Array Read Paths

**Block:** 1 (Memory Model)
**Depends on:** Phase 02 complete (Value::Array is ValueArray)

---

## Objective

Fix all array *read* operations in the VM (`vm/dispatch.rs`, `vm/mod.rs`, `vm/frame.rs`).
Mirror of Phase 06 for the VM engine. Same mechanical change: remove `.lock().unwrap()`,
call `ValueArray` read API directly.

**Parity requirement:** VM array reads must produce identical output to interpreter array
reads for all inputs. Any divergence is a parity break and is BLOCKING.

---

## Current State (verified 2026-02-21)

VM has 29 `Value::Array`/`.lock()` call sites across:
- `vm/dispatch.rs` — opcode execution, most sites here
- `vm/mod.rs` — main execution loop, stack management
- `vm/frame.rs` — call frame management (verify if array-related)

---

## VM-Specific Context

The VM operates on a stack of `Value`. Array operations correspond to opcodes:
- `GetIndex` — read `array[i]`
- `GetLength` — `array.len()`
- `Iterate` / `ForEach` — array iteration
- `Spread` — array spread into function call args

For each opcode, the VM pops operands from the stack, performs the operation, pushes result.

### Example: GetIndex opcode
```rust
// OLD:
Opcode::GetIndex => {
    let index = self.pop()?.as_number()? as usize;
    let array = self.pop()?;
    if let Value::Array(arr) = &array {
        let guard = arr.lock().unwrap();
        let val = guard.get(index).cloned().unwrap_or(Value::Null);
        self.push(val);
    }
}

// NEW:
Opcode::GetIndex => {
    let index = self.pop()?.as_number()? as usize;
    let array = self.pop()?;
    if let Value::Array(arr) = &array {
        let val = arr.get(index).cloned().unwrap_or(Value::Null);
        self.push(val);
    }
}
```

---

## Execution Steps

1. `cargo check -p atlas-runtime 2>&1 | grep "vm/" | grep "lock\|Mutex"` — get exact sites
2. For each error: identify opcode, determine read vs. mutation
3. Fix reads (this phase), mark mutations for Phase 10
4. Verify parity: each read opcode should produce same result as interpreter equivalent

---

## Tests

Run interpreter vs. VM parity check for array reads:
```atlas
// These must produce identical output in both engines:
let arr = [10, 20, 30]
print(arr[0])        // 10
print(arr[2])        // 30
print(arr.len())     // 3
```

Test via:
```
cargo nextest run -p atlas-runtime -- array_read
```

---

## Acceptance Criteria

- [ ] All array read opcodes in `vm/` updated to `ValueArray` API
- [ ] No `.lock().unwrap()` remains on read sites in `vm/`
- [ ] VM array index, length, iteration produce identical results to interpreter
- [ ] `cargo check` for `vm/` files reports no errors on read-only sites
