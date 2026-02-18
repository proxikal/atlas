# Phase Infra-05: File-Based Test Corpus

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Infra-01 through 04 complete. Suite green, zero bare `#[ignore]`.

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -3
grep -c "^#\[ignore\]$" crates/atlas-runtime/tests/*.rs  # all zeros
```

---

## Objective
Implement a file-based test corpus: tests written as real `.atlas` source files, not Rust strings. This is how every serious compiler tests — rustc has `tests/ui/`, clang has `test/`, Go has `test/`. It is the single most important missing piece in Atlas's test infrastructure. When a test is written in the language itself, it documents the language, is readable without knowing Rust, and survives refactors of the test harness. Build the corpus, the Rust harness that runs it, and seed it with 50+ real Atlas programs covering pass, fail, and warning cases.

## Files

**Create:** `crates/atlas-runtime/tests/corpus.rs` (~200 lines — Rust harness, test runner)
**Create:** `crates/atlas-runtime/tests/corpus/` directory tree:
```
tests/corpus/
├── pass/          # Atlas programs that should execute and produce expected output
├── fail/          # Atlas programs that should produce specific compile/runtime errors
└── warn/          # Atlas programs that should produce specific warnings
```
Each test case is a pair: `foo.atlas` + `foo.stdout` (expected stdout) or `foo.stderr` (expected errors).

**Seed:** 50+ `.atlas` corpus files across all three categories (detailed below)

## Dependencies
- Infra-01 through 04 complete
- The `Atlas::eval()` API and CLI runner must produce stable, deterministic output
- insta already present for snapshot comparison

## Implementation

### Step 1: Design the corpus harness (`tests/corpus.rs`)
The harness is a single Rust test file that discovers and runs all `.atlas` files. Use `std::fs::read_dir` to walk `tests/corpus/pass/`, `tests/corpus/fail/`, `tests/corpus/warn/`. For each `.atlas` file: read it, run it via the Atlas runtime API, capture output. Compare actual output against the expected `foo.stdout` or `foo.stderr` file using exact string comparison. If no `.stdout`/`.stderr` companion file exists, the harness fails with a clear message: "Missing expected output file — create `foo.stdout` with the expected output." Generate one rstest case per discovered `.atlas` file so nextest shows individual test names. Test names should be the corpus file path (e.g., `corpus::pass::arithmetic::fib_recursive`).

### Step 2: Pass corpus — programs that run correctly (30+ files)
Create `tests/corpus/pass/` with subdirectories by domain. Each file is a real Atlas program. Each has a companion `.stdout` with exact expected output. Minimum coverage:
- `arithmetic/` — basic ops, precedence, edge cases (5 files)
- `strings/` — concatenation, stdlib string functions, unicode (5 files)
- `functions/` — closures, recursion, first-class functions, generics (5 files)
- `types/` — Option, Result, union types, type guards, pattern matching (5 files)
- `collections/` — HashMap, HashSet, Queue, Stack, arrays (5 files)
- `modules/` — import, export, module-level state (3 files)
- `stdlib/` — json, file I/O, math, datetime, regex (5 files)
- `programs/` — real-world complete programs (3+ files, e.g., a CSV parser written in Atlas)

### Step 3: Fail corpus — programs that should produce errors (15+ files)
Create `tests/corpus/fail/` with programs that intentionally contain errors. Each has a companion `.stderr` with the exact expected error message. The harness verifies: Atlas returns an error AND the error message matches the snapshot. This is how you guarantee error message quality over time — they become contract tests. Coverage:
- `type_errors/` — type mismatches, wrong argument count, unknown field (5 files)
- `syntax_errors/` — malformed programs, missing braces, invalid tokens (5 files)
- `runtime_errors/` — division by zero, index out of bounds, null dereference (5 files)

### Step 4: Warn corpus — programs that produce warnings (5+ files)
Create `tests/corpus/warn/` with programs that should compile and run but emit warnings. Each has a companion `.stderr` for expected warning output. Coverage:
- unused variable, unused import, shadowing, unreachable code (5 files)

### Step 5: Parity enforcement in harness
For every `pass/` corpus test, the harness runs the program in BOTH interpreter and VM and asserts identical output. This is the cleanest possible parity test — written once in Atlas, verified in both engines automatically. No more duplicate Rust test functions for the same behavior.

### Step 6: Integration with nextest and CI
The corpus harness runs as `cargo nextest run -p atlas-runtime --test corpus`. Each `.atlas` file is an individually named nextest test. Add corpus run to the standard test command in `.config/nextest.toml`. Corpus tests are fast (pure computation, no I/O except the corpus files themselves).

### Step 7: Document the corpus contract
Add `tests/corpus/README.md` explaining: how to add a test case, the naming convention, how `.stdout`/`.stderr` files work, and how to update expected output when behavior intentionally changes (run with `UPDATE_CORPUS=1` env var which the harness checks to auto-update snapshot files).

## Tests
The corpus IS the tests. Minimum 50 `.atlas` files across pass/fail/warn. Each file is an independently named nextest test. The Rust harness in `corpus.rs` is ~200 lines and must itself be clean and well-commented.

## Integration Points
- Uses: `Atlas` public API for execution
- Uses: `SecurityContext::allow_all()` for corpus tests needing stdlib
- Adds: `tests/corpus/` as a permanent, growing test artifact
- All future language features MUST add corpus tests — this becomes the primary way features are proven

## Acceptance
- `tests/corpus.rs` compiles, discovers corpus files, runs them all
- 30+ pass corpus files, each with matching `.stdout`
- 15+ fail corpus files, each with matching `.stderr`
- 5+ warn corpus files, each with matching `.stderr`
- All corpus tests pass: `cargo nextest run -p atlas-runtime --test corpus`
- Harness runs each file in both interpreter AND VM (parity enforced automatically)
- `UPDATE_CORPUS=1` env var causes harness to write actual output to `.stdout`/`.stderr` files
- `tests/corpus/README.md` explains how to add tests
- `cargo nextest run -p atlas-runtime` full suite green including corpus
- No clippy warnings
- Clean git commit: `feat(tests): Add file-based test corpus (50+ .atlas test programs)`
