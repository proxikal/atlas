# Atlas Configuration System

Atlas provides a hierarchical configuration system supporting project-level (`atlas.toml`) and user-level (`~/.atlas/config.toml`) settings.

## Table of Contents

- [Configuration Hierarchy](#configuration-hierarchy)
- [Project Configuration](#project-configuration)
- [Global Configuration](#global-configuration)
- [Environment Variables](#environment-variables)
- [CLI Overrides](#cli-overrides)
- [Configuration Reference](#configuration-reference)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Configuration Hierarchy

Configuration is loaded and merged in the following order (later overrides earlier):

1. **Global config** (`~/.atlas/config.toml`) - lowest priority
2. **Project config** (`./atlas.toml`) - overrides global
3. **Environment variables** (`ATLAS_*`) - overrides project
4. **CLI flags** - highest priority (handled by caller)

## Project Configuration

Project configuration is stored in `atlas.toml` at the project root.

### File Location

Atlas searches for `atlas.toml` starting in the current directory and walking up the directory tree until one is found or the filesystem root is reached.

```bash
/path/to/project/
├── atlas.toml           # Project config
├── src/
│   └── main.atl
└── tests/
    └── test.atl
```

### Minimal Configuration

```toml
[package]
name = "my-project"
version = "1.0.0"
```

## Global Configuration

Global configuration is stored in `~/.atlas/config.toml`.

This file is optional and provides user-wide defaults for:
- Default edition for new projects
- Formatting preferences
- Permission defaults
- LSP settings

### File Location

```
~/.atlas/config.toml
```

## Environment Variables

Environment variables can override configuration values:

| Variable | Overrides | Values |
|----------|-----------|--------|
| `ATLAS_EDITION` | `package.edition` | Edition string (e.g., "2026") |
| `ATLAS_OPTIMIZE` | `compiler.optimize` | `true`, `false`, `1`, `0`, `yes`, `no` |
| `ATLAS_DEBUG` | `compiler.debug` | `true`, `false`, `1`, `0`, `yes`, `no` |

### Example

```bash
# Override edition for this run
ATLAS_EDITION=2027 atlas run main.atl

# Enable optimization
ATLAS_OPTIMIZE=true atlas build main.atl
```

## CLI Overrides

Command-line flags have the highest priority and override all configuration sources.

```bash
# --json flag overrides config settings
atlas run main.atl --json

# --no-history overrides REPL history settings
atlas repl --no-history
```

## Configuration Reference

### Package Section

```toml
[package]
name = "my-project"              # Required: Package name
version = "1.0.0"                # Required: Semver version
edition = "2026"                 # Optional: Atlas edition (default: 2026)
description = "My Atlas project" # Optional: Package description
authors = ["John Doe"]           # Optional: Package authors
license = "MIT"                  # Optional: License identifier
repository = "https://..."       # Optional: Repository URL
```

**Validation:**
- `name`: Cannot be empty
- `version`: Must be valid semver (e.g., "1.0.0", "0.1.0", "1.0")
- `edition`: Must be "2026" or later

### Build Section

```toml
[build]
output = "target"    # Optional: Output directory (default: "target")
source = "src"       # Optional: Source directory (default: "src")
entry = "src/main.atl"  # Optional: Entry point (default: "src/main.atl")
```

### Compiler Section

```toml
[compiler]
optimize = true      # Optional: Enable optimizations (default: false)
target = "bytecode"  # Optional: Target (interpreter, bytecode)
debug = true         # Optional: Enable debug info (default: false)
```

### Formatting Section

```toml
[formatting]
indent = 4            # Optional: Indentation size (default: 4)
max_line_length = 100 # Optional: Max line length (default: 100)
use_tabs = false      # Optional: Use tabs instead of spaces (default: false)
```

### Security Section

```toml
[security]
mode = "standard"  # Optional: Security mode (none, standard, strict)

[security.filesystem]
read = ["./data", "./config"]  # Paths allowed for reading
write = ["./output"]           # Paths allowed for writing
deny = ["/etc"]                # Paths explicitly denied

[security.network]
allow = ["api.example.com"]    # Hosts/domains allowed
deny = ["*"]                   # Hosts/domains denied

[security.process]
allow = ["git", "npm"]         # Commands allowed to execute
deny = ["rm", "dd"]            # Commands explicitly denied

[security.environment]
allow = ["PATH", "HOME"]       # Environment variables allowed to read
deny = ["SECRET_*"]            # Environment variables denied
```

**Security Modes:**
- `none`: No security restrictions
- `standard`: Default security policies (default)
- `strict`: Maximum security restrictions

### Dependencies Section

```toml
[dependencies]
# Simple version dependency
utils = "1.0.0"

# Git dependency
http = { git = "https://github.com/org/http.git" }

# Path dependency
local-lib = { path = "../local-lib" }

# Registry dependency
parser = { version = "2.0", registry = "atlas-registry" }

[dev-dependencies]
test-utils = "0.1.0"
```

## Examples

### Basic Project

```toml
[package]
name = "hello"
version = "0.1.0"
edition = "2026"
```

### Production Build

```toml
[package]
name = "my-app"
version = "1.0.0"
edition = "2026"

[compiler]
optimize = true
target = "bytecode"
debug = false

[build]
output = "dist"
entry = "src/main.atl"
```

### Strict Security

```toml
[package]
name = "secure-app"
version = "1.0.0"

[security]
mode = "strict"

[security.filesystem]
read = ["./data"]
write = ["./output"]
deny = ["/"]

[security.network]
deny = ["*"]

[security.process]
deny = ["*"]
```

### Global User Config

`~/.atlas/config.toml`:

```toml
[defaults]
edition = "2026"
author = "John Doe <john@example.com>"
license = "MIT"

[formatting]
indent = 2
max_line_length = 120
use_tabs = false

[permissions]
network = "prompt"
filesystem = "prompt"
env = "allow"

[lsp]
diagnostics = true
completion = true
hover = true
```

### Custom Formatting

```toml
[package]
name = "formatted-project"
version = "1.0.0"

[formatting]
indent = 2
max_line_length = 120
use_tabs = false
```

## Troubleshooting

### Configuration Not Found

If Atlas doesn't find `atlas.toml`, it uses default configuration:

```
No project configuration found. Using defaults.
```

**Solutions:**
- Create `atlas.toml` in project root
- Run commands from project directory
- Use absolute paths to specify files

### Invalid TOML Syntax

```
Error: Invalid TOML syntax in atlas.toml: expected '=', found ':'
```

**Solutions:**
- Validate TOML syntax (use TOML linter)
- Check for typos in keys
- Ensure proper quoting of strings

### Unknown Field

```
Error: Unknown field 'unknown_key' in atlas.toml
```

**Solutions:**
- Check spelling of configuration keys
- Refer to configuration reference above
- Remove unsupported fields

### Invalid Value

```
Error: Invalid value for 'package.edition': invalid edition '2025'
```

**Solutions:**
- Check valid values in configuration reference
- Ensure versions are valid semver
- Verify edition is 2026 or later

### Environment Variable Not Applied

**Issue:** Environment variable set but not taking effect

**Solutions:**
1. Verify variable name (must be `ATLAS_*`)
2. Check value format (e.g., "true" not "True")
3. Ensure no CLI flags are overriding

### Permission Issues

```
Error: Failed to read configuration file: Permission denied
```

**Solutions:**
- Check file permissions: `chmod 644 atlas.toml`
- Ensure `~/.atlas/` directory exists: `mkdir -p ~/.atlas`
- Verify user has read access to config file

## Advanced Usage

### Precedence Testing

To see which configuration is being used:

```bash
# Check effective configuration
atlas run main.atl  # Uses merged config

# Override with environment variable
ATLAS_EDITION=2027 atlas run main.atl

# Override with CLI flag (not all settings have CLI flags)
atlas run main.atl --json  # Overrides diagnostic format
```

### Multiple Projects

```
~/projects/
├── project-a/
│   └── atlas.toml  # Uses edition 2026
└── project-b/
    └── atlas.toml  # Uses edition 2027
```

Each project has independent configuration. Global config (`~/.atlas/config.toml`) provides defaults for both.

### Migration Between Editions

When upgrading to a new edition:

```toml
[package]
name = "my-project"
version = "2.0.0"
edition = "2027"  # Update edition
```

Run tests to ensure compatibility with new edition.

## Best Practices

1. **Always specify edition** - Avoid relying on defaults
2. **Version control atlas.toml** - Keep configuration in git
3. **Don't commit global config** - User preferences are personal
4. **Use security configs** - Explicitly define permissions
5. **Document custom settings** - Comment non-obvious configurations
6. **Test with defaults** - Ensure project works without global config

## Related Documentation

- [Security Model](security-model.md)
- [Package Manifest](package-manifest.md)
- [Build System](build-system.md)

## Version History

- **v0.2.0** - Configuration system implemented
  - Project config (atlas.toml)
  - Global config (~/.atlas/config.toml)
  - Environment variable overrides
  - Security configuration
  - Hierarchical precedence
