# Phase 05: Debugger Execution Control

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Debugger infrastructure from phase 04 must be complete.

**Verification:**
```bash
ls crates/atlas-runtime/src/debugger/protocol.rs
ls crates/atlas-runtime/src/debugger/state.rs
cargo test debugger_protocol
grep -n "debugger" crates/atlas-runtime/src/vm/mod.rs
```

**What's needed:**
- Debugger protocol defined from phase 04
- Debugger state management exists
- VM has debug hooks integrated

**If missing:** Complete phase bytecode-vm/phase-04 first

---

## Objective
Implement complete debugger execution control with breakpoints, step operations, variable inspection, and expression evaluation enabling full debugging capabilities in Atlas.

## Files
**Update:** `crates/atlas-runtime/src/debugger/mod.rs` (~400 lines added)
**Create:** `crates/atlas-runtime/src/debugger/breakpoints.rs` (~300 lines)
**Create:** `crates/atlas-runtime/src/debugger/stepping.rs` (~350 lines)
**Create:** `crates/atlas-runtime/src/debugger/inspection.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~100 lines enhance hooks)
**Tests:** `crates/atlas-runtime/tests/debugger_execution_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/debugger_inspection_tests.rs` (~400 lines)

## Dependencies
- Phase bytecode-vm/phase-04 complete with infrastructure
- VM instrumented for debugging
- Source mapping functional

## Implementation

### Breakpoint Management
Implement breakpoint operations. Set breakpoints by source location or bytecode address. Store breakpoints with unique IDs. Check breakpoints before instruction execution. Hit breakpoint pauses execution. Remove breakpoints by ID. List all active breakpoints. Support conditional breakpoints for future phases.

### Step Operations
Implement all stepping modes. Step over executes next statement at current level. Step into descends into function calls. Step out continues until current function returns. Track call depth for step operations. Handle stepping across source lines correctly. Pause at appropriate instruction boundaries.

### Variable Inspection
Enable variable inspection at breakpoints. Capture current stack state. Extract local variables with names and values. Get function arguments. Access global variables. Format values for display. Handle complex types arrays and objects. Provide variable scopes local, global, closure.

### Expression Evaluation
Evaluate Atlas expressions in paused context. Parse expression string. Execute in current environment. Access current variables. Return evaluation result. Handle evaluation errors gracefully. Support simple expressions for v0.2. Complex expressions in future phases.

### Integration
Wire all components together. Process debug requests during paused state. Emit appropriate responses. Maintain execution state correctly. Resume execution after inspection. Handle errors in debug operations without crashing VM.

## Tests (TDD - Use rstest)

**Execution control tests:**
1. Breakpoint set, hit, remove operations
2. Step over across statements
3. Step into function calls
4. Step out of functions
5. Variable inspection at various scopes
6. Expression evaluation simple and complex
7. Multiple breakpoints
8. Conditional logic in debugged code
9. Error handling in debug operations
10. Performance with many breakpoints

**Minimum test count:** 100 tests (60 execution, 40 inspection)

## Integration Points
- Uses: Debugger infrastructure from phase 04
- Uses: VM execution from vm/mod.rs
- Uses: Source mapping for locations
- Updates: VM with enhanced debug hooks
- Creates: Complete debugging functionality
- Output: Fully functional debugger

## Acceptance
- Breakpoints set and hit correctly
- Step over works at statement level
- Step into descends into calls
- Step out returns to caller
- Variables inspected accurately all scopes
- Expressions evaluate in context
- Multiple breakpoints supported
- Debug operations don't crash VM
- 100+ tests pass
- Debugger usable for development
- No clippy warnings
- cargo test passes
