# Atlas CLI Guide

**Version:** v0.2 | **Status:** Production-Ready

---

## Overview

The Atlas CLI (`atlas`) provides a comprehensive set of tools for developing, testing, and debugging Atlas programs. It follows familiar conventions from modern development tools while providing Atlas-specific features.

---

## Installation

### From Source

```bash
cargo install --path crates/atlas-cli
```

### Shell Completions

Generate and install shell completions for enhanced command-line experience:

**Bash:**
```bash
atlas completions bash > ~/.bash_completions/atlas.bash
echo 'source ~/.bash_completions/atlas.bash' >> ~/.bashrc
```

**Zsh:**
```bash
atlas completions zsh > ~/.zfunc/_atlas
# Add to .zshrc before compinit:
# fpath=(~/.zfunc $fpath)
```

**Fish:**
```bash
atlas completions fish > ~/.config/fish/completions/atlas.fish
# Automatically loaded by fish
```

**PowerShell:**
```powershell
atlas completions powershell > atlas.ps1
# Add to profile: . ./atlas.ps1
```

---

## Commands

### Run Programs

```bash
atlas run <file>           # Run an Atlas program
atlas r <file>             # Short alias
atlas run main.atl --watch # Watch mode with auto-reload
atlas run main.atl --json  # JSON diagnostic output
```

**Options:**
- `-w, --watch` - Watch for file changes and auto-recompile
- `--no-clear` - Don't clear terminal on recompile (with --watch)
- `-v, --verbose` - Show timing information
- `--json` - Output diagnostics as JSON

### Type Check

```bash
atlas check <file>         # Check for errors without running
atlas c <file>             # Short alias
atlas check main.atl --json
```

**Options:**
- `--json` - Output diagnostics as JSON

### Build Projects

```bash
atlas build                # Build with default profile
atlas b                    # Short alias
atlas build --release      # Build optimized release
atlas build --profile=test # Custom profile
atlas build --clean        # Clean rebuild
```

**Options:**
- `-p, --profile <name>` - Build profile (dev, release, test)
- `--release` - Shorthand for --profile=release
- `--clean` - Clean build, ignore cache
- `-v, --verbose` - Verbose output
- `-q, --quiet` - Errors only
- `--json` - JSON output

### Format Code

```bash
atlas fmt <files>          # Format files
atlas f <files>            # Short alias
atlas fmt src/ --check     # Check without modifying
atlas fmt . --write        # Format all recursively
atlas fmt main.atl --indent-size=2
```

**Options:**
- `--check` - Check formatting without modifying
- `-w, --write` - Write changes to files
- `-c, --config <file>` - Config file path
- `--indent-size <n>` - Indentation size (default: 4)
- `--max-width <n>` - Maximum line width (default: 100)
- `--trailing-commas <bool>` - Enable/disable trailing commas
- `-v, --verbose` - Verbose output with timing
- `-q, --quiet` - Suppress non-error output

### Run Tests

```bash
atlas test                 # Run all tests
atlas t                    # Short alias
atlas test auth            # Filter by pattern
atlas test --dir=tests/    # Specific directory
atlas test --verbose       # Show all test names
atlas test --sequential    # Disable parallelism
```

**Options:**
- `<pattern>` - Filter tests by name
- `--sequential` - Run sequentially (no parallelism)
- `-v, --verbose` - Show all test names
- `--no-color` - Disable colored output
- `--dir <path>` - Test directory (default: .)
- `--json` - JSON output

### Debug Programs

```bash
atlas debug <file>         # Start debugging
atlas d <file>             # Short alias
atlas debug main.atl -b 10 # Break at line 10
atlas debug main.atl -b 10 -b 20  # Multiple breakpoints
```

**Debugger Commands:**
- `break <line>` (b) - Set breakpoint at line
- `step` (s) - Step into
- `next` (n) - Step over
- `continue` (c) - Continue execution
- `out` (o) - Step out
- `print <expr>` (p) - Evaluate expression
- `vars` (v) - Show local variables
- `backtrace` (bt) - Show call stack
- `list` (l) - Show source around current line
- `location` (loc) - Show current location
- `breakpoints` (bp) - List breakpoints
- `clear` - Clear all breakpoints
- `quit` (q) - Exit debugger

### Start REPL

