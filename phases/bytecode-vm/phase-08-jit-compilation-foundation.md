# Phase 08: JIT Compilation Foundation

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** VM with profiler and optimizer must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/vm/mod.rs
ls crates/atlas-runtime/src/profiler/mod.rs
ls crates/atlas-runtime/src/optimizer/mod.rs
cargo test vm
cargo test profiler
```

**What's needed:**
- VM from v0.1
- Profiler from bytecode-vm/phase-03
- Optimizer from bytecode-vm/phase-02
- cranelift-jit or similar JIT backend

**If missing:** Complete bytecode-vm phases 02-03 first

---

## Objective
Implement JIT compilation foundation generating native machine code for hot functions identified by profiler - providing 5-10x performance improvements for computation-heavy code bringing Atlas performance to native levels.

## Files
**Create:** `crates/atlas-jit/` (new crate ~2000 lines total)
**Create:** `crates/atlas-jit/src/lib.rs` (~200 lines)
**Create:** `crates/atlas-jit/src/codegen.rs` (~800 lines)
**Create:** `crates/atlas-jit/src/backend.rs` (~600 lines)
**Create:** `crates/atlas-jit/src/cache.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~300 lines JIT integration)
**Update:** `Cargo.toml` (add cranelift dependencies)
**Create:** `docs/jit.md` (~800 lines)
**Tests:** `crates/atlas-jit/tests/jit_tests.rs` (~600 lines)

## Dependencies
- cranelift-jit for code generation
- VM with execution infrastructure
- Profiler for hotspot identification
- Optimizer for IR preparation

## Implementation

### Hotspot Detection
Use profiler to identify hot functions. Execution count threshold for JIT compilation. Time-based profiling for hot loops. Mark functions for JIT compilation. Adaptive compilation based on runtime behavior. Compile most-executed code first. Balance compilation time vs speedup.

### IR Translation
Translate bytecode to Cranelift IR. Map bytecode ops to IR instructions. Handle control flow in IR. Translate values to IR types. Function calling convention. Stack frame layout in JIT. Local variable allocation. Register allocation via Cranelift.

### Native Code Generation
Compile IR to machine code using Cranelift. Target native architecture x86-64, aarch64. Apply optimizations in Cranelift. Generate efficient calling sequences. Handle runtime calls to Atlas runtime. Link with runtime functions. Memory management integration. Exception handling propagation.

### Code Cache Management
Cache compiled native code. Lookup function in cache. Invalidate on recompilation. Memory limits for code cache. Evict cold code when full. Code versioning for invalidation. Executable memory management. Security: W^X protection.

### VM Integration
Replace interpreted functions with JIT code. Function dispatch checks for JIT version. Fall back to interpreter if needed. Mixed mode execution. Seamless transition between modes. Debug mode disables JIT. Tiered compilation: interpret then JIT.

### Performance Measurement
Benchmark JIT vs interpreter. Measure compilation overhead. Track speedup per function. Adaptive thresholds based on benefit. Profile-guided JIT decisions. Optimize for real-world workloads.

## Tests (TDD - Use rstest)
1. Detect hot function
2. Translate simple bytecode to IR
3. Compile IR to native code
4. Execute JIT compiled function
5. JIT result matches interpreter
6. Function dispatch to JIT
7. Fallback to interpreter
8. Code cache hit
9. Code cache eviction
10. Performance improvement measured

**Minimum test count:** 50 tests

## Acceptance
- Hot functions identified
- Bytecode translates to IR
- Native code generated
- JIT code executes correctly
- Results match interpreter
- Code cache functional
- 5-10x speedup on benchmarks
- 50+ tests pass
- Documentation with performance guide
- cargo test passes
