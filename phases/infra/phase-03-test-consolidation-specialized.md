# Phase Infra-03: Test Consolidation — Specialized + Final Verification

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase Infra-01 and Infra-02 complete. Suite green after both.

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -5
ls crates/atlas-runtime/tests/*.rs | wc -l  # should be ~38
```

**If failing:** Fix Infra-02 before proceeding.

---

## Objective
Consolidate the remaining ~38 specialized test files into 10 domain files. Produce the final professional test structure: ~17 total files, clean nextest run under 20 seconds. This phase covers async, FFI, debugger, security, modules, HTTP, regex, datetime, system I/O, API, REPL, and regression.

## Files

**Create:** `crates/atlas-runtime/tests/async_runtime.rs` (~500 lines, merge of 3)
**Create:** `crates/atlas-runtime/tests/ffi.rs` (~600 lines, merge of 6)
**Create:** `crates/atlas-runtime/tests/debugger.rs` (~500 lines, merge of 3)
**Create:** `crates/atlas-runtime/tests/security.rs` (~400 lines, merge of 3)
**Create:** `crates/atlas-runtime/tests/modules.rs` (~500 lines, merge of 4)
**Create:** `crates/atlas-runtime/tests/http.rs` (~400 lines, merge of 2)
**Create:** `crates/atlas-runtime/tests/datetime_regex.rs` (~400 lines, merge of 4)
**Create:** `crates/atlas-runtime/tests/system.rs` (~400 lines, merge of 5)
**Create:** `crates/atlas-runtime/tests/api.rs` (~500 lines, merge of 5)
**Create:** `crates/atlas-runtime/tests/repl.rs` (~300 lines, merge of 2)
**Rename/keep:** `regression_suite.rs` → stays as `tests/regression.rs`
**Delete:** All ~37 source files replaced by the above

## Dependencies
- Infra-01 and Infra-02 complete
- All runtime tests currently passing

## Implementation

### Step 1: Create `tests/async_runtime.rs`
Merge 3 files: `async_future_tests.rs`, `async_io_tests.rs`, `async_primitives_tests.rs`. Most tests here are properly `#[ignore]`'d — preserve all ignore annotations exactly. Do not remove any `#[ignore]` attributes. Verify: `cargo nextest run -p atlas-runtime --test async_runtime`

### Step 2: Create `tests/ffi.rs`
Merge 6 files: `ffi_callback_tests.rs`, `ffi_integration_complete_tests.rs`, `ffi_interpreter_tests.rs`, `ffi_parsing_tests.rs`, `ffi_types_tests.rs`, `ffi_vm_tests.rs`. Preserve all `#[ignore]` and `#[cfg_attr(target_os = ..., ignore = ...)]` annotations exactly — the platform-specific ignores are critical. Verify: `cargo nextest run -p atlas-runtime --test ffi`

### Step 3: Create `tests/debugger.rs`
Merge 3 files: `debugger_execution_tests.rs`, `debugger_inspection_tests.rs`, `debugger_protocol_tests.rs`. These share similar helpers — deduplicate. Verify: `cargo nextest run -p atlas-runtime --test debugger`

### Step 4: Create `tests/security.rs`
Merge 3 files: `security_tests.rs`, `runtime_security_tests.rs`, `audit_logging_tests.rs`. Verify: `cargo nextest run -p atlas-runtime --test security`

### Step 5: Create `tests/modules.rs`
Merge 4 files: `module_binding_tests.rs`, `module_execution_tests.rs`, `module_execution_vm_tests.rs`, `module_resolution_tests.rs`. Verify: `cargo nextest run -p atlas-runtime --test modules`

### Step 6: Create `tests/http.rs`
Merge 2 files: `http_core_tests.rs`, `http_advanced_tests.rs`. All network-dependent tests already carry `#[ignore = "requires network"]` — preserve exactly. Verify: `cargo nextest run -p atlas-runtime --test http`

### Step 7: Create `tests/datetime_regex.rs`
Merge 4 files: `datetime_core_tests.rs`, `datetime_advanced_tests.rs`, `regex_core_tests.rs`, `regex_operations_tests.rs`. These are small domain tests that belong together as stdlib extensions. Verify: `cargo nextest run -p atlas-runtime --test datetime_regex`

### Step 8: Create `tests/system.rs`
Merge 5 files: `path_tests.rs`, `fs_tests.rs`, `process_tests.rs`, `gzip_tests.rs`, `tar_tests.rs`, `zip_tests.rs`. All test OS-level interactions. Verify: `cargo nextest run -p atlas-runtime --test system`

### Step 9: Create `tests/api.rs`
Merge 5 files: `api_tests.rs`, `api_conversion_tests.rs`, `api_native_functions_tests.rs`, `api_sandboxing_tests.rs`, `reflection_tests.rs`. These all test the public embedding API surface. Verify: `cargo nextest run -p atlas-runtime --test api`

### Step 10: Create `tests/repl.rs`
Merge 2 files: `repl_state_tests.rs`, `repl_types_tests.rs`. Verify: `cargo nextest run -p atlas-runtime --test repl`

### Step 11: Handle `regression_suite.rs`
Keep as `tests/regression.rs` — rename via `git mv`. Regression tests are explicitly named and should stay isolated for easy targeted re-runs. No merge needed.

### Step 12: Delete old files
`git rm` all ~37 replaced source files.

### Step 13: Final verification and measurement
Run the complete suite: `time cargo nextest run -p atlas-runtime`. Record wall time. Compare to baseline from Infra-01/02. Document the improvement. Run: `cargo nextest list -p atlas-runtime | wc -l` to verify total test count matches pre-consolidation count. Run clippy: `cargo clippy -p atlas-runtime -- -D warnings`.

### Step 14: Final structure audit
```bash
ls crates/atlas-runtime/tests/*.rs
```
Expected output: exactly 17 files:
`api.rs`, `async_runtime.rs`, `bytecode.rs`, `collections.rs`, `datetime_regex.rs`, `debugger.rs`, `diagnostics.rs`, `ffi.rs`, `frontend_integration.rs`, `frontend_syntax.rs`, `http.rs`, `interpreter.rs`, `modules.rs`, `regression.rs`, `repl.rs`, `security.rs`, `stdlib.rs`, `system.rs`, `typesystem.rs`, `vm.rs`
Plus subdirectories: `common/`, `errors/`, `snapshots/`, `integration/`, `vm/`, `unit/`, `stdlib/`

## Tests
No new tests. Verification = all existing tests pass with identical count.

## Integration Points
- No runtime code changes whatsoever
- Completes the 3-phase test infrastructure overhaul
- Final state: ~17-20 test binaries instead of 125+

## Acceptance
- All 10 new test files compile and tests green
- `regression.rs` present (renamed from `regression_suite.rs`)
- All ~37 old source files deleted via `git rm`
- `cargo nextest run -p atlas-runtime` full suite green
- Test count before consolidation == test count after (verified with `cargo nextest list`)
- All `#[ignore]` annotations preserved exactly (network, platform, tokio-context)
- `cargo clippy -p atlas-runtime -- -D warnings` passes with zero warnings
- Wall time for full suite under 20 seconds on developer hardware
- Binary count: `ls target/debug/deps/*.d | wc -l` measurably reduced
- Clean git commit with message: `refactor(tests): Complete test infrastructure consolidation (125 → ~17 binaries)`
- Update STATUS.md: mark all 3 infra phases complete, restore Next Phase to `interpreter/phase-01`
