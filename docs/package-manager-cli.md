# Atlas Package Manager CLI

The Atlas package manager provides npm-like dependency management for Atlas projects. This document covers all package management commands and their usage.

## Quick Start

```bash
# Create a new project
atlas init my-project
cd my-project

# Add dependencies
atlas add http@1.0
atlas add test-utils --dev

# Install all dependencies
atlas install

# Update dependencies
atlas update

# Remove a dependency
atlas remove http
```

## Commands

### `atlas init`

Initialize a new Atlas project with the standard directory structure.

#### Usage

```bash
atlas init [OPTIONS] [NAME]
```

#### Arguments

| Argument | Description |
|----------|-------------|
| `NAME` | Project name (defaults to directory name) |

#### Options

| Option | Description |
|--------|-------------|
| `--lib` | Create a library project instead of binary |
| `--no-git` | Skip git repository initialization |
| `-v, --verbose` | Verbose output |

#### Examples

```bash
# Initialize in current directory (interactive)
atlas init

# Create a new project with explicit name
atlas init my-app

# Create a library project
atlas init my-lib --lib

# Skip git initialization
atlas init my-project --no-git
```

#### Generated Structure

**Binary project:**
```
my-project/
├── atlas.toml
├── src/
│   └── main.atl
└── .gitignore
```

**Library project:**
```
my-lib/
├── atlas.toml
├── src/
│   └── lib.atl
└── .gitignore
```

---

### `atlas add`

Add a dependency to your project.

#### Usage

```bash
atlas add <PACKAGE> [OPTIONS]
```

#### Arguments

| Argument | Description |
|----------|-------------|
| `PACKAGE` | Package name (optionally with @version) |

#### Options

| Option | Description |
|--------|-------------|
| `--ver <VER>` | Version constraint (e.g., "1.0", "^1.2.3") |
| `--dev` | Add as dev dependency |
| `--git <URL>` | Git repository URL |
| `--branch <BRANCH>` | Git branch |
| `--tag <TAG>` | Git tag |
| `--rev <REV>` | Git revision |
| `--path <PATH>` | Local path dependency |
| `-F, --features <FEAT>` | Enable specific features (repeatable) |
| `--no-default-features` | Disable default features |
| `--optional` | Mark as optional dependency |
| `--rename <NAME>` | Rename the dependency |
| `--dry-run` | Show what would be done |

#### Examples

```bash
# Add latest version
atlas add http

# Add specific version
atlas add http@1.2.3
atlas add http --ver "^1.0"

# Add as dev dependency
atlas add test-framework --dev

# Add from git
atlas add mylib --git https://github.com/user/mylib --branch main

# Add local path dependency
atlas add mylib --path ../mylib

# Add with features
atlas add http -F async -F json

# Add optional dependency
atlas add fancy-logger --optional

# Rename a dependency
atlas add http --rename my-http
```

#### Version Constraints

| Syntax | Description |
|--------|-------------|
| `1.0.0` | Exact version |
| `^1.0.0` | Compatible with 1.x.x |
| `~1.0.0` | Compatible with 1.0.x |
| `>=1.0.0` | Greater than or equal |
| `*` | Any version |

---

### `atlas remove`

Remove one or more dependencies from your project.

#### Usage

```bash
atlas remove <PACKAGES>... [OPTIONS]
```

#### Arguments

| Argument | Description |
|----------|-------------|
| `PACKAGES` | Package names to remove |

#### Options

| Option | Description |
|--------|-------------|
| `--dev` | Remove from dev dependencies |
| `--dry-run` | Show what would be done |
| `-v, --verbose` | Verbose output |

#### Examples

```bash
# Remove single dependency
atlas remove http

# Remove multiple dependencies
atlas remove http json logger

# Remove dev dependency
atlas remove test-framework --dev

# Preview removal
atlas remove http --dry-run
```

---

### `atlas install`

Install all dependencies from atlas.toml.

#### Usage

```bash
atlas install [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--production` | Only install production dependencies |
| `--force` | Force reinstall even if cached |
| `--dry-run` | Show what would be installed |
| `-v, --verbose` | Verbose output |
| `-q, --quiet` | Quiet output (errors only) |

#### Examples

```bash
# Install all dependencies
atlas install

# Install production only (skip dev)
atlas install --production

# Force reinstall
atlas install --force

# Preview installation
atlas install --dry-run

# Quiet mode for scripts
atlas install --quiet
```

#### Behavior

