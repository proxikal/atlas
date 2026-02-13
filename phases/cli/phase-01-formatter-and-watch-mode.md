# Phase 01: Formatter & Watch Mode CLI

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Formatter must exist from frontend/phase-02 and CLI from v0.1.

**Verification:**
```bash
ls crates/atlas-formatter/src/formatter.rs
ls crates/atlas-cli/src/main.rs
cargo test formatter_tests
cargo run --bin atlas -- --help
```

**What's needed:**
- Formatter from frontend/phase-02 with comment preservation
- CLI from v0.1 with command structure
- Configuration system from foundation/phase-04

**If missing:** Complete frontend/phase-02 and foundation/phase-04 first

---

## Objective
Integrate formatter into CLI with file and directory support plus implement watch mode for automatic recompilation on source file changes enabling rapid development workflow.

## Files
**Create:** `crates/atlas-cli/src/commands/fmt.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/commands/watch.rs` (~400 lines)
**Update:** `crates/atlas-cli/src/main.rs` (~50 lines add commands)
**Tests:** `crates/atlas-cli/tests/fmt_tests.rs` (~200 lines)
**Tests:** `crates/atlas-cli/tests/watch_tests.rs` (~200 lines)

## Dependencies
- Formatter from frontend/phase-02
- CLI structure from v0.1
- Configuration system from foundation/phase-04
- notify crate for file watching

## Implementation

### Formatter CLI Command
Create fmt subcommand in CLI. Accept file paths and directories as arguments. Read source file into string. Parse source into AST handling parse errors. Load formatter configuration from atlas.toml. Create formatter instance with loaded config. Format AST producing formatted output. Support stdout mode printing formatted code. Support write mode modifying files in-place. Support check mode verifying formatting without changes. Recurse through directories finding all Atlas files. Format multiple files with progress indication. Report formatting errors clearly. Exit with appropriate status codes.

### Formatter Flags
Implement command-line flags for formatter. Add write flag for in-place modification. Add check flag for verification without changes. Add config flag for custom config file path. Add max-width flag overriding config. Add indent-size flag overriding config. Add trailing-comma flag overriding config. Add verbose flag for detailed output. Add quiet flag suppressing non-error output.

### Watch Mode Infrastructure
Implement watch mode for automatic recompilation. Use notify crate for file system monitoring. Watch source file and dependencies for changes. Detect file modification events. Debounce rapid successive changes. Clear terminal on recompilation. Recompile and run program on change. Display compilation errors clearly. Continue watching after errors. Support graceful shutdown with Ctrl-C.

### Watch Mode Integration
Add watch flag to run command. Start file watcher before initial run. Execute program initially then watch for changes. On file change recompile and rerun automatically. Maintain REPL-like experience with automatic updates. Show clear feedback on recompilation. Handle multiple rapid changes gracefully. Cleanup file watcher on exit.

### Directory Recursion
Implement recursive directory formatting. Find all Atlas files with .at extension. Maintain directory structure during formatting. Skip ignored files and directories from .gitignore. Format files in parallel when safe. Report summary of formatted files. Handle formatting errors per file without stopping. Provide option to format only changed files.

### Error Handling
Handle all error conditions gracefully. Report parse errors with file location. Report I/O errors reading or writing files. Report invalid configuration errors. Report watch mode errors file not found or permission denied. Provide clear error messages with suggestions. Exit with non-zero status on errors. Support continuing after recoverable errors in batch mode.

## Tests (TDD - Use rstest)

**Formatter CLI tests:**
1. Format to stdout
2. Format in-place with write flag
3. Check mode verification
4. Format directory recursively
5. Configuration override via flags
6. Invalid file handling
7. Parse error reporting
8. Multiple file formatting
9. Progress indication
10. Exit status codes

**Watch mode tests:**
1. Initial run before watching
2. Detect file changes
3. Recompile on change
4. Display errors during watch
5. Continue after errors
6. Debounce rapid changes
7. Graceful shutdown
8. Multiple file watching
9. Terminal clearing
10. File not found handling

**Minimum test count:** 60 tests (30 formatter, 30 watch)

## Integration Points
- Uses: Formatter from frontend/phase-02
- Uses: CLI from v0.1
- Uses: Configuration from foundation/phase-04
- Creates: CLI fmt command
- Creates: Watch mode for run command
- Output: Developer-friendly CLI tools

## Acceptance
- atlas fmt command works with files
- atlas fmt with write flag modifies in-place
- atlas fmt with check flag verifies formatting
- atlas fmt formats directories recursively
- Configuration flags override atlas.toml
- atlas run with watch flag auto-recompiles
- Watch mode detects changes within 500ms
- Watch mode clears terminal on recompile
- Watch mode continues after errors
- 60+ tests pass 30 formatter 30 watch
- Error messages clear and actionable
- Exit status codes appropriate
- No clippy warnings
- cargo test passes
