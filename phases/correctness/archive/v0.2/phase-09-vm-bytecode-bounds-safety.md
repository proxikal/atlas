# Phase Correctness-09: VM Bytecode Bounds Safety

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-08 complete. Build passes. Suite green.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
cargo nextest run -p atlas-runtime 2>&1 | tail -3
```

---

## Objective

The VM's hot-path functions `read_u8()` and `read_u16()` use `unsafe { *self.bytecode.instructions.get_unchecked(self.ip) }` without bounds checking. `read_opcode()` checks `self.ip >= self.bytecode.instructions.len()` for the opcode byte, but the operand bytes that follow are read unchecked. If the bytecode is truncated (e.g., a `LoadConst` instruction at the last byte, or malformed bytecode from deserialization), `read_u8`/`read_u16` will read out of bounds — undefined behavior.

This is only safe if **every bytecode execution path** is guaranteed to have valid bytecode. Currently there is a bytecode validator in the codebase, but it is not mandatory before execution. Any path that skips validation (direct `VM::run()` calls, deserialized bytecode, test harnesses) can trigger UB.

The fix: either (a) make the validator mandatory and provably run before every `VM::run()`, or (b) add bounds checks to `read_u8`/`read_u16` that are optimized away in practice (the branch predictor will make them effectively free on valid bytecode). Option (b) is the standard approach — WASM runtimes, JVMs, and Lua's VM all validate operand bounds.

---

## Files Changed

- `crates/atlas-runtime/src/vm/mod.rs` — add bounds checking to `read_u8` and `read_u16`, returning `Result`
- `crates/atlas-runtime/src/vm/dispatch.rs` — update dispatch if it calls `read_u8`/`read_u16` directly

---

## Dependencies

- Correctness-08 complete
- No other phases are prerequisites

---

## Implementation

### Step 1: Audit all unsafe in vm/mod.rs

Search for every `unsafe` block in the VM. Categorize each as:
- **Justified and safe:** e.g., `push`/`pop` if stack depth is provably correct
- **Needs bounds check:** `read_u8`, `read_u16`, any `get_unchecked`

Document findings before changing anything.

### Step 2: Add bounds checking to read_u8 and read_u16

Change `read_u8`:
```rust
#[inline(always)]
fn read_u8(&mut self) -> Result<u8, RuntimeError> {
    if self.ip >= self.bytecode.instructions.len() {
        return Err(RuntimeError::UnknownOpcode {
            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
        });
    }
    let byte = self.bytecode.instructions[self.ip];
    self.ip += 1;
    Ok(byte)
}
```

Change `read_u16`:
```rust
#[inline(always)]
fn read_u16(&mut self) -> Result<u16, RuntimeError> {
    if self.ip + 1 >= self.bytecode.instructions.len() {
        return Err(RuntimeError::UnknownOpcode {
            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
        });
    }
    let hi = self.bytecode.instructions[self.ip] as u16;
    let lo = self.bytecode.instructions[self.ip + 1] as u16;
    self.ip += 2;
    Ok((hi << 8) | lo)
}
```

The `#[inline(always)]` ensures the branch predictor handles the bounds check with zero overhead on valid bytecode. This matches what WASM runtimes do.

### Step 3: Propagate Result through call sites

Every call site of `read_u8()` and `read_u16()` must now handle the `Result`. They're already inside `run()` which returns `Result`, so add `?` at each call site:

```rust
// Before: let index = self.read_u16() as usize;
// After:  let index = self.read_u16()? as usize;
```

Count all call sites before starting. Ensure every one is updated.

### Step 4: Audit remaining unsafe blocks

After the bounds-check changes, the remaining `unsafe` blocks should be:
- Stack `push`/`pop` — document the invariant that guarantees safety
- `current_frame` — document that frames vec is never empty during execution
- FFI-related unsafe — handled by Phase 08

For each remaining `unsafe`, add a `// SAFETY:` comment explaining the invariant. If any `unsafe` cannot be justified, convert it to safe code.

### Step 5: Add truncated bytecode tests

```rust
#[test]
fn test_truncated_bytecode_load_const() {
    // LoadConst needs 2 operand bytes — provide only 1
    let mut bytecode = Bytecode::new();
    bytecode.instructions = vec![Opcode::LoadConst as u8, 0x00]; // missing second byte
    let result = VM::new(bytecode).run();
    assert!(result.is_err()); // must not crash/UB
}

#[test]
fn test_truncated_bytecode_empty() {
    let bytecode = Bytecode::new();
    let result = VM::new(bytecode).run();
    // Should either return Ok(Null) or a clean error — not UB
    assert!(result.is_ok() || result.is_err());
}
```

### Step 6: Performance verification

Run the benchmark suite (if available) before and after. The bounds checks should show < 1% overhead on valid bytecode due to branch prediction. If benchmarks don't exist yet, note this in the commit message — the benchmark phase (Infra-07) will verify later.

---

## Tests

- `test_truncated_bytecode_load_const` — missing operand bytes produce error, not crash
- `test_truncated_bytecode_jump` — truncated jump instruction produces error
- `test_truncated_bytecode_empty` — empty bytecode handled cleanly
- `test_valid_bytecode_unchanged` — all existing VM tests pass identically (behavior unchanged for valid bytecode)
- All existing tests pass: `cargo nextest run -p atlas-runtime`
- Zero clippy warnings

---

## Acceptance

- Zero `get_unchecked` calls in `read_u8` and `read_u16`
- `read_u8()` and `read_u16()` return `Result<_, RuntimeError>`
- All call sites propagate the error with `?`
- Every remaining `unsafe` block in `vm/mod.rs` has a `// SAFETY:` comment
- Truncated bytecode tests pass (error, not crash)
- All existing tests pass unchanged
- Zero clippy warnings: `cargo clippy -p atlas-runtime -- -D warnings`
- Commit: `fix(vm): Add bounds checking to read_u8/read_u16 — eliminate UB on malformed bytecode`
