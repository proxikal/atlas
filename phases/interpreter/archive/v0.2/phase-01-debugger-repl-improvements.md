# Phase 01: Interpreter Debugger & REPL Improvements

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Interpreter and REPL must exist from v0.1, VM debugger from bytecode-vm phases.

**Verification:**
```bash
ls crates/atlas-runtime/src/interpreter/mod.rs
ls crates/atlas-runtime/src/repl.rs
ls crates/atlas-cli/src/commands/repl.rs
ls crates/atlas-runtime/src/debugger/mod.rs
cargo nextest run -p atlas-runtime --test interpreter
```

**What's needed:**
- Interpreter from v0.1 with AST evaluation (`crates/atlas-runtime/src/interpreter/mod.rs`)
- REPL in `crates/atlas-runtime/src/repl.rs` and `crates/atlas-cli/src/commands/repl.rs` (NO separate atlas-repl crate)
- VM debugger infrastructure from bytecode-vm/phase-04
- Debugger protocol defined

**If missing:** Verify v0.1 interpreter and REPL files above exist, complete bytecode-vm debugger phases

---

## Objective
Add debugger support to interpreter achieving parity with VM debugger and enhance REPL with comprehensive commands, multi-line input detection, and improved usability for interactive development.

## Files
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~300 lines add debug hooks)
**Create:** `crates/atlas-runtime/src/interpreter/debugger.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/repl.rs` (~400 lines commands multi-line)
**Update:** `crates/atlas-cli/src/commands/repl.rs` (~300 lines add REPL commands)
**Create:** `crates/atlas-runtime/src/repl/multiline.rs` (~200 lines)
**Tests:** `crates/atlas-runtime/tests/debugger.rs` (add interpreter debugger tests to existing file)
**Tests:** `crates/atlas-runtime/tests/repl.rs` (add REPL state tests to existing file)

## Dependencies
- Interpreter from v0.1
- REPL from v0.1
- Debugger protocol from bytecode-vm/phase-04
- VM debugger as reference implementation

## Implementation

### Interpreter Debugger Infrastructure
Create InterpreterDebugger struct managing debugger state for interpreter mode. Maintain breakpoint collection with source locations. Track step mode for step-over, step-into, step-out operations. Store current pause reason breakpoint hit or step complete. Implement breakpoint checking before statement execution. Support variable inspection accessing interpreter environment. Enable expression evaluation in paused state. Maintain parity with VM debugger features and protocol.

### Debug Hooks Integration
Add debug hooks in interpreter execution loop. Insert breakpoint check before evaluating each statement. Track call depth for step operations. Pause execution at breakpoints emitting pause event. Support continue operation resuming execution. Handle step operations tracking appropriate boundaries. Allow inspection of current environment and call stack. Minimize performance impact when debugging disabled.

### REPL Command System
Implement comprehensive REPL command system with colon-prefix commands. Add help command listing all available commands with descriptions. Add clear command resetting REPL state and environment. Add load command executing Atlas file in REPL context. Add type command showing inferred type from typing/phase-02. Add vars command showing all variables from typing/phase-02. Add quit command exiting REPL. Parse commands distinguishing from Atlas code. Provide command completion hints.

### Multi-line Input Detection
Implement multi-line input support for incomplete expressions. Detect incomplete input when parsing fails with unexpected EOF. Track brace, bracket, and parenthesis nesting levels. Continue reading input lines until expression complete. Show continuation prompt for multi-line input. Handle multi-line strings and comments. Allow editing multi-line input before execution. Support cancelling multi-line input.

### REPL Usability Improvements
Enhance REPL user experience. Add color-coded output for different value types. Improve error message display using enhanced diagnostics. Add input history with up/down arrow navigation. Support persistent history across sessions. Add tab completion for variables and functions. Show helpful welcome message with available commands. Display result types alongside values.

### Interpreter-VM Parity
Ensure interpreter debugger has feature parity with VM debugger. Support identical debugger protocol. Implement same breakpoint management operations. Provide same stepping modes. Enable same inspection capabilities. Generate compatible debug events. Allow switching between interpreter and VM debugging seamlessly. Test parity with cross-engine test suite.

## Tests (TDD - Use rstest)

**Interpreter debugger tests:**
1. Breakpoint set and hit
2. Step over statements
3. Step into functions
4. Step out of functions
5. Variable inspection at breakpoints
6. Expression evaluation in pause
7. Continue after pause
8. Multiple breakpoints
9. Call stack inspection
10. Debugger performance impact

**REPL command tests:**
1. Help command lists commands
2. Clear command resets state
3. Load command executes file
4. Type command shows types
5. Vars command lists variables
6. Quit command exits
7. Invalid command handling
8. Command parsing accuracy

**Multi-line input tests:**
1. Detect incomplete braces
2. Detect incomplete brackets
3. Detect incomplete parentheses
4. Multi-line string input
5. Multi-line comment handling
6. Continuation prompt display
7. Complete expression detection
8. Cancel multi-line input

**Parity tests:**
1. Interpreter debugger matches VM debugger features
2. Protocol compatibility
3. Event format identical
4. Cross-engine test suite passes

**Minimum test count:** 100 tests (50 debugger, 50 REPL)

## Integration Points
- Uses: Interpreter from v0.1
- Uses: REPL from v0.1
- Uses: Debugger protocol from bytecode-vm phases
- Updates: Interpreter with debug hooks
- Updates: REPL with commands and multi-line
- Creates: Interpreter debugging capability
- Output: Enhanced interactive development experience

## Acceptance
- Debugger works in interpreter mode
- Breakpoints set and hit correctly
- Step operations work over into out
- Variable inspection accurate
- Expression evaluation works in pause
- All REPL commands functional help clear load type vars quit
- Multi-line input works for incomplete expressions
- Input history persists across sessions
- Color-coded output improves readability
- 100+ tests pass 50 debugger 50 REPL
- Interpreter-VM debugger parity verified
- No clippy warnings
- cargo nextest run -p atlas-runtime passes
- Enhanced REPL usability
