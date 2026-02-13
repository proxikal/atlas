# Phase 07: Bytecode-VM Integration Tests

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** All previous bytecode-vm phases must be complete.

**Verification:**
```bash
ls crates/atlas-runtime/src/{optimizer,profiler,debugger}/mod.rs
cargo test optimizer profiler debugger vm
ls benches/vm_performance_benches.rs
```

**What's needed:**
- Phases 01-06 complete optimizer profiler debugger performance
- All unit tests passing
- VM optimizations applied

**If missing:** Complete previous bytecode-vm phases first

---

## Objective
Comprehensive integration testing of all bytecode-VM features verifying optimizer, profiler, and debugger work together correctly with complex end-to-end programs and zero regressions from v0.1.

## Files
**Create:** `crates/atlas-runtime/tests/vm_integration_tests.rs` (~1000 lines)
**Create:** `crates/atlas-runtime/tests/vm_complex_programs.rs` (~800 lines)
**Create:** `crates/atlas-runtime/tests/vm_regression_tests.rs` (~600 lines)
**Create:** `examples/` directory with 20+ Atlas programs
**Create:** `docs/vm-architecture.md` (~600 lines)
**Update:** `TESTING.md` (add VM testing guide)

## Dependencies
- All bytecode-vm phases 01-06 complete
- All optimizations applied and tested
- Profiler profiler debugger optimizer all functional

## Implementation

### Feature Integration Testing
Test all features working together correctly. Verify optimizer with debugger optimized code remains debuggable with preserved debug info and accurate breakpoints. Test profiler with optimizer showing instruction count reduction and performance improvements. Verify debugger step operations work through optimized bytecode maintaining source line accuracy. Test all features combined optimizer profiler debugger and performance improvements running simultaneously. Ensure feature combinations do not interfere with each other.

### Complex Program Testing
Test real-world complex programs exercising all VM capabilities. Create recursive algorithms fibonacci factorial ackermann quicksort with deep call stacks. Build closure-heavy programs with multiple nested closures and captured variable mutations. Test deeply nested data structures objects within arrays within objects multiple levels deep. Implement data processing programs JSON parsing CSV processing with stdlib integration. Create programs combining multiple stdlib functions in pipelines. Test class-based OOP patterns if supported. Verify async simulation patterns using closures.

### Regression Testing
Ensure zero regressions from v0.1. Load all v0.1 test programs and verify identical results. Test interpreter-VM parity maintained across all test cases. Verify performance meets or exceeds v0.1 benchmarks. Check all previously passing tests still pass. Ensure semantic behavior unchanged. Validate edge cases still handled correctly.

### Example Program Creation
Create 20+ working example programs demonstrating VM capabilities. Programs should be realistic and non-trivial. Cover different domains algorithms, data processing, simulations. Include programs testing specific features recursion, closures, stdlib usage. Examples serve as integration tests and documentation. Each example should be runnable and produce correct output.

### Performance Verification
Verify VM performance meets targets from phase 06. Run performance regression tests ensuring no slowdowns. Benchmark complex programs checking acceptable execution time. Verify profiler reports show optimization improvements. Test that optimized programs run faster than unoptimized. Ensure memory usage reasonable under load.

### Documentation
Write comprehensive VM architecture documentation. Document bytecode format and opcode encoding. Explain execution model and stack management. Describe optimization techniques constant folding dead code peephole. Document profiler usage and report interpretation. Explain debugger protocol and integration. Include performance benchmarks and trade-offs. Provide testing guide in TESTING.md.

## Tests (TDD - Use rstest)

**Feature integration tests:**
1. Optimizer and debugger together
2. Profiler and optimizer combination
3. Debugger stepping through optimized code
4. All features enabled simultaneously
5. Feature flags working correctly

**Complex program tests:**
1. Recursive algorithms deep stacks
2. Closure-heavy programs with captures
3. Deeply nested data structures
4. JSON and CSV processing
5. Stdlib function pipelines
6. OOP pattern programs
7. Data transformation programs

**Regression tests:**
1. All v0.1 programs identical results
2. Interpreter-VM parity maintained
3. Performance meets benchmarks
4. Edge cases handled correctly

**Example programs:**
20+ programs covering algorithms, data processing, simulations

**Minimum test count:** 200 integration tests

## Integration Points
- Uses: All bytecode-vm components optimizer profiler debugger VM
- Uses: All stdlib functions for complex programs
- Uses: Interpreter for parity testing
- Creates: Comprehensive integration test suite
- Creates: Example program collection
- Creates: VM architecture documentation
- Output: Fully tested documented production-ready VM

## Acceptance
- 200+ integration tests pass
- 20+ example programs run correctly
- Zero regressions from v0.1
- All v0.1 tests pass
- 100% interpreter-VM parity maintained
- Complex programs work recursion closures stdlib
- All features work together optimizer debugger profiler
- Performance meets phase 06 targets
- Documentation complete and accurate
- cargo test passes all 1591+ tests
- No clippy warnings
- VM ready for production usage
