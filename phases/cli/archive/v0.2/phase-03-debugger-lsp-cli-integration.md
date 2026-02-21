# Phase 03: Debugger & LSP CLI Integration

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Debugger and LSP must exist from bytecode-vm and lsp phases.

**Verification:**
```bash
ls crates/atlas-runtime/src/debugger/mod.rs
ls crates/atlas-lsp/src/server.rs
cargo nextest run -p atlas-runtime -E 'test(debugger_execution_tests)'
cargo nextest run -p atlas-lsp
```

**What's needed:**
- Debugger from bytecode-vm/phase-05 with execution control
- LSP server from lsp phases with protocol implementation
- CLI from v0.1 with command infrastructure

**If missing:** Complete bytecode-vm debugger phases and lsp phases first

---

## Objective
Create CLI commands for interactive debugger enabling breakpoints, stepping, and inspection plus LSP server launcher supporting stdio and TCP modes for editor integration.

## Files
**Create:** `crates/atlas-cli/src/commands/debug.rs` (~800 lines)
**Create:** `crates/atlas-cli/src/commands/lsp.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/debugger/repl.rs` (~400 lines)
**Update:** `crates/atlas-cli/src/main.rs` (~50 lines add commands)
**Tests:** `crates/atlas-cli/tests/debug_cli_tests.rs` (~300 lines)
**Tests:** `crates/atlas-cli/tests/lsp_cli_tests.rs` (~200 lines)

## Dependencies
- Debugger from bytecode-vm/phase-05
- LSP server from lsp phases
- CLI from v0.1
- REPL infrastructure for debug commands

## Implementation

### Debugger CLI Command
Create debug subcommand launching interactive debugger. Accept source file as argument. Compile source to bytecode with debug info. Create VM with debugger enabled. Start debugger REPL for interactive control. Parse debugger commands break step next continue vars inspect quit. Execute commands updating debugger state. Display current source line and execution state. Show variables and call stack on request. Support expression evaluation in debug context.

### Debugger REPL
Implement interactive debugger REPL. Display prompt showing current state paused or running. Parse user commands with arguments. Implement break command setting breakpoints by line number or function name. Implement step command stepping into functions. Implement next command stepping over function calls. Implement continue command resuming execution until breakpoint. Implement vars command showing all variables in scope. Implement inspect command evaluating expressions. Implement backtrace command showing call stack. Implement quit command exiting debugger. Show source context around current line. Highlight current execution line.

### Breakpoint Management
Manage breakpoints in debugger. Add breakpoints by line number or function name. Remove breakpoints by ID or location. List all active breakpoints. Validate breakpoint locations. Store breakpoints persistently across commands. Hit breakpoints pausing execution. Report breakpoint hits with location.

### Source Display
Display source code during debugging. Show current line with highlighting. Display context lines before and after current. Use line numbers for reference. Highlight breakpoints in source display. Update display on step operations. Support multiple source files in project.

### LSP Server CLI Command
Create lsp subcommand launching LSP server. Support stdio mode for standard LSP communication. Support TCP mode for network connections. Accept port flag for TCP mode. Initialize LSP server with protocol handler. Start message loop processing requests. Handle server lifecycle initialization, shutdown. Log server activity for debugging. Exit gracefully on shutdown request.

### LSP Server Modes
Implement both stdio and TCP server modes. Stdio mode reads from stdin writes to stdout for pipe communication. TCP mode listens on specified port accepting connections. Use same LSP handler for both modes. Support multiple concurrent connections in TCP mode. Handle connection errors gracefully. Provide clear startup messages indicating mode and address.

### Integration Testing
Test debugger CLI with various programs. Test breakpoint operations. Test stepping through code. Test variable inspection. Test expression evaluation. Test LSP server startup in both modes. Test LSP protocol message handling. Test graceful shutdown. Test error handling for invalid files.

## Tests (TDD - Use rstest)

**Debugger CLI tests:**
1. Launch debugger
2. Set breakpoint
3. Run to breakpoint
4. Step into function
5. Step over call
6. Continue execution
7. Show variables
8. Evaluate expression
9. Show backtrace
10. Quit debugger
11. Invalid command handling
12. Breakpoint removal
13. List breakpoints
14. Source display
15. Multi-file debugging

**LSP CLI tests:**
1. Start LSP stdio mode
2. Start LSP TCP mode
3. Accept connections
4. Handle initialization
5. Process requests
6. Handle shutdown
7. TCP port binding
8. Connection errors
9. Server logging
10. Graceful exit

**Minimum test count:** 60 tests (40 debugger, 20 LSP)

## Integration Points
- Uses: Debugger from bytecode-vm/phase-05
- Uses: LSP server from lsp phases
- Uses: CLI from v0.1
- Creates: Interactive debugger CLI
- Creates: LSP server launcher
- Output: Complete development tooling

## Acceptance
- atlas debug launches interactive debugger
- Breakpoints set and hit correctly
- Step commands work into over out
- Variable inspection shows values
- Expression evaluation works
- Backtrace shows call stack
- Source display highlights current line
- atlas lsp starts server in stdio mode
- atlas lsp with tcp flag starts TCP server
- LSP server handles protocol messages
- Server initialization completes successfully
- Server shutdown graceful
- 60+ tests pass 40 debugger 20 LSP
- Debugger REPL user-friendly
- Error messages clear
- No clippy warnings
- cargo nextest run -p atlas-cli passes
