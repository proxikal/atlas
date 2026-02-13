# Phase 04: Debugger Infrastructure & Protocol

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Debug info and debugger hooks must exist from v0.1.

**Verification:**
```bash
grep -n "DebugInfo\|SourceMap" crates/atlas-runtime/src/bytecode/mod.rs
grep -n "debugger\|TODO.*debug" crates/atlas-runtime/src/vm/mod.rs
cargo test debug_info
```

**What's needed:**
- Debug info tracks source locations line and column
- VM has hooks for debugger integration
- Bytecode includes debug metadata

**If missing:** Check v0.1 phases bytecode-vm/phase-13 through phase-15

---

## Objective
Implement debugger infrastructure with protocol for communication, source mapping, and VM instrumentation support for breakpoints and execution control.

## Files
**Create:** `crates/atlas-runtime/src/debugger/mod.rs` (~1000 lines)
**Create:** `crates/atlas-runtime/src/debugger/protocol.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/debugger/source_map.rs` (~300 lines)
**Create:** `crates/atlas-runtime/src/debugger/state.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~150 lines add debug hooks)
**Update:** `crates/atlas-runtime/src/lib.rs` (add debugger module)
**Tests:** `crates/atlas-runtime/tests/debugger_protocol_tests.rs` (~400 lines)

## Dependencies
- v0.1 complete with debug info and source mapping
- VM supports instrumentation
- Debug info includes line and column mappings

## Implementation

### Debugger Protocol
Define communication protocol for debugger client and VM. Request types for breakpoint management set, remove, list. Execution control requests continue, step over, step into, step out, pause. Inspection requests get variables, get stack, evaluate expressions. Response types for events and data. Use serde for JSON serialization.

### Source Mapping
Maintain bidirectional mapping between bytecode locations and source positions. Map instruction pointer to source file, line, column. Map source location to bytecode instruction range. Handle multiple source files. Efficient lookup structures for performance.

### Debugger State Management
Track debugger state during execution. Maintain breakpoint list with IDs and locations. Track current execution mode running, paused, stepping. Store pause reason breakpoint hit, step complete, manual pause. Manage step mode state for step operations.

### VM Instrumentation
Add debug hooks in VM execution loop. Check breakpoints before each instruction. Support step modes with appropriate pausing. Allow inspection of stack and variables. Enable expression evaluation in paused state. Minimize performance impact when debugging disabled.

### Integration
Integrate debugger with VM as optional component. Create VM with debugger when needed. Process debug requests during execution. Emit debug events to client. Maintain debug state across execution steps.

## Tests (TDD - Use rstest)

**Debugger tests:**
1. Protocol request and response serialization
2. Source mapping bidirectional accuracy
3. Breakpoint management set, remove, hit
4. Step operations over, into, out
5. Variable inspection at breakpoints
6. Stack trace generation
7. Expression evaluation in context
8. Performance impact when disabled

**Minimum test count:** 60 tests

## Integration Points
- Uses: VM from vm/mod.rs
- Uses: Bytecode and debug info from bytecode/mod.rs
- Uses: serde for protocol serialization
- Updates: VM with debug hooks
- Creates: Complete debugger infrastructure
- Output: Debugger-ready VM for tooling

## Acceptance
- Protocol defined with all request and response types
- Source mapping accurate bidirectionally
- Breakpoints can be set and hit correctly
- Step modes work over, into, out
- Variables inspectable at breakpoints
- Stack traces generated accurately
- Expressions evaluate in paused context
- 60+ tests pass
- Performance impact negligible when disabled
- No clippy warnings
- cargo test passes
