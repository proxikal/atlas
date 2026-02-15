# Phase 13: Performance Benchmarking Framework

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Test infrastructure from v0.1 and profiler from bytecode-vm/phase-03.

**Verification Steps:**
1. Check v0.1 test infrastructure:
   ```bash
   cargo test --all 2>&1 | grep "test result"
   ```

2. Check STATUS.md: Bytecode-VM section
   - If phase-03 (Profiler) is ✅: Can use profiler integration
   - If phase-03 is ⬜: Benchmarking works without profiler (add integration later)

3. Verify profiler exists (if phase-03 complete):
   ```bash
   ls crates/atlas-runtime/src/profiler/mod.rs 2>/dev/null || echo "Profiler not yet implemented"
   ```

4. Check if criterion already added:
   ```bash
   grep -n "criterion" Cargo.toml 2>/dev/null || echo "Will add criterion"
   ```

**Expected from v0.1:**
- Test infrastructure with 1,391+ tests
- Test utilities (rstest, insta, proptest)
- Benchmark harness capability

**Bytecode-VM/phase-03 status:**
- If ✅ complete: Profiler hooks available for integration
- If ⬜ incomplete: Benchmarking still works, add profiler integration later

**Decision Tree:**

a) If v0.1 tests exist (1,391+ tests pass):
   → Proceed with phase-13
   → Add criterion for Rust-side benchmarks
   → Create Atlas benchmark syntax and runner

b) If v0.1 tests missing or failing:
   → ERROR: v0.1 must be complete
   → Fix v0.1 test infrastructure
   → Verify tests pass
   → Then proceed with phase-13

c) If bytecode-vm/phase-03 complete (profiler exists):
   → Great! Include profiler integration in benchmarks
   → Hook benchmarks into profiler for detailed analysis

d) If bytecode-vm/phase-03 incomplete (no profiler):
   → This is OK - benchmarking doesn't strictly require profiler
   → Implement benchmarking framework
   → Add profiler integration later when phase-03 complete
   → Note in phase: "Profiler integration deferred to bytecode-vm/phase-03 completion"

**No user questions needed:** Test infrastructure is verifiable. Profiler is optional enhancement, not blocker.

---

## Objective
Implement comprehensive performance benchmarking framework measuring Atlas code performance, tracking regressions, comparing interpreter vs VM, and providing statistical analysis - ensuring Atlas remains fast and competitive.

## Files
**Create:** `crates/atlas-bench/` (new crate ~1000 lines total)
**Create:** `crates/atlas-bench/src/lib.rs` (~200 lines)
**Create:** `crates/atlas-bench/src/runner.rs` (~400 lines)
**Create:** `crates/atlas-bench/src/reporter.rs` (~300 lines)
**Create:** `crates/atlas-bench/src/harness.rs` (~100 lines)
**Update:** `crates/atlas-cli/src/commands/bench.rs` (~300 lines)
**Create:** `benches/` (benchmark suite directory)
**Create:** `benches/stdlib_benchmarks.atl` (~500 lines)
**Create:** `benches/vm_benchmarks.atl` (~400 lines)
**Create:** `benches/compiler_benchmarks.rs` (~300 lines)
**Create:** `docs/benchmarking.md` (~600 lines)
**Tests:** `crates/atlas-bench/tests/bench_tests.rs` (~400 lines)

## Dependencies
- Criterion for Rust-side benchmarks
- Statistical analysis libraries
- Profiler integration from phase-03
- CLI framework for bench command
- JSON output for tooling

## Implementation

### Benchmark Definition Format
Define Atlas benchmark syntax using bench keyword or annotations. Benchmark function with setup and teardown. Iterations parameter for sample size. Warmup iterations to stabilize. Benchmark groups for related tests. Parameter sweeps for different inputs. Baseline comparisons against previous runs. Custom measurement units. Metadata tags for filtering. Documentation comments for benchmarks.

### Benchmark Runner Infrastructure
Create BenchmarkRunner executing benchmarks consistently. Load benchmark files from benches directory. Parse benchmark definitions. Execute setup before each run. Time benchmark iterations with high precision. Run warmup iterations first. Collect sample data for statistics. Execute teardown after benchmarking. Isolate benchmarks preventing interference. Parallel execution option for speed. Sequential execution for stability.

