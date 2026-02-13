# Atlas CLI Configuration

Atlas uses **environment variables** for optional configuration. This approach is:
- Simple and standard across platforms
- Easy for AI agents to use
- No config file parsing needed
- Command-line flags always override environment variables

## Environment Variables

### `ATLAS_DIAGNOSTICS=json`

Default to JSON output format for diagnostics.

```bash
# Always output JSON diagnostics
export ATLAS_DIAGNOSTICS=json
atlas check file.atl  # Outputs JSON

# Override with command-line flag
atlas check file.atl  # Still JSON (from env)
```

**Use cases:**
- CI/CD pipelines that parse diagnostics
- Editor integrations (LSP, etc.)
- Automated tooling

### `ATLAS_NO_COLOR=1` or `NO_COLOR=1`

Disable colored terminal output.

```bash
# Standard NO_COLOR convention
export NO_COLOR=1

# Or Atlas-specific
export ATLAS_NO_COLOR=1
```

**Use cases:**
- CI/CD environments
- When redirecting output to files
- Terminal compatibility issues

**Note:** `NO_COLOR` is a [standard convention](https://no-color.org/) supported by many CLI tools.

### `ATLAS_HISTORY_FILE=/path/to/file`

Custom location for REPL history file.

```bash
# Use project-specific history
export ATLAS_HISTORY_FILE=./.atlas_history

# Use XDG directory
export ATLAS_HISTORY_FILE="$XDG_DATA_HOME/atlas/history"
```

**Default:** `~/.atlas/history`

**Use cases:**
- Project-specific REPL history
- XDG Base Directory compliance
- Shared history across team

### `ATLAS_NO_HISTORY=1`

Disable REPL history persistence by default.

```bash
# Privacy mode - don't save history
export ATLAS_NO_HISTORY=1
atlas repl  # No history saved

# Override with command-line
atlas repl --no-history  # Also works
```

**Use cases:**
- Privacy-sensitive environments
- Shared systems
- Demo/presentation mode

## Precedence

Command-line flags **always** override environment variables:

```bash
export ATLAS_DIAGNOSTICS=json

# Uses JSON (from env)
atlas check file.atl

# Uses human-readable (flag overrides)
atlas check file.atl  # No --json flag = human format when no env set

# To force human-readable when env is set, simply don't use --json
# The flag is opt-in, env is default
```

## Configuration Philosophy

Atlas follows these principles for configuration:

### 1. Zero Config Required

Atlas works perfectly with no configuration. All settings are optional enhancements.

```bash
# Works out of the box
atlas repl
atlas run file.atl
atlas check file.atl
```

### 2. Environment Over Files

Environment variables instead of config files because:
- ✅ Simple to set and read
- ✅ Standard across all platforms
- ✅ Easy for automation and scripts
- ✅ No parsing or validation needed
- ✅ Process-scoped (no global state)
- ✅ Easy to override per-command

### 3. Command-Line Flags Win

Explicit flags always override environment variables:
- Principle of least surprise
- Allows per-invocation customization
- Supports automation with env defaults

### 4. Sensible Defaults

All defaults are chosen for the common case:
- Human-readable output (not JSON)
- Colored output (unless disabled)
- History enabled (unless disabled)
- Standard paths (~/.atlas/)

## Examples

### CI/CD Pipeline

```bash
# .gitlab-ci.yml or .github/workflows/ci.yml
env:
  ATLAS_DIAGNOSTICS: json
  NO_COLOR: 1

script:
  - atlas check src/main.atl
  - atlas build src/main.atl
```

### Development Environment

```bash
# .envrc (direnv) or .env
export ATLAS_HISTORY_FILE=./.atlas_history  # Project-specific history
```

### Privacy Mode

```bash
# ~/.bashrc or ~/.zshrc
export ATLAS_NO_HISTORY=1  # Never save REPL history
```

### AI Agent Integration

```python
import subprocess
import os

# Set up environment for AI agent
env = os.environ.copy()
env['ATLAS_DIAGNOSTICS'] = 'json'
env['NO_COLOR'] = '1'

# Run Atlas and parse JSON output
result = subprocess.run(
    ['atlas', 'check', 'file.atl'],
    env=env,
    capture_output=True
)

diagnostics = json.loads(result.stdout)
```

## Future Configuration

Potential additions (not in v0.1):

- `ATLAS_EDITOR` - Editor for multi-line REPL input
- `ATLAS_CACHE_DIR` - Custom cache directory
- `ATLAS_MAX_ERRORS` - Maximum errors before stopping
- `ATLAS_TRACE` - Enable debug tracing

## Migration from Config Files

If you're migrating from a tool that uses config files:

```bash
# Instead of .atlasrc.json:
# {
#   "diagnostics": "json",
#   "no_color": true,
#   "history_file": "./history"
# }

# Use environment variables:
export ATLAS_DIAGNOSTICS=json
export NO_COLOR=1
export ATLAS_HISTORY_FILE=./history
```

Benefits:
- ✅ No file parsing
- ✅ No schema validation
- ✅ Easy to override
- ✅ Works everywhere (containers, CI, local)

## See Also

- [NO_COLOR](https://no-color.org/) - Standard for disabling colored output
- [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) - Standard directory locations
- [12 Factor App](https://12factor.net/config) - Config via environment
