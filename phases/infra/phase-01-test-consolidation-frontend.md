# Phase Infra-01: Test Consolidation — Frontend & Infrastructure

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Clean working tree, all tests currently passing.

**Verification:**
```bash
git status
cargo nextest run -p atlas-runtime --test lexer_tests
cargo nextest run -p atlas-runtime --test parser_tests
cargo nextest run -p atlas-runtime --test frontend_integration_tests
```

**If failing:** Fix failing tests before consolidating. Do not move broken tests.

---

## Objective
Consolidate 20 frontend/syntax/diagnostic test files into 3 test files. Add `nextest.toml` with professional defaults. Move 2 pure value unit tests into `#[cfg(test)]` in their source files. Reduce test binary count from 125 to ~108. Verify suite is green and faster before handing off.

## Files

**Create:** `.config/nextest.toml` (~20 lines)
**Create:** `crates/atlas-runtime/tests/frontend_syntax.rs` (~600 lines, merge of 10 files)
**Create:** `crates/atlas-runtime/tests/diagnostics.rs` (~400 lines, merge of 4 files)
**Create:** `crates/atlas-runtime/tests/frontend_integration.rs` (~500 lines, merge of 3 files)
**Update:** `crates/atlas-runtime/src/value.rs` (add `#[cfg(test)]` block, migrate 2 files)
**Delete:** 20 old test files (listed in Implementation)

## Dependencies
- All existing tests currently passing (verify with GATE -1)
- No logic changes — pure file reorganization

## Implementation

### Step 0: GATE -1 — Baseline measurement
Run `cargo nextest run -p atlas-runtime` and record total wall time. This is the before number. If any tests fail, stop and report — do not proceed with broken tests.

### Step 1: Create `.config/nextest.toml`
Create nextest configuration with sensible production defaults. Set `test-threads = "num-cpus"`. Set `slow-timeout = { period = "60s", terminate-after = 3 }` to kill genuinely hung tests. Set `failure-output = "immediate"`. Set default profile to show test times. This file is tiny but important — it makes nextest behavior predictable across machines.

### Step 2: Merge into `tests/frontend_syntax.rs`
Merge these 10 files in order, preserving all test functions. Resolve name conflicts by prefixing with module-style comments as section headers. Deduplicate helper functions (e.g., `eval_ok`, `lex`, `parse` helpers appear in multiple files — keep one canonical version at top of file). Files to merge: `lexer_tests.rs`, `lexer_golden_tests.rs`, `parser_tests.rs`, `parser_error_tests.rs`, `operator_precedence_tests.rs`, `keyword_policy_tests.rs`, `generic_syntax_tests.rs`, `module_syntax_tests.rs`, `warning_tests.rs`, `warnings_tests.rs` (these two are duplicates — merge and deduplicate test functions). Also merge `test_for_in_parsing.rs` here as it is purely a syntax concern.

### Step 3: Merge into `tests/diagnostics.rs`
Merge these 4 files: `diagnostic_ordering_tests.rs`, `related_spans_tests.rs`, `enhanced_errors_tests.rs`, `sourcemap_tests.rs`. Deduplicate any shared helpers. These all test the diagnostic and error reporting infrastructure — they belong together.

### Step 4: Merge into `tests/frontend_integration.rs`
Merge these 3 files: `frontend_integration_tests.rs`, `ast_instantiation.rs`, `bytecode_validator_tests.rs`. These test the full frontend pipeline and bytecode validation as integration concerns.

### Step 5: Move unit tests into source files
`value_send_test.rs` and `value_model_tests.rs` test internal `Value` type properties — they are unit tests masquerading as integration tests. Move them into `crates/atlas-runtime/src/value.rs` inside a `#[cfg(test)] mod tests { ... }` block at the bottom of the file. They have no external dependencies that require a separate binary.

### Step 6: Delete old files
After verifying the merged files compile and all tests pass, delete the 20 source files. Use `git rm` so deletions are staged. Files to delete: all 11 files from Step 2, all 4 from Step 3, all 3 from Step 4, plus `value_send_test.rs` and `value_model_tests.rs`.

### Step 7: Verify and measure
Run full suite. Record after time. Confirm: green suite, fewer binaries, measurably faster.

## Tests
This phase contains no new tests. The acceptance criterion is that all existing tests still pass after reorganization. Verify each merged file individually before deleting sources:
```bash
cargo nextest run -p atlas-runtime --test frontend_syntax
cargo nextest run -p atlas-runtime --test diagnostics
cargo nextest run -p atlas-runtime --test frontend_integration
cargo test -p atlas-runtime value  # verifies moved unit tests
```

## Integration Points
- No runtime code changes
- No test logic changes
- Adds: `.config/nextest.toml` (affects all future test runs)
- Reduces: test binary count by ~17

## Acceptance
- All tests that existed before the phase still pass
- `frontend_syntax.rs` compiles and all tests green
- `diagnostics.rs` compiles and all tests green
- `frontend_integration.rs` compiles and all tests green
- `value.rs` `#[cfg(test)]` block compiles and all value unit tests pass
- 20 old test files deleted via `git rm`
- `.config/nextest.toml` present and valid
- `cargo nextest run -p atlas-runtime` wall time measurably reduced
- No clippy warnings introduced
- Clean git commit