### Statistical Analysis
Analyze benchmark results statistically. Compute mean execution time. Compute median for robustness. Calculate standard deviation. Determine confidence intervals. Detect outliers and filter. Compare against baseline with t-test. Report regression or improvement. Percentage change from baseline. Statistical significance testing. Handle high variance scenarios.

### Performance Comparison
Compare performance across dimensions. Interpreter vs VM execution for same code. Different optimization levels. Release vs debug builds. Different input sizes. Before and after code changes. Across platforms if available. Stdlib function alternatives. Historical trend analysis.

### Benchmark Reporting
Generate comprehensive benchmark reports. Console output with color coding. HTML reports with charts. JSON output for tooling integration. Markdown reports for documentation. CSV export for spreadsheets. Comparison tables showing improvements or regressions. Flame graphs for profiler integration. Historical trend graphs. Export to CI artifacts.

### Regression Detection
Automatically detect performance regressions. Compare against baseline thresholds. Alert on significant slowdowns. Track regression history. Bisect to find regressing commit (future). Configurable regression thresholds. False positive filtering. Integration with CI pipeline. Block merges on regression if configured.

### Standard Benchmark Suite
Create comprehensive benchmark suite. Stdlib function benchmarks for all APIs. Compiler benchmarks parsing, type checking, codegen. VM benchmarks bytecode execution, optimization. Interpreter benchmarks AST walking. Microbenchmarks for hot paths. Real-world scenario benchmarks. Memory allocation benchmarks. Startup time benchmarks. Incremental compilation benchmarks.

### Integration with Profiler
Connect benchmarks with profiler from phase-03. Run benchmarks under profiler. Collect profiling data during benchmark. Identify hotspots in benchmarked code. Optimization recommendations. Profile-guided optimization data. Cache behavior analysis. Instruction-level profiling if available.

## Tests (TDD - Use rstest)

**Benchmark runner tests:**
1. Load benchmark from file
2. Execute simple benchmark
3. Run warmup iterations
4. Collect timing samples
5. Setup and teardown execution
6. Benchmark group execution
7. Parameter sweep benchmarks
8. Parallel benchmark execution
9. Benchmark timeout handling
10. Error in benchmark handling

**Statistical analysis tests:**
1. Compute mean from samples
2. Compute median
3. Calculate standard deviation
4. Confidence interval computation
5. Outlier detection
6. Baseline comparison
7. Regression detection
8. Statistical significance

**Comparison tests:**
1. Compare interpreter vs VM
2. Compare optimization levels
3. Compare different inputs
4. Historical comparison
5. Platform comparison

**Reporting tests:**
1. Console output format
2. HTML report generation
3. JSON output structure
4. Markdown report format
5. Comparison table format
6. Trend graph data
7. Export to CI artifacts

**Regression detection tests:**
1. Detect slowdown regression
2. Detect improvement
3. Threshold configuration
4. Historical tracking
5. False positive filtering

**Standard suite tests:**
1. Stdlib benchmarks exist
2. Compiler benchmarks run
3. VM benchmarks run
4. Interpreter benchmarks run
5. Microbenchmarks stable
6. Real-world scenarios

**Profiler integration tests:**
1. Run benchmark with profiler
2. Collect profiling data
3. Hotspot identification
4. Profile report generation

**Minimum test count:** 70 tests

## Integration Points
- Uses: Profiler from bytecode-vm/phase-03
- Uses: Interpreter and VM from v0.1
- Uses: CLI framework from v0.1
- Creates: atlas-bench crate
- Creates: Benchmark runner infrastructure
- Creates: Standard benchmark suite
- Output: Performance tracking and regression detection

## Acceptance
- Benchmark definition format works
- Run benchmarks from CLI atlas bench
- Statistical analysis provides accurate metrics
- Compare interpreter vs VM performance
- Detect regressions automatically
- Generate reports in multiple formats
- Standard benchmark suite comprehensive
- Profiler integration functional
- 70+ tests pass
- Benchmark suite runs in CI
- Performance baseline established
- Documentation with writing benchmarks guide
- No clippy warnings
- cargo test passes