1. Reads dependencies from `atlas.toml`
2. Uses `atlas.lock` for reproducible builds (if present)
3. Downloads packages to `atlas_modules/`
4. Creates/updates `atlas.lock`

---

### `atlas update`

Update dependencies to their latest compatible versions.

#### Usage

```bash
atlas update [PACKAGES]... [OPTIONS]
```

#### Arguments

| Argument | Description |
|----------|-------------|
| `PACKAGES` | Specific packages to update (empty = all) |

#### Options

| Option | Description |
|--------|-------------|
| `--dev` | Only update dev dependencies |
| `--dry-run` | Show what would be updated |
| `-v, --verbose` | Verbose output |

#### Examples

```bash
# Update all dependencies
atlas update

# Update specific package
atlas update http

# Update multiple packages
atlas update http json

# Preview updates
atlas update --dry-run
```

---

### `atlas publish`

Publish your package to the Atlas registry.

#### Usage

```bash
atlas publish [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--registry <URL>` | Registry to publish to |
| `--no-verify` | Skip validation checks |
| `--dry-run` | Validate without publishing |
| `--allow-dirty` | Allow publishing with dirty git state |
| `-v, --verbose` | Verbose output |

#### Examples

```bash
# Publish to default registry
atlas publish

# Validate without publishing
atlas publish --dry-run

# Skip verification
atlas publish --no-verify

# Publish with uncommitted changes
atlas publish --allow-dirty
```

#### Publishing Steps

1. **Manifest validation** - Verify atlas.toml is valid
2. **Git status check** - Ensure clean working directory
3. **Structure verification** - Check required files exist
4. **Build** - Compile the package
5. **Tests** - Run test suite
6. **Package** - Create archive
7. **Upload** - Push to registry

---

## Project Files

### atlas.toml

The project manifest file.

```toml
[package]
name = "my-package"
version = "1.0.0"
description = "My awesome package"
authors = ["You <you@example.com>"]
license = "MIT"

[dependencies]
http = "1.0"
json = { version = "2.0", features = ["async"] }
local-lib = { path = "../local-lib" }
git-lib = { git = "https://github.com/user/lib", branch = "main" }

[dev-dependencies]
test-utils = "0.1"

[[bin]]
name = "my-app"
path = "src/main.atl"

[lib]
path = "src/lib.atl"

[features]
async = []
full = ["async"]
```

### atlas.lock

The lockfile for reproducible builds (auto-generated).

```toml
version = 1

[[packages]]
name = "http"
version = "1.2.3"
checksum = "abc123..."

[packages.source]
type = "registry"

[[packages]]
name = "json"
version = "2.0.0"

[packages.source]
type = "registry"
```

### atlas_modules/

Directory containing installed packages (auto-generated).

---

## Configuration

### Environment Variables

| Variable | Description |
|----------|-------------|
| `ATLAS_REGISTRY` | Default registry URL |
| `ATLAS_CACHE_DIR` | Package cache directory |
| `ATLAS_OFFLINE` | Run in offline mode |

---

## Best Practices

### Version Constraints

- Use `^` for most dependencies (compatible updates)
- Use `~` for patch-level stability
- Use exact versions for critical dependencies
- Commit `atlas.lock` for applications
- Consider not committing `atlas.lock` for libraries

### Dependency Hygiene

```bash
# Review dependencies regularly
atlas update --dry-run

# Remove unused dependencies
atlas remove unused-dep

# Keep dev dependencies separate
atlas add test-lib --dev
```

### CI/CD Integration

```yaml
# Example CI workflow
- name: Install dependencies
  run: atlas install --production

- name: Build
  run: atlas build --release

- name: Test
  run: atlas test
```

---

## Troubleshooting

### Common Issues

**"Could not find atlas.toml"**
- Run command from project root
- Ensure atlas.toml exists

**"Version conflict"**
- Check version constraints in atlas.toml
- Run `atlas update` to resolve

**"Package not found"**
- Verify package name spelling
- Check registry connectivity

### Getting Help

```bash
atlas init --help
atlas add --help
atlas install --help
atlas update --help
atlas publish --help
```

---

## Command Aliases

| Alias | Command |
|-------|---------|
| `atlas i` | `atlas init` |
| `atlas rm` | `atlas remove` |
| `atlas up` | `atlas update` |

---

## See Also

- [Atlas Language Specification](specification/)
- [Build System Documentation](build-system.md)
- [Configuration Reference](configuration.md)
