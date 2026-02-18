# Phase Infra-02: Test Consolidation — Core Runtime

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase Infra-01 complete. `frontend_syntax.rs`, `diagnostics.rs`, `frontend_integration.rs` exist and are green. `.config/nextest.toml` present.

**Verification:**
```bash
cargo nextest run -p atlas-runtime --test frontend_syntax
cargo nextest run -p atlas-runtime --test diagnostics
cargo nextest run -p atlas-runtime --test frontend_integration
cargo nextest run -p atlas-runtime 2>&1 | tail -5
```

**If failing:** Fix before proceeding. Do not consolidate failing tests.

---

## Objective
Consolidate ~70 core runtime test files into 6 target files covering: type system, interpreter, VM, stdlib (with parity), collections, and bytecode. Fix the `vm_performance_tests.rs` fib timeout. Reduce test binary count from ~108 to ~38. This is the highest-impact phase — the stdlib and VM duplicate pairs eliminated here account for most of the 2.3GB binary bloat.

## Files

**Create:** `crates/atlas-runtime/tests/typesystem.rs` (~700 lines, merge of 14 files)
**Create:** `crates/atlas-runtime/tests/interpreter.rs` (~700 lines, merge of 10 files)
**Create:** `crates/atlas-runtime/tests/vm.rs` (~700 lines, merge of 9 files)
**Create:** `crates/atlas-runtime/tests/stdlib.rs` (~1200 lines, merge of 20 files)
**Create:** `crates/atlas-runtime/tests/collections.rs` (~400 lines, merge of 5 files)
**Create:** `crates/atlas-runtime/tests/bytecode.rs` (~400 lines, merge of 5 files)
**Delete:** All ~70 source files listed per step below

## Dependencies
- Phase Infra-01 complete
- All existing tests passing

## Implementation

### Step 0: GATE -1 — Baseline
Record current test count and wall time: `cargo nextest run -p atlas-runtime 2>&1 | tail -3`

### Step 1: Create `tests/typesystem.rs`
Merge 14 type system files. Deduplicate shared helpers (most use `eval_ok`/`check_types` variants — keep one per approach at file top). Organize with section headers matching the original file purposes. Files: `advanced_inference_tests.rs`, `constraint_tests.rs`, `function_return_analysis_tests.rs`, `generic_type_checking_tests.rs`, `intersection_type_tests.rs`, `nullability_tests.rs`, `type_alias_tests.rs`, `type_guard_tests.rs`, `type_improvements_tests.rs`, `type_inference_tests.rs`, `type_rules_tests.rs`, `typecheck_dump_stability_tests.rs`, `typing_integration_tests.rs`, `union_type_tests.rs`. Verify: `cargo nextest run -p atlas-runtime --test typesystem`

### Step 2: Create `tests/interpreter.rs`
Merge 10 interpreter-focused test files. Deduplicate `eval_ok`/`run_interpreter` helpers. Files: `interpreter_integration_tests.rs`, `interpreter_member_tests.rs`, `nested_function_binding_tests.rs`, `nested_function_interpreter_tests.rs`, `nested_function_typecheck_tests.rs`, `scope_shadowing_tests.rs`, `pattern_matching_tests.rs`, `assignment_target_tests.rs`, `test_for_in_edge_cases.rs`, `test_for_in_semantic.rs`. Verify: `cargo nextest run -p atlas-runtime --test interpreter`

### Step 3: Create `tests/vm.rs`
Merge 9 VM-focused test files. Fix `test_perf_recursive_fib_completes` during merge: change `fib(20)` to `fib(15)` (1973 calls vs 21891) and change the timeout assertion from `< 10` seconds to `< 2` seconds. This makes the test a real guard not a rubber stamp. Files: `vm_integration_tests.rs`, `vm_member_tests.rs`, `vm_complex_programs.rs`, `vm_regression_tests.rs`, `vm_performance_tests.rs`, `vm_first_class_functions_tests.rs`, `vm_generics_runtime_tests.rs`, `nested_function_vm_tests.rs`, `test_for_in_execution.rs`. Verify: `cargo nextest run -p atlas-runtime --test vm`

### Step 4: Create `tests/stdlib.rs` (critical — eliminate duplicate pairs)
This is the most important merge. The 10 duplicate `stdlib_X.rs` + `vm_stdlib_X.rs` pairs are the structural rot at the core of the bloat. The new pattern: one function `assert_parity(code, expected)` that runs code in both interpreter and VM and asserts identical output. Every test uses this helper — no more separate interpreter/VM test functions for the same behavior. Merge 20 files: `stdlib_integration_tests.rs`, `stdlib_string_tests.rs`, `vm_stdlib_string_tests.rs`, `stdlib_json_tests.rs`, `vm_stdlib_json_tests.rs`, `stdlib_io_tests.rs`, `vm_stdlib_io_tests.rs`, `stdlib_types_tests.rs`, `vm_stdlib_types_tests.rs`, `stdlib_real_world_tests.rs`, `stdlib_parity_verification.rs`, `option_result_tests.rs`, `vm_option_result_tests.rs`, `result_advanced_tests.rs`, `vm_result_advanced_tests.rs`, `first_class_functions_tests.rs`, `test_primitives.rs`, `prelude_tests.rs`, `numeric_edge_cases_tests.rs`, `collection_iteration_tests.rs`. For the duplicate pairs: keep the interpreter test as-is, keep the VM test as-is — do NOT silently drop them. Both variants are valid coverage. Just co-locate them in one file. Verify: `cargo nextest run -p atlas-runtime --test stdlib`

### Step 5: Create `tests/collections.rs`
Merge 5 collection files: `hash_function_tests.rs`, `hashset_tests.rs`, `queue_tests.rs`, `stack_tests.rs`, `generics_runtime_tests.rs`. Verify: `cargo nextest run -p atlas-runtime --test collections`

### Step 6: Create `tests/bytecode.rs`
Merge 5 bytecode/optimizer files: `bytecode_compiler_integration.rs`, `optimizer_tests.rs`, `optimizer_integration_tests.rs`, `profiler_tests.rs`, `nested_function_parity_tests.rs`. Also merge `pattern_matching_runtime_tests.rs` here as it exercises bytecode execution paths. Verify: `cargo nextest run -p atlas-runtime --test bytecode`

### Step 7: Delete old files
`git rm` all ~70 source files merged in Steps 1-6. Run full suite to confirm green: `cargo nextest run -p atlas-runtime`

## Tests
No new tests. Acceptance = all existing tests still pass after reorganization. Track function counts before/after to ensure no tests were accidentally dropped.

## Integration Points
- No runtime code changes
- vm_performance_tests fib fix is a correctness improvement (old 10s tolerance was noise)
- Eliminates the `vm_stdlib_X.rs` + `stdlib_X.rs` duplicate binary pattern
- Reduces ~70 binaries to 6

## Acceptance
- All 6 new test files compile and all tests green
- `vm_performance_tests` fib reduced to fib(15) with 2s timeout
- All duplicate `vm_stdlib_*` + `stdlib_*` pairs merged into single `stdlib.rs`
- All ~70 old source files deleted via `git rm`
- `cargo nextest run -p atlas-runtime` suite green end-to-end
- Test count before == test count after (no accidental drops — verify with `cargo nextest list`)
- Wall time measurably faster than after Infra-01
- No clippy warnings
- Clean git commit
