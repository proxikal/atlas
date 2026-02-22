# VM Performance Report — Phase 06

## Optimization Summary

### 1. Instruction Dispatch Optimization
**File:** `vm/dispatch.rs` (~100 lines)

- **Static lookup table** (`OPCODE_TABLE[256]`): O(1) opcode decoding via direct array indexing
- Replaces `Opcode::try_from()` match-based dispatch
- Eliminates branch mispredictions in the hot decode path
- `decode_opcode()` is `#[inline(always)]` for zero-overhead dispatch

### 2. Hot Path Inlining
**File:** `vm/mod.rs`

All dispatch-critical functions annotated `#[inline(always)]`:
- `push()`, `pop()`, `peek()` — stack operations
- `pop_number()` — numeric type extraction
- `binary_numeric_op()` — arithmetic fast path
- `read_opcode()`, `read_u8()`, `read_u16()`, `read_i16()` — instruction reading
- `current_frame()` — frame access

### 3. Unsafe Bounds Check Elimination
**File:** `vm/mod.rs`

Removed redundant bounds checks in hot paths where VM invariants guarantee safety:
- `pop()`: uses `unwrap_unchecked()` (stack is never empty when pop is called)
- `peek()`: uses `get_unchecked()` (distance is always valid)
- `current_frame()`: uses `unwrap_unchecked()` (frames always non-empty during execution)
- `read_opcode()`/`read_u8()`/`read_u16()`: uses `get_unchecked()` after bounds-checking ip
- Explicit bounds check on `ip` in `read_opcode()` preserved for safety

### 4. Stack Operations Optimization
**File:** `vm/mod.rs`

- Increased default stack capacity: 256 → 1024 slots (reduces reallocation for deep programs)
- Stack cleanup in `Return` opcode: replaced `while pop` loop with `truncate()` (O(1) vs O(n) for stack frames)
- Stack cleanup in `vm_call_function_value`: same `truncate()` optimization

### 5. String Concatenation Optimization
**File:** `vm/mod.rs`

- Added reusable `string_buffer` field to VM struct (pre-allocated with 256 bytes)
- String concatenation (`Add` for strings) reuses the buffer: `clear()` + `push_str()` pattern
- Avoids `format!()` allocation overhead for repeated concatenation

### 6. Constant Pool Access
**File:** `vm/mod.rs`

- Constants already use `Arc<String>` for strings (shared ownership, cheap cloning)
- Number and bool constants are unboxed (Copy types, no allocation)
- Indexed access via `Vec<Value>` (already O(1))

## Optimization Techniques Applied

| Category | Technique | Impact |
|----------|-----------|--------|
| Dispatch | Static lookup table | Eliminates match branching |
| Dispatch | `#[inline(always)]` | Eliminates function call overhead |
| Stack | `unsafe` unchecked access | Eliminates bounds checks |
| Stack | `truncate()` cleanup | O(1) vs O(n) frame cleanup |
| Stack | Larger pre-allocation | Fewer reallocations |
| Strings | Buffer reuse | Fewer heap allocations |
| Memory | `unwrap_unchecked()` | Removes panic branch |

## Benchmark Suite

**File:** `benches/vm_performance_benches.rs` (~290 lines)

25 benchmarks across 9 categories:
- Arithmetic (6): add, sub, mul, div, mixed, chained
- Functions (4): simple call, recursive (fib), nested, multi-arg
- Loops (4): counting, accumulation, nested, conditionals
- Arrays (3): creation, index access, set index
- Variables (1): local variable access
- Comparison (2): comparison ops, equality
- Strings (1): concatenation
- Stack (2): heavy expressions, deep nesting
- Scaling (2): loop scaling, function call scaling (parameterized)

Run: `cargo bench --bench vm_performance_benches`

## Test Suite

**File:** `tests/vm_performance_tests.rs` (~440 lines)

48 regression tests across 8 categories:
- Arithmetic correctness (8 tests)
- Function call correctness (8 tests)
- Loop correctness (8 tests)
- Array correctness (6 tests)
- Stack correctness (4 tests)
- Comparison correctness (4 tests)
- String correctness (2 tests)
- Dispatch table correctness (4 tests)
- Performance smoke tests (4 tests)

## Files Modified/Created

| File | Action | Lines | Purpose |
|------|--------|-------|---------|
| `vm/dispatch.rs` | Created | ~100 | Static opcode dispatch table |
| `vm/mod.rs` | Updated | ~3720 | Inlining, unsafe fast paths, truncate, string buffer |
| `benches/vm_performance_benches.rs` | Created | ~290 | Criterion benchmark suite |
| `tests/vm_performance_tests.rs` | Created | ~440 | 48 regression tests |
| `PERFORMANCE_REPORT.md` | Created | — | This document |
| `Cargo.toml` | Updated | +4 | Bench target entry |
