# Atlas JIT Status

**Status:** Foundation complete — not yet wired to production execution path
**Last updated:** v0.2 completion sprint
**Crate:** `crates/atlas-jit/`
**Backend:** Cranelift (native code generation)

---

## Summary

The JIT crate compiles arithmetic-only Atlas bytecode functions to native machine code via
Cranelift. It supports numeric computations with local variables and comparisons. It does NOT
support control flow (jumps, calls), global variables, or collection/object opcodes.

The crate is NOT wired to the VM's execution path — functions always run interpreted. Wiring
requires control flow opcode support (v0.3 work).

---

## Supported Opcodes

These opcodes compile and execute correctly in the JIT:

| Opcode | Hex | Description |
|--------|-----|-------------|
| `Constant` | `0x01` | Load numeric constant (f64 only; strings/other types bail out) |
| `True` | `0x03` | Push 1.0 |
| `False` | `0x04` | Push 0.0 |
| `Null` | `0x02` | Push 0.0 |
| `Add` | `0x20` | f64 addition |
| `Sub` | `0x21` | f64 subtraction |
| `Mul` | `0x22` | f64 multiplication |
| `Div` | `0x23` | f64 division |
| `Mod` | `0x24` | f64 modulo (floor-based) |
| `Negate` | `0x25` | f64 negation |
| `Equal` | `0x30` | f64 equality → 1.0 or 0.0 |
| `NotEqual` | `0x31` | f64 inequality → 1.0 or 0.0 |
| `Less` | `0x32` | f64 less-than → 1.0 or 0.0 |
| `LessEqual` | `0x33` | f64 less-than-or-equal → 1.0 or 0.0 |
| `Greater` | `0x34` | f64 greater-than → 1.0 or 0.0 |
| `GreaterEqual` | `0x35` | f64 greater-than-or-equal → 1.0 or 0.0 |
| `Not` | `0x40` | Boolean NOT (treats 0.0 as false) → 1.0 or 0.0 |
| `GetLocal` | `0x10` | Read local variable (f64, up to 64 locals) |
| `SetLocal` | `0x11` | Write local variable (f64, up to 64 locals) |
| `Pop` | `0x80` | Discard top of stack |
| `Dup` | `0x81` | Duplicate top of stack |
| `Return` | `0x61` | End of function — return top of stack |
| `Halt` | `0xFF` | End of program — treated as Return |

---

## Unsupported Opcodes

These opcodes return `JitError::UnsupportedOpcode` and cause JIT compilation to bail out.
The VM's interpreter handles them instead.

| Opcode | Hex | Reason |
|--------|-----|--------|
| `GetGlobal` | `0x12` | Globals require runtime hash map lookup (not native-addressable) |
| `SetGlobal` | `0x13` | Same as GetGlobal |
| `Jump` | `0x50` | Control flow requires Cranelift block graph; not yet implemented |
| `JumpIfFalse` | `0x51` | Same as Jump |
| `Loop` | `0x52` | Same as Jump |
| `Call` | `0x60` | Nested calls require indirect function dispatch |
| `And` | `0x41` | Short-circuit logic requires control flow blocks |
| `Or` | `0x42` | Same as And |
| `Array` | `0x70` | Arrays are heap-allocated `Arc<Mutex<Vec<Value>>>` — not f64 |
| `GetIndex` | `0x71` | Array indexing requires GC-managed heap access |
| `SetIndex` | `0x72` | Same as GetIndex |
| `IsOptionSome` | `0x90` | Option type is not representable as f64 |
| `IsOptionNone` | `0x91` | Same as IsOptionSome |
| `IsResultOk` | `0x92` | Result type is not representable as f64 |
| `IsResultErr` | `0x93` | Same as IsResultOk |
| `ExtractOptionValue` | `0x94` | Requires heap Value unwrapping |
| `ExtractResultValue` | `0x95` | Same as ExtractOptionValue |
| `IsArray` | `0x96` | Type tag check requires runtime Value enum |
| `GetArrayLen` | `0x97` | Array metadata requires heap access |

---

## What Works Today

Arithmetic-only functions with local variables can be JIT-compiled and produce correct results.
Example: a function that computes `(x + y) * z - w` with parameters bound as locals.

See `tests/jit_tests.rs` for the full test suite (100+ tests).

---

## Integration Requirements (v0.3)

To wire the JIT to the VM hotspot profiler:

1. Implement `Jump`, `JumpIfFalse`, `Loop` in `codegen.rs` using Cranelift block graph
2. Implement `Call` in `codegen.rs` with indirect function pointer dispatch
3. Add `GetGlobal`/`SetGlobal` with the VM's global value array (passed as a pointer)
4. Wire `hotspot.rs` to the VM's profiler threshold (suggested: 1000 executions)
5. Replace interpreter loop for hot functions with JIT-compiled native function pointer
6. Add `And`/`Or` short-circuit logic using conditional blocks

**Prerequisite:** Control flow opcode support must come before globals or calls, because
function bodies always contain at least an implicit Return opcode which is already handled.
The next required opcode is `JumpIfFalse` (for `if` expressions inside hot loops).

---

## Architecture Notes

- **Backend:** `backend.rs` — Cranelift `SimpleJITModule`, compiles IR to native code
- **Code Cache:** `cache.rs` — Fixed-size cache mapping function offset → native code pointer
- **Hotspot Tracker:** `hotspot.rs` — Counts function invocations, triggers compilation at threshold
- **IR Translator:** `codegen.rs` — Translates Atlas bytecode to Cranelift IR (f64-only model)
- **Engine:** `lib.rs` — `JitEngine` integrates all four components

The JIT uses an f64-only value model. All Atlas `number` values are native f64. Boolean
results from comparisons are converted to 1.0/0.0. This is sufficient for arithmetic-heavy
numeric loops which are the primary JIT target.
