# Phase 02: Interpreter Performance & Integration Tests

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** All Correctness phases (01–07) complete. Interpreter-01 complete. VM complete.

**Verification:**
```bash
# Correctness phases must be done first — parity must be proven correct before measuring it
grep -rn "current_security.*\*const" crates/atlas-runtime/src/  # must return 0 results
grep "Item::Import" crates/atlas-runtime/src/interpreter/mod.rs | grep -c "skip"  # must be 0
cargo nextest run -p atlas-runtime -E 'test(parity)' 2>&1 | tail -3  # must be all green
ls crates/atlas-runtime/src/interpreter/debugger.rs
cargo nextest run -p atlas-runtime -E 'test(interpreter_debug_tests)'
cargo nextest run -p atlas-runtime -E 'test(repl_tests)'
cargo bench | grep vm
```

**What's needed:**
- ALL Correctness phases complete (01–07): raw pointer eliminated, builtin registry unified,
  Value::Builtin variant added, callback parity fixed, method dispatch unified,
  immutability enforced, imports wired
- Phase interpreter/phase-01 complete with debugger and REPL improvements
- VM from v0.1 complete for parity comparison
- Profiler from bytecode-vm/phase-03 for performance measurement
- All unit tests passing, all parity tests green

**Why correctness phases must precede this phase:**
Parity verification in this phase is only meaningful if the known parity breaks have been
fixed. Running parity benchmarks against a broken baseline produces misleading results and
may mask real divergence behind the noise of known failures.

**If missing:** Complete Correctness phases 01–07, then Interpreter-01

---

## Objective
Optimize interpreter performance targeting 30%+ improvement through caching, reduced cloning, and optimized operations plus comprehensive integration testing ensuring interpreter, debugger, REPL work together correctly with full interpreter-VM parity verification.

## Files
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~200 lines optimized)
**Create:** `crates/atlas-runtime/src/interpreter/cache.rs` (~300 lines)
**Create:** `benches/interpreter_benches.rs` (~400 lines)
**Update:** `crates/atlas-runtime/tests/interpreter.rs` (add integration + parity tests to existing file)
**Create:** `docs/interpreter-status.md` (~300 lines)
**Update:** `STATUS.md` (~50 lines mark interpreter complete)

## Dependencies
- Phase interpreter/phase-01 complete
- VM complete for parity testing
- Profiler for performance measurement
- Benchmark framework Criterion

## Implementation

### Environment Lookup Caching
Optimize variable lookups with caching. Implement environment lookup cache mapping variable names to locations. Cache successful lookups for repeated access. Invalidate cache on environment changes. Use simple hash map for cache storage. Measure cache hit rate ensuring high percentage. Profile before and after to verify improvement. Target 20-30% speedup on variable-heavy code.

### Value Cloning Reduction
Reduce unnecessary value cloning using reference counting. Replace owned values with Rc-wrapped values for large data strings, arrays, objects. Share values across scopes when possible. Clone only when mutation needed. Implement copy-on-write for mutable operations. Measure allocation reduction. Profile memory usage improvements. Target 30-40% fewer allocations.

### Function Call Optimization
Optimize function call overhead. Cache function lookups similar to variables. Inline small functions when beneficial. Reduce call frame allocation overhead. Reuse call frame structures. Optimize argument passing. Profile function call performance. Target 15-20% speedup on function-heavy code.

### Baseline Benchmarking
Create comprehensive benchmark suite with Criterion. Benchmark arithmetic operations. Benchmark function calls with various depths. Benchmark loops with different iteration counts. Benchmark array operations. Benchmark object field access. Benchmark recursive algorithms. Record baseline before optimizations. Compare after optimizations showing improvements.

### Integration Testing
Test all interpreter features working together. Test debugger with complex programs. Test REPL with multi-line input and commands. Test interpreter with enhanced error messages and warnings. Test type integration with REPL. Test all stdlib functions in interpreter mode. Test closure and scope handling. Test error recovery and reporting.

### Interpreter-VM Parity Testing
Verify identical behavior between interpreter and VM. Create comprehensive parity test suite. Run same programs in both modes. Compare results ensuring exact matches. Test all language features for parity. Test edge cases and error conditions. Test stdlib function behavior. Test debugger protocol compatibility. Report any divergence as critical bugs.

### Performance Verification
Measure and validate performance improvements. Run benchmarks comparing before and after optimizations. Verify 30%+ overall improvement target. Identify remaining bottlenecks for future work. Compare interpreter performance to VM showing acceptable gap. Document performance characteristics. Profile with real-world programs.

### Interpreter Status Documentation
Write comprehensive interpreter status report. Document implementation status of both interpreter phases. List all optimizations applied with measured improvements. Describe debugger capabilities and parity with VM. Describe REPL enhancements and commands. List verification checklist with test coverage. Document known limitations. Propose future enhancements. Conclude Interpreter is complete and production-ready.

### STATUS.md Update
Update STATUS.md marking Interpreter category as 2/2 complete with both phases checked off. Update overall progress percentage.

## Tests (TDD - Use rstest)

**Performance tests:**
1. Environment lookup performance
2. Value cloning reduction verification
3. Function call overhead measurement
4. Benchmark suite correctness
5. Performance regression detection

**Integration tests:**
1. Debugger with complex programs
2. REPL full workflow sessions
3. Enhanced errors in interpreter
4. Warnings in interpreter mode
5. Type integration REPL commands
6. All stdlib functions work
7. Closures and scopes correct
8. Error recovery robust

**Parity tests:**
1. Arithmetic operations identical
2. Function calls identical
3. Loops identical
4. Arrays identical
5. Objects identical
6. Closures identical
7. Error behavior identical
8. Edge cases identical

**Minimum test count:** 100 tests (30 performance, 40 integration, 30 parity)

## Integration Points
- Uses: Interpreter from phase 01
- Uses: VM for parity comparison
- Uses: Profiler for measurement
- Updates: Interpreter with optimizations
- Creates: Comprehensive benchmarks
- Creates: Parity test suite
- Updates: STATUS.md and interpreter-status.md
- Output: Production-ready optimized interpreter

## Acceptance
- 30%+ overall performance improvement measured
- Environment lookup caching provides 20%+ speedup
- Value cloning reduced by 30%+ allocations
- Function calls 15%+ faster
- Benchmark suite comprehensive
- 100+ tests pass 30 performance 40 integration 30 parity
- All interpreter features work together
- Debugger works with complex programs
- REPL fully functional with all commands
- Interpreter-VM parity verified 100%
- No behavioral divergence between modes
- Documentation complete interpreter-status.md
- STATUS.md updated Interpreter marked 2/2 complete
- No clippy warnings
- cargo nextest run -p atlas-runtime passes
- cargo bench shows improvements
- Interpreter production-ready for v0.2
