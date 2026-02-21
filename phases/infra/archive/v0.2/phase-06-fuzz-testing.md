# Phase Infra-06: Fuzz Testing

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Infra-01 through 05 complete. Suite green.

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -3
cargo +nightly --version  # nightly required for cargo-fuzz
```

**Install if missing:**
```bash
cargo install cargo-fuzz
rustup toolchain install nightly
```

---

## Objective
A compiler that panics on malformed input is unprofessional and potentially unsafe for embedding. Implement fuzz testing across all Atlas frontend entry points: lexer, parser, typechecker, and the full interpreter pipeline. Any input — no matter how malformed — must produce a clean error, never a panic or crash. This is the standard every production compiler meets. cargo-fuzz with libFuzzer is the Rust standard for this.

## Files

**Create:** `crates/atlas-runtime/fuzz/` directory (cargo-fuzz workspace)
**Create:** `crates/atlas-runtime/fuzz/Cargo.toml` (~20 lines)
**Create:** `crates/atlas-runtime/fuzz/fuzz_targets/fuzz_lexer.rs` (~40 lines)
**Create:** `crates/atlas-runtime/fuzz/fuzz_targets/fuzz_parser.rs` (~40 lines)
**Create:** `crates/atlas-runtime/fuzz/fuzz_targets/fuzz_typechecker.rs` (~50 lines)
**Create:** `crates/atlas-runtime/fuzz/fuzz_targets/fuzz_eval.rs` (~60 lines)
**Create:** `crates/atlas-runtime/fuzz/corpus/` (seed corpus for each target)
**Update:** `.github/workflows/ci.yml` (add fuzz job, time-limited)

## Dependencies
- cargo-fuzz installed
- nightly Rust toolchain
- Infra-01 through 05 complete (clean test suite)

## Implementation

### Step 1: Initialize cargo-fuzz workspace
Create `crates/atlas-runtime/fuzz/Cargo.toml` as a cargo-fuzz workspace. It declares the atlas-runtime dependency. Each fuzz target is a separate binary. The fuzzer runs under nightly Rust but the targets are written in stable-compatible code. The fuzz directory is gitignored for fuzzer output (`fuzz/artifacts/`, `fuzz/corpus/*/`) — only the seed corpus is committed.

### Step 2: Fuzz target — `fuzz_lexer`
Input: arbitrary bytes as a `&str` (libFuzzer provides UTF-8 via `std::str::from_utf8`). Entry: `Lexer::new(input).tokenize()`. Contract: must never panic, must always return `(Vec<Token>, Vec<Diagnostic>)`. Any panic is a fuzzer-found bug. The lexer is the simplest target — start here to establish the pattern.

### Step 3: Fuzz target — `fuzz_parser`
Input: arbitrary string. Entry: lex then parse. `Lexer::new(input).tokenize()` → `Parser::new(tokens).parse()`. Contract: must never panic at any stage. The parser must handle every possible token stream gracefully. Known challenging cases: deeply nested expressions (stack overflow risk), extremely long token sequences, empty input, single-character inputs.

### Step 4: Fuzz target — `fuzz_typechecker`
Input: arbitrary string. Entry: lex, parse, bind, typecheck. Contract: no panics through the full frontend pipeline. The typechecker is the most complex target — it operates on an AST that may be arbitrarily malformed. This target will find the most interesting bugs.

### Step 5: Fuzz target — `fuzz_eval`
Input: arbitrary string. Entry: full `Atlas::new().eval(input)`. Contract: must return `Ok(Value)` or `Err(diagnostics)` — never panic. This exercises the complete pipeline including interpreter. This is the most valuable target for an embedding use case: a host application cannot have the Atlas runtime crash it.

### Step 6: Seed corpus
For each fuzz target, create a small seed corpus of interesting inputs in `fuzz/corpus/<target>/`. Seeds are files containing representative valid and invalid Atlas programs. Good seeds dramatically accelerate fuzzer coverage. Include: empty string, single characters, valid programs, programs with common error patterns, deeply nested structures, very long identifiers, unicode content, binary data as strings.

### Step 7: Run initial fuzz campaigns
Before committing, run each target for at least 60 seconds to shake out immediate panics:
```bash
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=60
cargo +nightly fuzz run fuzz_parser -- -max_total_time=60
cargo +nightly fuzz run fuzz_typechecker -- -max_total_time=60
cargo +nightly fuzz run fuzz_eval -- -max_total_time=60
```
If any panics are found, fix them before proceeding. Each panic found is a real bug — fix it at the source, not in the fuzz target.

### Step 8: CI integration
Add a fuzz job to `.github/workflows/ci.yml` that runs each target for 120 seconds (`-max_total_time=120`). This job runs on a schedule (nightly) not on every PR — fuzzing is not fast enough for PR gates but must run regularly. Use `cargo +nightly fuzz run <target> -- -max_total_time=120 -error_exitcode=1` so CI fails on any crash.

### Step 9: Document the fuzz setup
Add `crates/atlas-runtime/fuzz/README.md`: how to run fuzzing locally, how to reproduce a crash from an artifact file, how to add a new fuzz target, how to extend the seed corpus. One page.

## Tests
Fuzz testing is not unit tested — it IS the test. Acceptance is: all 4 targets run for 60+ seconds without panics. CI job added. Documentation present.

## Integration Points
- Requires nightly Rust toolchain (fuzz build only — production code stays stable)
- Runs against the public `Atlas`, `Lexer`, `Parser` APIs
- Any panics found must be fixed in `atlas-runtime/src/` — the fuzz targets themselves are never patched to hide crashes

## Acceptance
- 4 fuzz targets exist and compile under nightly
- Each target ran for minimum 60 seconds without crashes before committing
- Seed corpus present for each target (minimum 5 seeds each)
- CI fuzz job added, runs nightly for 120 seconds per target
- `crates/atlas-runtime/fuzz/README.md` documents usage
- Any panics discovered during initial runs are FIXED in the runtime (not suppressed)
- `cargo nextest run -p atlas-runtime` still green (fuzz targets don't affect normal test suite)
- Clean git commit: `feat(fuzz): Add cargo-fuzz targets for lexer, parser, typechecker, eval`
