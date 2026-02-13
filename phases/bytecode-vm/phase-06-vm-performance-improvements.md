# Phase 06: VM Performance Improvements

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Profiler must be complete to identify bottlenecks.

**Verification:**
```bash
ls crates/atlas-runtime/src/profiler/mod.rs
cargo test profiler
cargo run --release -- profile examples/fibonacci.at
```

**What's needed:**
- Profiler from phase 03 working
- Baseline performance measurements
- VM can be profiled to identify slow paths

**If missing:** Complete phase bytecode-vm/phase-03 first

---

## Objective
Optimize VM performance based on profiler data with improved instruction dispatch, constant pool access, stack operations, and reduced allocation overhead - targeting 30-50% overall performance improvement across arithmetic, function calls, loops, and array operations.

## Files
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~300 lines optimized)
**Create:** `crates/atlas-runtime/src/vm/dispatch.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/vm/stack.rs` (~200 lines optimized)
**Create:** `benches/vm_performance_benches.rs` (~600 lines)
**Create:** `PERFORMANCE_REPORT.md` (document improvements)

## Dependencies
- Profiler from phase 03
- VM from v0.1 complete
- Criterion benchmarking framework

## Implementation

### Baseline Benchmarking
Establish performance baselines before optimization. Create comprehensive benchmark suite with Criterion. Benchmark arithmetic operations constant folding and chained operations. Benchmark function calls at various recursion depths. Benchmark loops with different iteration counts. Benchmark array operations push, map, filter. Record baseline measurements for comparison.

### Instruction Dispatch Optimization
Optimize hot dispatch loop identified by profiler. Replace match-based dispatch with direct threaded code using jump tables. Generate static dispatch table mapping opcode index to handler function. Use inline always for dispatch-critical functions. Implement opcode discriminant method for fast indexing. Separate execution handlers for each opcode type. Minimize branch mispredictions in hot path.

### Constant Pool Access Optimization
Reduce overhead of constant access operations. Use reference counting Rc for large values strings and arrays to avoid deep cloning. Keep small values number and bool unboxed. Make constant loading cheap by sharing ownership via Rc. Maintain constant pool as vector for fast indexed access.

### Stack Operations Optimization
Optimize stack manipulation hot path. Pre-allocate stack with reasonable capacity 256 slots to avoid reallocation. Inline push and pop operations. Use fixed-size array for call frames instead of Vec to eliminate allocation. Implement fast frame management with frame count tracking. Keep stack operations bounds-checked but optimized.

### Allocation Overhead Reduction
Minimize heap allocations during execution. Reuse string buffer in VM for temporary string operations. Pre-allocate buffers for common operations. Use object pools for frequently created temporary values when beneficial. Profile allocation patterns and eliminate unnecessary clones.

### Numeric Operations Optimization
Optimize arithmetic-heavy code paths. Keep numeric values unboxed in hot loops when types known. Use raw f64 arithmetic without Value boxing. Implement specialized fast paths for pure numeric operations. Consider separate number stack for tight loops in future.

### Performance Measurement
Benchmark all optimizations against baseline. Run full benchmark suite with optimized VM. Compare results to baseline measurements. Generate performance report documenting improvements per category. Verify 30-50% improvement target met. Document optimization techniques applied. Profile optimized VM to ensure no new bottlenecks introduced.

## Tests (TDD - Use rstest)

**Correctness tests:**
All existing tests must pass - optimizations change performance not semantics. Run full test suite. Verify identical results for all programs. Check edge cases still handled correctly.

**Performance tests:**
1. Arithmetic operations 30%+ faster
2. Function calls 20%+ faster
3. Loops with iterations 40%+ faster
4. Array operations 25%+ faster
5. Memory usage unchanged or reduced
6. No performance regressions in any area
7. Fibonacci benchmark under threshold
8. Large program performance acceptable

**Minimum test count:** 40 performance regression tests

## Integration Points
- Uses: VM from vm/mod.rs
- Uses: Profiler from phase 03 for validation
- Uses: Criterion for benchmarking
- Updates: VM dispatch mechanism
- Updates: Stack operations
- Updates: Constant pool access
- Creates: Comprehensive benchmarks
- Creates: Performance report
- Output: 30-50% faster VM

## Acceptance
- All existing tests pass no semantic changes
- Arithmetic ops 30%+ faster
- Function calls 20%+ faster
- Loops 40%+ faster
- Array ops 25%+ faster
- Overall improvement 30-50%
- Memory usage stable or better
- Performance report complete
- Benchmarks document improvements
- 40+ performance tests pass
- cargo bench runs successfully
- cargo test passes all tests
- No clippy warnings
- Optimizations commented clearly
