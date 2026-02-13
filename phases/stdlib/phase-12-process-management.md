# Phase 12: Process Management

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Security permissions must exist, stdlib infrastructure in place.

**Verification:**
```bash
ls crates/atlas-runtime/src/security/permissions.rs
cargo test security
ls crates/atlas-runtime/src/stdlib/mod.rs
```

**What's needed:**
- Security permissions from foundation/phase-15
- Result types from foundation/phase-09
- Process spawning capabilities (std::process)
- Environment variable access

**If missing:** Complete foundation phases 09 and 15 first

---

## Objective
Implement process management enabling spawning external commands, capturing output, setting environment variables, and managing child processes - providing shell-like capabilities for system automation and tooling.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/process.rs` (~800 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~100 lines Process value)
**Create:** `docs/process-management.md` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/process_tests.rs` (~700 lines)

## Dependencies
- std::process for spawning
- Security permission system
- Result types for errors
- String and array types
- Async runtime for non-blocking (optional)

## Implementation

### Command Execution
Execute external commands with exec function. Command as string or array. Capture stdout and stderr. Wait for process completion. Return exit code and output. Inherit stdio option. Spawn without waiting. Shell expansion optional. Platform-specific command handling.

### Process Spawning
Spawn child processes with spawn function. Non-blocking process creation. Returns process handle. Monitor process status. Kill or terminate process. Wait for completion. Detached process mode. Process group management. Security permission required.

### Standard I/O Handling
Capture stdout as string or stream. Capture stderr separately. Pipe stdin to process. Inherit parent stdio. Null stdio option. Custom file descriptor redirection. Line-buffered or unbuffered. Handle large output efficiently.

### Environment Variables
Get environment variable with getenv. Set environment variable with setenv. Remove environment variable with unsetenv. List all environment variables. Environment isolation for spawned processes. Inherit parent environment option. Custom environment map. Permission checks for env access.

### Working Directory
Set working directory for spawned process. Current directory with cwd function. Change directory with chdir. Validate directory exists. Permission checks for directory access. Relative and absolute paths.

### Exit Codes and Signals
Process exit code retrieval. Success and failure checking. Signal handling send signals to process. Terminate, kill, interrupt signals. Wait for signal or timeout. Exit code semantics. Platform-specific signal handling.

### Process Information
Get current process ID with pid function. Parent process ID. Process creation time. CPU and memory usage (optional). List child processes. Process status querying. Platform-specific process info.

### Shell Integration
Execute shell commands with shell function. Shell detection bash, sh, cmd, powershell. Command escaping and quoting. Shell script execution. Pipeline support with pipes. Background job execution. Shell environment inheritance.

## Tests (TDD - Use rstest)

**Command execution tests:**
1. Execute simple command
2. Capture stdout
3. Capture stderr
4. Exit code check
5. Command with arguments
6. Command not found error
7. Permission denied error
8. Inherit stdio
9. Shell command execution

**Process spawning tests:**
1. Spawn child process
2. Non-blocking spawn
3. Wait for process
4. Kill running process
5. Process exit code
6. Multiple concurrent processes
7. Detached process

**Standard I/O tests:**
1. Capture stdout
2. Capture stderr
3. Pipe stdin to process
4. Large output handling
5. Null stdio
6. Custom redirection

**Environment variable tests:**
1. Get environment variable
2. Set environment variable
3. Remove environment variable
4. List all variables
5. Custom environment for process
6. Inherit parent environment
7. Permission check for env access

**Working directory tests:**
1. Get current directory
2. Set working directory for process
3. Change directory
4. Directory not found error
5. Permission check

**Exit codes tests:**
1. Success exit code 0
2. Failure exit code non-zero
3. Specific exit code check
4. Send signal to process
5. Terminate process
6. Kill signal

**Process info tests:**
1. Get current process ID
2. Parent process ID
3. Child process list
4. Process status

**Shell integration tests:**
1. Execute shell command
2. Shell pipeline
3. Shell script
4. Background job
5. Shell detection
6. Command escaping

**Security tests:**
1. Process permission required
2. Environment permission required
3. Permission denied blocks spawn
4. Sandboxed code cannot spawn
5. Security audit log entry

**Integration tests:**
1. Run build command
2. Execute test suite
3. System automation script
4. Pipeline processing
5. Multi-process coordination

**Minimum test count:** 70 tests

## Integration Points
- Uses: Security permissions from foundation/phase-15
- Uses: Result types from foundation/phase-09
- Uses: String and array stdlib
- Updates: Value with Process variant
- Creates: Process management API
- Output: System automation capabilities

## Acceptance
- Execute external commands
- Spawn child processes
- Capture stdout and stderr
- Environment variable management
- Working directory control
- Exit codes and signals work
- Security permissions enforced
- 70+ tests pass
- Cross-platform compatibility
- Documentation with examples
- No clippy warnings
- cargo test passes