```bash
atlas repl                 # Start line editor REPL
atlas repl --tui           # Start TUI mode (ratatui)
atlas repl --no-history    # Disable history persistence
```

**REPL Commands:**
- `:help` (`:h`) - Show help
- `:quit` (`:q`) - Exit REPL
- `:reset` (`:clear`) - Clear all definitions
- `:load <file>` (`:l`) - Load and run a file
- `:type <expr>` - Show expression type
- `:vars [page]` - List variables

### Start Language Server

```bash
atlas lsp                  # Start in stdio mode
atlas lsp --tcp            # Start TCP server
atlas lsp --tcp --port=8080
atlas lsp --verbose        # Enable logging
```

**Options:**
- `--tcp` - Use TCP mode instead of stdio
- `--port <n>` - TCP port (default: 9257)
- `--host <addr>` - Bind address (default: 127.0.0.1)
- `-v, --verbose` - Enable verbose logging

### Profile Programs

```bash
atlas profile <file>       # Profile execution
atlas profile slow.atl -o report.txt
atlas profile slow.atl --summary
atlas profile slow.atl --threshold=5.0
```

**Options:**
- `--threshold <pct>` - Hotspot detection threshold (default: 1.0)
- `-o, --output <file>` - Save report to file
- `--summary` - Brief output only

### Dump AST

```bash
atlas ast <file>           # Print AST as JSON
atlas ast main.atl > ast.json
```

### Dump Type Info

```bash
atlas typecheck <file>     # Print type info as JSON
atlas typecheck main.atl | jq
```

### Generate Completions

```bash
atlas completions <shell>
atlas completions bash
atlas completions zsh
atlas completions fish
atlas completions powershell
```

---

## Command Aliases

Quick access to common commands:

| Full Command | Alias | Description |
|-------------|-------|-------------|
| `atlas run` | `atlas r` | Run a program |
| `atlas check` | `atlas c` | Type-check |
| `atlas build` | `atlas b` | Build project |
| `atlas test` | `atlas t` | Run tests |
| `atlas fmt` | `atlas f` | Format code |
| `atlas debug` | `atlas d` | Debug program |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ATLAS_JSON` | Set to '1' for JSON output by default |
| `ATLAS_NO_HISTORY` | Set to '1' to disable REPL history |
| `NO_COLOR` | Set to disable colored output |

---

## Configuration

The CLI reads configuration from `atlas.toml` in the current directory:

```toml
[project]
name = "my-project"
version = "1.0.0"
entry = "src/main.atl"

[build]
output = "dist"

[format]
indent_size = 4
max_width = 100
trailing_commas = true
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (compilation, runtime, etc.) |
| 2 | Invalid arguments or usage |

---

## Examples

### Development Workflow

```bash
# Format, check, then run
atlas fmt src/ --write
atlas check src/main.atl
atlas run src/main.atl

# Or use watch mode for continuous development
atlas run src/main.atl --watch
```

### Testing Workflow

```bash
# Run all tests with verbose output
atlas test --verbose

# Run specific tests
atlas test auth_
atlas test --dir=tests/integration
```

### Debugging Workflow

```bash
# Start debugger with breakpoint
atlas debug src/main.atl -b 15

# In debugger:
(atlas-debug) vars      # See variables
(atlas-debug) step      # Step into
(atlas-debug) print x   # Evaluate expression
(atlas-debug) continue  # Run to next breakpoint
```

### CI/CD Integration

```bash
# Check formatting (fails if changes needed)
atlas fmt src/ --check

# Type-check (fails on errors)
atlas check src/main.atl

# Run tests with JSON output
atlas test --json > test-results.json
```

---

## Troubleshooting

### Common Issues

**"File not found" error:**
Check that the file path is correct. Use absolute paths or paths relative to current directory.

**Formatter changes nothing:**
Use `--write` flag to modify files. Without it, `fmt` only checks and reports.

**Tests not discovered:**
Ensure test files follow the naming convention (e.g., `test_*.atl` or `*_test.atl`).

**LSP not connecting:**
In TCP mode, ensure the port isn't already in use. Check firewall settings.

### Getting Help

```bash
atlas --help           # General help
atlas <command> --help # Command-specific help
atlas help <command>   # Alternative help syntax
```

For more information: https://atl-lang.github.io/atlas
