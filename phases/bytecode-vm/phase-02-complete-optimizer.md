# Phase 02: Complete Bytecode Optimizer

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Optimizer hooks must exist from v0.1.

**Verification:**
```bash
grep -n "optimizer\|TODO.*optim" crates/atlas-runtime/src/compiler/mod.rs
ls crates/atlas-runtime/src/optimizer/ 2>/dev/null || echo "Need to create"
grep -n "pub struct Chunk\|pub bytecode" crates/atlas-runtime/src/bytecode/mod.rs
```

**What's needed:**
- Compiler has hooks/TODOs for optimizer
- Bytecode structure allows analysis and modification
- Constants pool accessible

**If missing:** Check v0.1 phase bytecode-vm/phase-05

---

## Objective
Implement complete bytecode optimizer with constant folding, dead code elimination, and peephole optimizations - reducing bytecode size and improving runtime performance.

## Files
**Create:** `crates/atlas-runtime/src/optimizer/mod.rs` (~1200 lines)
**Create:** `crates/atlas-runtime/src/optimizer/constant_folding.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/optimizer/dead_code.rs` (~350 lines)
**Create:** `crates/atlas-runtime/src/optimizer/peephole.rs` (~450 lines)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (~50 lines integrate)
**Update:** `crates/atlas-runtime/src/lib.rs` (add optimizer module)
**Tests:** `crates/atlas-runtime/tests/optimizer_tests.rs` (~800 lines)
**Tests:** `crates/atlas-runtime/tests/optimizer_integration_tests.rs` (~400 lines)

## Dependencies
- v0.1 complete with compiler, bytecode, VM
- Bytecode structure supports modification
- Validator from phase 01 to verify optimized bytecode

## Implementation

### Constant Folding
Evaluate constant expressions at compile time. Scan for patterns like Constant-Constant-Add and fold to single constant. Handle all arithmetic operators. Include unary operations. Fold nested expressions. Add computed constants to pool.

### Dead Code Elimination
Remove unreachable code after returns and unconditional jumps. Build reachability graph with BFS from entry. Mark unreachable instructions. Remove dead code updating jump targets accordingly. Maintain jump target consistency.

### Peephole Optimizations
Local instruction pattern simplifications. Eliminate push-pop pairs. Collapse jump chains. Remove noop instructions like dup-pop. Simplify jump-to-next-instruction. Multiple passes until no changes. Strength reduction for future phases.

### Integration
Integrate optimizer in compiler pipeline. Run multiple passes - constant folding, dead code elimination, peephole optimization. Validate optimized bytecode. Track optimization statistics. Make optimization optional with flag.

## Tests (TDD - Use rstest)

**Optimizer tests:**
1. Constant folding - arithmetic, nested expressions
2. Dead code elimination - after returns, jumps
3. Peephole opts - push-pop, jump chains, noops
4. Integration - optimized equals unoptimized results
5. Bytecode size reduction measurements
6. Performance improvements via benchmarks
7. Edge cases - empty programs, already optimal
8. Validation - optimized bytecode is valid

**Minimum test count:** 100 tests (60 unit, 40 integration)

## Integration Points
- Uses: Bytecode, Opcode from bytecode/mod.rs
- Uses: Value from value.rs
- Uses: Validator from phase 01
- Updates: Compiler to integrate optimizer
- Creates: Complete optimizer with 3 passes
- Output: Smaller faster bytecode

## Acceptance
- Constant folding works on all operators
- Dead code eliminated correctly
- Peephole optimizations applied
- Optimized code semantically equivalent
- Bytecode size reduced 20-40% typical
- Performance improved 10-30%
- 100+ tests pass
- Optimized bytecode passes validator
- No clippy warnings
- cargo test passes
