# Package Manifest System

Atlas uses `atlas.toml` manifest files for package configuration and dependency management, similar to Cargo, npm, and other modern package managers.

## Table of Contents

1. [Manifest Structure](#manifest-structure)
2. [Package Metadata](#package-metadata)
3. [Dependencies](#dependencies)
4. [Features](#features)
5. [Build Configuration](#build-configuration)
6. [Workspaces](#workspaces)
7. [Lockfile (atlas.lock)](#lockfile)
8. [Version Constraints](#version-constraints)
9. [Validation](#validation)
10. [Examples](#examples)

## Manifest Structure

An `atlas.toml` manifest contains package metadata, dependencies, optional features, and build configuration.

### Minimal Example

```toml
[package]
name = "my-package"
version = "1.0.0"
```

### Complete Example

```toml
[package]
name = "my-package"
version = "1.2.3"
description = "A complete package example"
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/my-package"
homepage = "https://example.com"
keywords = ["atlas", "example", "package"]
categories = ["development-tools"]

[dependencies]
foo = "1.0"
bar = { version = "2.0", optional = true }

[dev-dependencies]
test-utils = "0.1"

[build]
optimize = "size"
target = "wasm32"

[lib]
path = "src/lib.atl"
name = "mylib"

[[bin]]
name = "mybinary"
path = "src/main.atl"

[features]
default = { dependencies = [], default = true }
extra = { dependencies = ["bar/feature"], default = false }

[workspace]
members = ["packages/*"]
exclude = ["packages/experimental"]
```

## Package Metadata

The `[package]` section contains core package information.

### Required Fields

- **name**: Package name (lowercase, alphanumeric, hyphens, underscores)
  - Must start with lowercase letter or digit
  - Cannot start/end with hyphen or underscore
  - No consecutive hyphens or underscores
  - Maximum 64 characters

- **version**: Semantic version (e.g., "1.2.3")
  - Must follow [semver](https://semver.org/) specification
  - Format: MAJOR.MINOR.PATCH

### Optional Fields

- **description**: Brief package description
- **authors**: List of package authors (format: "Name <email>")
- **license**: License identifier (SPDX format)
- **repository**: Source code repository URL
- **homepage**: Package homepage URL
- **keywords**: Search keywords (lowercase)
- **categories**: Package categories

## Dependencies

Dependencies are packages required to build and run your code.

### Simple Dependencies

```toml
[dependencies]
foo = "1.0"
bar = "^2.0"
baz = "~1.2.3"
```

### Detailed Dependencies

```toml
[dependencies]
# Registry dependency with features
foo = { version = "1.0", features = ["feature1", "feature2"], default-features = false }

# Git dependency
git-dep = { git = "https://github.com/example/repo", branch = "main" }

# Path dependency
local = { path = "../local-package" }

# Renamed dependency
new-name = { version = "1.0", package = "old-name" }

# Optional dependency
optional-dep = { version = "1.0", optional = true }
```

### Dev Dependencies

Dependencies only needed for testing and development:

```toml
[dev-dependencies]
test-utils = "0.1"
benchmark = "1.0"
```

### Dependency Sources

1. **Registry** (default): Package from package registry
   - Requires `version` field
   - Optional `registry` field for custom registry

2. **Git**: Package from Git repository
   - Requires `git` URL
   - Requires one of: `branch`, `tag`, or `rev`
   - Cannot specify multiple Git references

3. **Path**: Local filesystem package
   - Requires `path` field
   - Useful for workspace members and local development

**Note**: Only ONE source type per dependency (version OR git OR path).

## Features

Features enable conditional compilation and optional dependencies.

```toml
[features]
# Default feature enabled automatically
default = { dependencies = ["feature1"], default = true }

# Feature enabling another feature
feature1 = { dependencies = ["feature2"], default = false }

# Feature enabling dependency feature
feature2 = { dependencies = ["foo/feature"], default = false }

[dependencies]
foo = { version = "1.0", optional = true }
```

### Feature Dependencies

Feature dependencies can reference:
- Other features: `"feature-name"`
- Dependency features: `"dependency-name/feature-name"`

## Build Configuration

Optional build settings for compilation and optimization.

```toml
[build]
optimize = "size"  # or "speed", "none"
target = "wasm32"  # Target platform

[build.scripts]
prebuild = "echo 'Running prebuild'"
postbuild = "echo 'Running postbuild'"
```

## Workspaces

Workspaces group multiple packages in a single repository.

```toml
[workspace]
members = [
    "packages/*",
    "tools/cli"
]
exclude = [
    "packages/experimental",
    "packages/archived"
]

# Shared dependencies across workspace
[workspace.dependencies]
shared-lib = "1.0"
```

### Workspace Member Patterns

- Exact paths: `"packages/foo"`
- Glob patterns: `"packages/*"`
- Subdirectories: `"tools/cli"`

## Lockfile

The lockfile (`atlas.lock`) records exact versions for reproducible builds.

### Lockfile Structure

```toml
version = 1

[[packages]]
name = "foo"
version = "1.2.3"
checksum = "abc123..."

[packages.source]
type = "registry"

[[packages]]
name = "git-dep"
version = "1.0.0"

[packages.source]
type = "git"
url = "https://github.com/example/repo"
rev = "abc123def456"

[metadata]
atlas_version = "0.1.0"
generated_at = "2025-01-15T12:00:00Z"
```

### Lockfile Sources

1. **Registry**:
   ```toml
   [packages.source]
   type = "registry"
   registry = "https://registry.example.com"  # optional
   ```

2. **Git**:
   ```toml
   [packages.source]
   type = "git"
   url = "https://github.com/example/repo"
   rev = "abc123def456"  # resolved commit hash
   ```

3. **Path**:
   ```toml
   [packages.source]
   type = "path"
   path = "../local-package"
   ```

### Lockfile Operations

- Generated automatically when resolving dependencies
- Should be committed to version control for applications
- Should NOT be committed for libraries
- Updated when dependencies change

## Version Constraints

Atlas supports multiple version constraint formats:

### Exact Version

```toml
foo = "1.2.3"
```
Matches exactly version 1.2.3.

### Caret (^) - Semver Compatible

```toml
foo = "^1.2.3"
```
- Matches: >=1.2.3, <2.0.0
- Allows: patch and minor updates
- Example: 1.2.3, 1.2.4, 1.9.9 ✓ | 2.0.0 ✗

Special case for 0.x.y versions:
```toml
foo = "^0.1.2"
```
- Matches: >=0.1.2, <0.2.0
- More restrictive for pre-1.0 versions

### Tilde (~) - Patch Compatible

```toml
foo = "~1.2.3"
```
- Matches: >=1.2.3, <1.3.0
- Allows: only patch updates
- Example: 1.2.3, 1.2.4, 1.2.99 ✓ | 1.3.0 ✗

### Range Constraints

```toml
# Greater than or equal
foo = ">=1.0.0"

# Less than
bar = "<2.0.0"

# Combined
baz = ">=1.0.0, <2.0.0"
```

### Wildcard

```toml
foo = "*"
```
Matches any version (not recommended for production).

## Validation

The validator checks manifests for correctness.

### Package Name Rules

- Lowercase letters, digits, hyphens, underscores only
- Must start with lowercase letter or digit
- Cannot start/end with hyphen or underscore
- No consecutive hyphens or underscores
- Maximum 64 characters

**Valid**: `my-package`, `my_package`, `package123`
**Invalid**: `_package`, `my..package`, `My-Package`

### Dependency Rules

1. **Source requirement**: Must specify exactly one source (version, git, or path)
2. **Git requirements**: Git dependencies must specify branch, tag, or rev (only one)
3. **Version format**: Version constraints must be valid semver
4. **No circular dependencies**: Dependency graph must be acyclic

### Feature Rules

1. **Valid names**: Alphanumeric, hyphens, underscores only
2. **Valid references**: Feature dependencies must reference existing features or dependencies
3. **Dependency features**: Format `"dep-name/feature"` requires `dep-name` in dependencies

### Workspace Rules

1. **Non-empty members**: Workspace must have at least one member
2. **Valid paths**: Member and exclude paths cannot be empty

## Examples

### Library Package

```toml
[package]
name = "my-lib"
version = "1.0.0"
description = "A reusable library"
license = "MIT"

[lib]
path = "src/lib.atl"

[dependencies]
util = "^1.0"
```

### Binary Package

```toml
[package]
name = "my-app"
version = "0.1.0"

[[bin]]
name = "my-app"
path = "src/main.atl"

[dependencies]
cli-framework = "2.0"
```

### Package with Features

```toml
[package]
name = "flexible-lib"
version = "1.0.0"

[dependencies]
required = "1.0"
optional-dep = { version = "1.0", optional = true }

[features]
default = { dependencies = ["std"], default = true }
std = { dependencies = [], default = false }
extra = { dependencies = ["optional-dep/feature"], default = false }
```

### Monorepo Workspace

```toml
[package]
name = "workspace-root"
version = "1.0.0"

[workspace]
members = [
    "packages/core",
    "packages/cli",
    "packages/utils"
]

[workspace.dependencies]
shared-types = "1.0"
```

## API Reference

### PackageManifest

Core manifest structure:

```rust
pub struct PackageManifest {
    pub package: PackageMetadata,
    pub dependencies: HashMap<String, Dependency>,
    pub dev_dependencies: HashMap<String, Dependency>,
    pub build: Option<BuildConfig>,
    pub lib: Option<LibConfig>,
    pub bin: Vec<BinConfig>,
    pub features: HashMap<String, Feature>,
    pub workspace: Option<Workspace>,
}

impl PackageManifest {
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error>;
    pub fn from_file(path: &Path) -> Result<Self>;
    pub fn to_string(&self) -> Result<String, toml::ser::Error>;
}
```

### Validator

Manifest validation:

```rust
pub struct Validator;

impl Validator {
    pub fn validate(manifest: &PackageManifest) -> Result<(), Vec<ValidationError>>;
    pub fn validate_package_name(name: &str) -> Result<(), ValidationError>;
}
```

### Lockfile

Dependency locking:

```rust
pub struct Lockfile {
    pub version: u32,
    pub packages: Vec<LockedPackage>,
    pub metadata: LockfileMetadata,
}

impl Lockfile {
    pub fn new() -> Self;
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error>;
    pub fn from_file(path: &Path) -> Result<Self>;
    pub fn write_to_file(&self, path: &Path) -> Result<()>;
    pub fn add_package(&mut self, package: LockedPackage);
    pub fn get_package(&self, name: &str) -> Option<&LockedPackage>;
    pub fn verify(&self) -> Result<(), String>;
}
```

### Resolver

Dependency resolution:

```rust
pub struct Resolver;

impl Resolver {
    pub fn resolve(manifest: &PackageManifest) -> Result<Lockfile>;
}
```

## Best Practices

1. **Version constraints**: Use caret (^) for most dependencies, exact versions for critical stability
2. **Lockfiles**: Commit lockfiles for applications, ignore for libraries
3. **Features**: Use features for optional functionality, not for platform-specific code
4. **Workspaces**: Use workspaces for monorepos with shared dependencies
5. **Validation**: Run validation before publishing packages
6. **Documentation**: Document feature flags and optional dependencies
7. **Semver**: Follow semantic versioning for version bumps

## CLI Integration

Package manifest commands (future):

```bash
# Initialize new package
atlas init --name my-package

# Add dependency
atlas add foo@^1.0

# Update dependencies
atlas update

# Generate lockfile
atlas lock

# Validate manifest
atlas validate
```

## See Also

- [Semantic Versioning](https://semver.org/)
- [SPDX License List](https://spdx.org/licenses/)
- [TOML Specification](https://toml.io/)
- [Cargo Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html) (inspiration)
