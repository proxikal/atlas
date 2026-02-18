# Phase Infra-07: Criterion Benchmark Suite

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Infra-01 through 06 complete. Suite green. Fuzz targets run clean.

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -3
ls crates/atlas-runtime/fuzz/fuzz_targets/  # 4 fuzz targets exist
```

---

## Objective
Performance regressions in a compiler are invisible without measurement. Implement a Criterion benchmark suite covering every critical performance path: lexer throughput, parser throughput, typechecker speed, interpreter vs VM execution, stdlib function performance, and memory allocation patterns. These benchmarks establish Atlas's performance baseline, make regressions immediately visible, and provide the data needed to drive optimization work. This is the infrastructure that makes "world class" more than a claim.

## Files

**Create:** `crates/atlas-runtime/benches/lexer.rs` (~100 lines)
**Create:** `crates/atlas-runtime/benches/parser.rs` (~100 lines)
**Create:** `crates/atlas-runtime/benches/typechecker.rs` (~100 lines)
**Create:** `crates/atlas-runtime/benches/interpreter.rs` (~150 lines)
**Create:** `crates/atlas-runtime/benches/vm.rs` (~150 lines)
**Create:** `crates/atlas-runtime/benches/stdlib.rs` (~150 lines)
**Create:** `crates/atlas-runtime/benches/parity.rs` (~100 lines — interpreter vs VM head-to-head)
**Update:** `crates/atlas-runtime/Cargo.toml` (add criterion dev-dependency, bench entries)
**Update:** `.github/workflows/ci.yml` (add benchmark job on main branch pushes)

## Dependencies
- Criterion 0.5+ added as dev-dependency
- Infra-01 through 06 complete
- Representative Atlas programs for benchmarking (reuse corpus from Infra-05 where applicable)

## Implementation

### Step 1: Add Criterion to Cargo.toml
Add `criterion = { version = "0.5", features = ["html_reports"] }` to `[dev-dependencies]`. Add `[[bench]]` entries for each benchmark file with `harness = false`. Criterion replaces libtest for benchmarks — `harness = false` is required.

### Step 2: `benches/lexer.rs` — Lexer throughput
Benchmark the lexer at multiple input sizes. Use `criterion::black_box` to prevent dead code elimination. Benchmarks: tokenize a 100-token program, a 1000-token program, a 10000-token program. Measure tokens-per-second. Include both a simple numeric expression program and a complex real-world program (from the corpus). Throughput metric: report as `Throughput::Bytes` so Criterion shows MB/s.

### Step 3: `benches/parser.rs` — Parser throughput
Benchmark parsing at multiple complexity levels. Use pre-tokenized input to isolate parser cost from lexer cost. Benchmarks: parse a 10-node AST, 100-node AST, 1000-node AST. Include deeply nested expressions, function-heavy programs, and type-annotation-heavy programs as separate benchmark cases.

### Step 4: `benches/typechecker.rs` — Typechecker speed
Benchmark the full frontend pipeline (lex + parse + bind + typecheck) on representative programs. Key benchmarks: a function-heavy program (measures function call resolution), a generics-heavy program (measures constraint solving), a large module (measures scope lookup performance). This tells you how fast the typechecker is on real programs, not micro-benchmarks.

### Step 5: `benches/interpreter.rs` — Interpreter execution
Benchmark the interpreter on a canonical set of programs that stress different execution paths:
- Arithmetic loop: `let sum = 0; let i = 0; while (i < 10000) { sum = sum + i; i++; } sum`
- Recursive fibonacci: `fib(20)` (canonical recursive benchmark)
- String operations: repeated string concatenation and stdlib calls
- Collection operations: array push/pop in a loop, HashMap insert/lookup
- Function calls: tight loop calling a simple function 10000 times
Report each as operations-per-second.

### Step 6: `benches/vm.rs` — VM execution
Identical benchmark programs as `interpreter.rs` but run through the VM (lex → parse → compile → vm.run). This gives you the absolute performance of each engine on the same workload. The VM should be faster than the interpreter on all benchmarks — if it isn't, that is actionable data.

### Step 7: `benches/parity.rs` — Interpreter vs VM head-to-head
Run the same 5 benchmark programs through both engines in the same benchmark file. Report the ratio. This makes interpreter/VM performance divergence immediately visible. A 10x VM speedup over interpreter is expected and healthy. A 1x ratio means the VM optimizer isn't working.

### Step 8: `benches/stdlib.rs` — Stdlib function performance
Benchmark critical stdlib functions that users call frequently:
- String: `len`, `substring`, `split`, `join`, `replace` on large strings
- Array: `push`, `pop`, `slice`, `concat` on large arrays
- Math: `sqrt`, `pow`, `floor` in tight loops
- JSON: `parseJSON` on a complex JSON document, `toJSON` on a complex Value
- HashMap: insert 1000 keys, lookup 1000 keys, iteration

### Step 9: Establish and document baseline
After writing all benchmarks, run the full suite and capture baseline numbers:
```bash
cargo bench -p atlas-runtime 2>&1 | tee benches/baseline.txt
```
Commit `benches/baseline.txt`. This is the performance contract. Every future optimization phase references this file.

### Step 10: CI integration
Add a benchmark job to CI that runs on pushes to main (not PRs — too slow). Use `cargo bench -p atlas-runtime -- --output-format bencher | tee output.txt`. Store results as a CI artifact. Optionally integrate with `github-action-benchmark` for automated trend tracking and PR comments when performance regresses >10%.

### Step 11: Document the benchmark suite
Add `crates/atlas-runtime/benches/README.md`: how to run benchmarks locally, how to interpret Criterion output, how to add a new benchmark, what the baseline numbers mean, and the performance targets for Atlas v0.2.

## Tests
Benchmarks are not tests. They must compile and run without errors. Each benchmark file must run to completion: `cargo bench -p atlas-runtime --bench <name>` succeeds. No assertions except `black_box` usage (required to prevent dead code elimination).

## Integration Points
- Uses: public `Atlas`, `Lexer`, `Parser`, `Compiler`, `VM` APIs
- Requires: `harness = false` in Cargo.toml for all bench entries
- CI: benchmark job on main branch, stored as artifact
- Output: `benches/baseline.txt` committed as performance contract

## Acceptance
- 7 benchmark files compile and run without errors
- `cargo bench -p atlas-runtime` completes without panics
- `benches/baseline.txt` committed with actual numbers from the build machine
- VM benchmarks consistently faster than interpreter benchmarks (if not: document the gap as a known issue)
- CI benchmark job added (runs on main push, not PR gate)
- `crates/atlas-runtime/benches/README.md` documents suite
- `cargo nextest run -p atlas-runtime` still green (benches don't affect test suite)
- All benchmark files pass clippy: `cargo clippy -p atlas-runtime -- -D warnings`
- Clean git commit: `feat(bench): Add Criterion benchmark suite — lexer, parser, typechecker, interpreter, VM, stdlib`
- **Final STATUS.md update:** Mark all 7 infra phases complete. Remove infrastructure blocker note. Restore Next Phase to `phases/interpreter/phase-01-debugger-repl-improvements.md`.
