# Phase Interpreter-01a: Interpreter Debugger

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** All Correctness phases complete. VM debugger exists in `src/debugger/mod.rs`.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
ls crates/atlas-runtime/src/debugger/mod.rs         # VM debugger exists
ls crates/atlas-runtime/src/interpreter/mod.rs      # Interpreter exists
cargo nextest run -p atlas-runtime 2>&1 | tail -3   # Suite green
```

---

## Objective

Add debugger support to the interpreter achieving feature parity with the VM debugger. The VM debugger (`src/debugger/mod.rs`) already implements breakpoints, stepping, variable inspection, and expression evaluation. This phase creates an equivalent system for the interpreter.

**Why separate?** The VM debugger operates on bytecode with instruction pointers. The interpreter operates on AST nodes with call stacks. They require different implementations but must expose the same `DebugRequest`/`DebugResponse` protocol.

---

## Files Changed

- `crates/atlas-runtime/src/interpreter/debugger.rs` — **CREATE** (~400 lines) interpreter debugger
- `crates/atlas-runtime/src/interpreter/mod.rs` — **UPDATE** (~150 lines) add debug hooks
- `crates/atlas-runtime/tests/debugger.rs` — **UPDATE** add interpreter debugger tests

---

## Dependencies

- All Correctness phases complete
- VM debugger as reference (`src/debugger/mod.rs`)
- Shared `DebugRequest`/`DebugResponse` types

---

## Implementation

### Step 1: Create InterpreterDebugger struct

Create `interpreter/debugger.rs` with:
```rust
pub struct InterpreterDebugger {
    breakpoints: HashMap<SourceLocation, Breakpoint>,
    state: DebuggerState,
    step_mode: Option<StepMode>,
    call_depth_at_step: usize,
    pause_reason: Option<PauseReason>,
}

pub enum StepMode { Over, Into, Out }
pub enum PauseReason { Breakpoint(BreakpointId), Step, Entry }
```

Implement the same `DebugRequest`/`DebugResponse` protocol as VM debugger:
- `SetBreakpoint { file, line }` → `BreakpointSet { id }`
- `RemoveBreakpoint { id }` → `BreakpointRemoved`
- `ListBreakpoints` → `Breakpoints { list }`
- `Continue` → `Continued` or `Paused { reason }`
- `StepOver/StepInto/StepOut` → `Paused { reason }`
- `GetVariables { frame }` → `Variables { list }`
- `Evaluate { expr, frame }` → `EvaluationResult { value }`
- `GetStackTrace` → `StackTrace { frames }`

### Step 2: Add debug hooks to interpreter

In `interpreter/mod.rs`, add hooks at statement boundaries:
```rust
impl Interpreter {
    pub fn set_debugger(&mut self, debugger: Option<InterpreterDebugger>) { ... }

    fn check_breakpoint(&mut self, span: Span) -> Result<(), DebugPause> {
        if let Some(ref mut dbg) = self.debugger {
            if dbg.should_pause(span, self.call_depth) {
                return Err(DebugPause { reason: dbg.pause_reason() });
            }
        }
        Ok(())
    }
}
```

Insert `self.check_breakpoint(stmt.span)?` before each statement evaluation.

### Step 3: Implement stepping modes

**Step Over:** Execute until returning to same call depth
**Step Into:** Pause at next statement (including function entry)
**Step Out:** Execute until call depth decreases

Track `call_depth` in interpreter: increment on function call, decrement on return.

### Step 4: Variable inspection

Expose current environment for inspection:
```rust
fn collect_variables(&self, frame_index: usize) -> Vec<Variable> {
    // Walk environment chain to frame_index
    // Collect all bindings with names, types, values
}
```

### Step 5: Expression evaluation in paused state

Allow evaluating expressions in the context of a paused frame:
```rust
fn evaluate_in_context(&mut self, expr: &str, frame_index: usize) -> Result<Value, EvalError> {
    // Parse expression
    // Temporarily set environment to frame's environment
    // Evaluate expression
    // Restore environment
}
```

### Step 6: Parity verification

Ensure interpreter debugger passes the same protocol tests as VM debugger. Create parameterized tests that run against both.

---

## Tests

Add to `tests/debugger.rs` (use rstest for parameterization):

**Breakpoint tests (10):**
- `test_interp_breakpoint_set_returns_id`
- `test_interp_breakpoint_hit_pauses`
- `test_interp_breakpoint_line_accuracy`
- `test_interp_multiple_breakpoints`
- `test_interp_remove_breakpoint`
- `test_interp_clear_all_breakpoints`
- `test_interp_breakpoint_in_function`
- `test_interp_breakpoint_in_loop`
- `test_interp_breakpoint_conditional` (if supported)
- `test_interp_disabled_breakpoint`

**Stepping tests (10):**
- `test_interp_step_over_statement`
- `test_interp_step_over_function_call`
- `test_interp_step_into_function`
- `test_interp_step_out_of_function`
- `test_interp_step_over_loop_body`
- `test_interp_step_into_nested_function`
- `test_interp_step_out_multiple_frames`
- `test_interp_continue_after_step`
- `test_interp_step_at_return`
- `test_interp_step_through_if_else`

**Inspection tests (10):**
- `test_interp_get_variables_current_frame`
- `test_interp_get_variables_outer_frame`
- `test_interp_get_stack_trace`
- `test_interp_stack_frame_has_location`
- `test_interp_variable_types_correct`
- `test_interp_variable_values_correct`
- `test_interp_closure_variables_visible`
- `test_interp_global_variables_visible`
- `test_interp_shadowed_variable_correct`
- `test_interp_array_variable_inspection`

**Evaluation tests (10):**
- `test_interp_evaluate_simple_expression`
- `test_interp_evaluate_with_locals`
- `test_interp_evaluate_arithmetic`
- `test_interp_evaluate_function_call`
- `test_interp_evaluate_in_outer_frame`
- `test_interp_evaluate_syntax_error`
- `test_interp_evaluate_runtime_error`
- `test_interp_evaluate_does_not_modify_state`
- `test_interp_evaluate_with_closures`
- `test_interp_evaluate_string_operations`

**Parity tests (10):**
- `test_parity_breakpoint_protocol`
- `test_parity_step_protocol`
- `test_parity_variable_format`
- `test_parity_stack_trace_format`
- `test_parity_evaluation_result_format`
- `test_parity_error_responses`
- `test_parity_state_transitions`
- `test_parity_continue_behavior`
- `test_parity_pause_reasons`
- `test_parity_source_locations`

**Minimum test count:** 50 tests

---

## Acceptance

- `interpreter/debugger.rs` exists with `InterpreterDebugger` struct
- Debug hooks integrated in interpreter statement evaluation
- Breakpoints can be set, hit, and cleared
- Step over/into/out operations work correctly
- Variable inspection returns accurate data
- Expression evaluation works in paused state
- Protocol parity with VM debugger verified
- 50+ new interpreter debugger tests pass
- All existing tests pass unchanged
- Zero clippy warnings
- Commit: `feat(interpreter): Add debugger with VM parity`
