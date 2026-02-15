# Dependency Resolution

Atlas uses a production-grade dependency resolver based on the PubGrub algorithm to find compatible package versions and manage dependencies.

## Table of Contents

1. [Overview](#overview)
2. [Resolution Algorithm](#resolution-algorithm)
3. [Version Constraints](#version-constraints)
4. [Conflict Resolution](#conflict-resolution)
5. [Build Order](#build-order)
6. [Lockfile Integration](#lockfile-integration)
7. [Registry System](#registry-system)
8. [Caching](#caching)
9. [Offline Mode](#offline-mode)
10. [API Reference](#api-reference)
11. [Examples](#examples)
12. [Troubleshooting](#troubleshooting)

---

## Overview

The Atlas package manager provides complete dependency management with:

- **PubGrub algorithm**: State-of-the-art version resolution (used by Dart, Swift, Poetry)
- **Lockfile support**: Reproducible builds with `atlas.lock`
- **Conflict detection**: Clear error messages with resolution suggestions
- **Build order computation**: Topological sorting with parallel build groups
- **Multiple registries**: HTTP, local filesystem, and git sources
- **SHA256 verification**: Package integrity checks
- **LRU caching**: Offline mode support
- **Transitive dependencies**: Automatic resolution of nested dependencies

### Key Components

```
Package Manifest (atlas.toml)
         ↓
    Resolver (PubGrub)
         ↓
    Resolution (exact versions)
         ↓
    Lockfile (atlas.lock)
         ↓
    Build Order Computation
         ↓
    Package Download & Cache
```

---

## Resolution Algorithm

### PubGrub Algorithm

Atlas uses the PubGrub algorithm, which provides:

1. **Completeness**: Always finds a solution if one exists
2. **Minimality**: Returns the smallest solution
3. **Clarity**: Clear conflict explanations when no solution exists

**How it works:**

```
1. Start with root package constraints
2. For each package, find compatible versions
3. Add transitive dependencies
4. Detect version conflicts
5. If conflict: backtrack and try alternative versions
6. If no conflicts: return resolution
```

### Version Solver

The `VersionSolver` maintains available versions for each package:

```rust
use atlas_package::VersionSolver;

let mut solver = VersionSolver::new();
solver.add_package_versions("serde", vec![
    Version::new(1, 0, 0),
    Version::new(1, 1, 0),
    Version::new(2, 0, 0),
]);

// Find maximum version satisfying ^1.0
let version = solver.max_satisfying_version("serde", &[
    "^1.0".parse().unwrap()
]);
// Returns Some(1.1.0)
```

### Dependency Graph

The `DependencyGraph` tracks package relationships:

```rust
use atlas_package::DependencyGraph;
use semver::Version;

let mut graph = DependencyGraph::new();
graph.add_package("myapp".to_string(), Version::new(1, 0, 0));
graph.add_package("serde".to_string(), Version::new(1, 0, 0));
graph.add_edge("myapp", "serde")?;

// Get topological build order
let order = graph.topological_sort()?;
// Returns ["serde", "myapp"]
```

---

## Version Constraints

### Supported Formats

Atlas uses semantic versioning (semver) with the following constraint formats:

#### Caret Requirements (`^`)

Match on major version (or minor for 0.x):

```toml
# ^1.2.3 matches >=1.2.3, <2.0.0
serde = "^1.0"

# ^0.2.3 matches >=0.2.3, <0.3.0 (0.x is special)
alpha-lib = "^0.2"
```

#### Tilde Requirements (`~`)

Match on minor version:

```toml
# ~1.2.3 matches >=1.2.3, <1.3.0
tokio = "~1.2"
```

#### Exact Requirements (`=`)

Exact version match:

```toml
legacy-lib = "=2.1.0"
```

#### Wildcard Requirements (`*`)

Accept any version:

```toml
test-utils = "*"
```

#### Comparison Requirements

Use comparison operators:

```toml
newer-lib = ">= 2.0, < 3.0"
compatible = ">=1.0, <2.0"
```

### Constraint Compatibility

Version constraints are **compatible** if they overlap:

```
^1.0 and ^1.1  →  Compatible (1.1.x satisfies both)
^1.0 and ^2.0  →  Incompatible (no overlap)
~1.2 and ~1.3  →  Incompatible (different minors)
>=1.0, <2.0 and ^1.5  →  Compatible (1.5.x-1.9.x overlap)
```

---

## Conflict Resolution

### Detecting Conflicts

The `ConflictResolver` detects incompatible version constraints:

```rust
use atlas_package::{ConflictResolver, VersionConstraint};
use std::collections::HashMap;

let mut resolver = ConflictResolver::new();
let mut constraints = HashMap::new();

constraints.insert("serde".to_string(), vec![
    VersionConstraint {
        requirement: "^1.0".parse().unwrap(),
        source: "myapp".to_string(),
    },
    VersionConstraint {
        requirement: "^2.0".parse().unwrap(),
        source: "other-lib".to_string(),
    },
]);

let conflicts = resolver.detect_conflicts(&constraints);
// Returns conflict: serde ^1.0 (myapp) vs ^2.0 (other-lib)
```

### Conflict Reports

Conflicts generate human-readable error messages:

```
Version conflict for package 'serde':
  myapp requires ^1.0
  other-lib requires ^2.0

Possible solutions:
  1. Update dependencies of 'serde' to latest compatible versions
  2. Use dependency overrides in atlas.toml
  3. Check for alternative packages
```

### Resolution Suggestions

The conflict resolver provides actionable suggestions:

```rust
let suggestions = resolver.suggest_resolutions(&conflict);
// Returns:
// - "Try updating all dependencies of 'serde' to latest compatible versions"
// - "Consider checking if myapp and other-lib can use compatible version ranges"
// - "Check if a newer version of 'serde' exists that satisfies all constraints"
// - "As a last resort, use [dependencies.overrides] in atlas.toml..."
```

### Dependency Overrides

Force specific versions when needed:

```toml
[dependencies]
mylib = "^1.0"

[dependencies.overrides]
# Force serde to specific version despite conflicts
serde = "=1.0.200"
```

---

## Build Order

### Topological Sort

The `BuildOrderComputer` calculates correct build order using Kahn's algorithm:

```rust
use atlas_package::{BuildOrderComputer, Resolution};

let computer = BuildOrderComputer::new(&resolution);
let order = computer.compute_build_order()?;

// Example output: ["base-lib", "mid-lib", "app"]
// Guarantees dependencies are built before dependents
```

### Parallel Build Groups

Identify packages that can be built in parallel:

```rust
let groups = computer.parallel_build_groups()?;

// Example:
// Group 1: ["base-lib1", "base-lib2"]  // No dependencies, build in parallel
// Group 2: ["mid-lib"]                 // Depends on group 1
// Group 3: ["app"]                     // Depends on group 2
```

**Benefits:**

- Faster builds (parallelize independent packages)
- Optimal resource utilization
- Clear dependency visualization

### Circular Dependency Detection

The build order computation detects cycles:

```
Error: Circular dependency detected: Dependency cycle detected in graph

Packages involved:
  pkg-a → pkg-b → pkg-c → pkg-a

Resolution:
  Break the cycle by refactoring package boundaries
```

---

## Lockfile Integration

### Lockfile Format

`atlas.lock` stores resolved versions for reproducible builds:

```toml
version = 1

[[packages]]
name = "serde"
version = "1.0.200"
checksum = "abc123..."

[packages.source]
type = "registry"

[packages.dependencies]
serde_derive = "1.0.200"

[metadata]
generated_at = "2026-02-15T10:30:00Z"
atlas_version = "0.1.0"
```

### Using Lockfiles

The resolver integrates lockfiles for consistent builds:

```rust
use atlas_package::{Resolver, Lockfile};

let mut resolver = Resolver::new();
let lockfile = Lockfile::from_file("atlas.lock".as_ref())?;

// Use lockfile if valid, otherwise re-resolve
let resolution = resolver.resolve_with_lockfile(
    &manifest,
    Some(&lockfile)
)?;
```

**Lockfile validation checks:**

1. Lockfile integrity (no duplicates, correct format)
2. All manifest dependencies present in lockfile
3. Locked versions satisfy manifest constraints

### Generating Lockfiles

```rust
let lockfile = resolver.generate_lockfile(&resolution);
lockfile.write_to_file("atlas.lock".as_ref())?;
```

**Generated lockfile includes:**

- Exact resolved versions
- SHA256 checksums (when available)
- Dependency relationships
- Source information (registry/git/path)
- Generation metadata

### Lockfile Updates

**When lockfiles are regenerated:**

1. Manifest dependencies change
2. Lockfile validation fails
3. Explicit `atlas update` command
4. Lockfile version mismatch

**When lockfiles are reused:**

1. Lockfile valid and satisfies constraints
2. No manifest changes
3. Offline mode enabled

---

## Registry System

### Registry Trait

All registries implement the `Registry` trait:

```rust
pub trait Registry {
    fn fetch_metadata(&self, package: &str) -> RegistryResult<PackageMetadata>;
    fn available_versions(&self, package: &str) -> RegistryResult<Vec<Version>>;
}
```

### HTTP Registry

Connect to remote package registries:

```rust
use atlas_package::RemoteRegistry;

let registry = RemoteRegistry::new("https://packages.atlas-lang.org")?;
let metadata = registry.fetch_metadata("serde")?;
let versions = registry.available_versions("serde")?;
```

**Features:**

- HTTP/HTTPS support
- Retry logic
- Timeout configuration
- Connection pooling

### Local Filesystem Registry

Use local package directories:

```rust
use atlas_package::LocalRegistry;

let registry = LocalRegistry::new("/path/to/packages")?;
let metadata = registry.fetch_metadata("my-local-lib")?;
```

**Use cases:**

- Offline development
- Corporate mirrors
- Testing
- CI/CD caching

### Registry Manager

Manage multiple registries with fallback:

```rust
use atlas_package::RegistryManager;

let mut manager = RegistryManager::new();
manager.add_registry(Box::new(remote_registry));
manager.add_registry(Box::new(local_registry));

// Tries registries in order until success
let metadata = manager.fetch_metadata("serde")?;
```

---

## Caching

### Package Cache

LRU cache with size limits:

```rust
use atlas_package::PackageCache;

let cache = PackageCache::new("/path/to/cache", 1024 * 1024 * 1024)?; // 1GB
cache.store("serde", &Version::new(1, 0, 0), package_data)?;

if let Some(data) = cache.get("serde", &Version::new(1, 0, 0))? {
    // Use cached package
}
```

**Features:**

- LRU eviction policy
- Configurable size limits
- Atomic operations
- Corruption detection

### Cache Strategy

```
1. Check local cache
   ↓
2. If miss, download from registry
   ↓
3. Verify checksum
   ↓
4. Store in cache
   ↓
5. Extract to target directory
```

---

## Offline Mode

### Enabling Offline Mode

Use cached packages without network access:

```rust
let manager = RegistryManager::offline_only(cache);
let resolution = resolver.resolve(&manifest)?; // Uses cache only
```

**Requirements:**

1. Packages already in cache
2. All dependencies cached
3. Valid checksums

**Failures:**

- Missing packages → error
- Checksum mismatch → error
- Corrupted cache → error

---

## API Reference

### Core Types

#### Resolver

```rust
pub struct Resolver {
    // Internal state
}

impl Resolver {
    pub fn new() -> Self;
    pub fn resolve(&mut self, manifest: &PackageManifest) -> ResolverResult<Resolution>;
    pub fn resolve_with_lockfile(&mut self, manifest: &PackageManifest, lockfile: Option<&Lockfile>) -> ResolverResult<Resolution>;
    pub fn generate_lockfile(&self, resolution: &Resolution) -> Lockfile;
    pub fn compute_build_order(&self) -> ResolverResult<Vec<String>>;
}
```

#### Resolution

```rust
pub struct Resolution {
    pub packages: HashMap<String, ResolvedPackage>,
}

pub struct ResolvedPackage {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<String>,
}
```

#### ConflictResolver

```rust
pub struct ConflictResolver;

impl ConflictResolver {
    pub fn new() -> Self;
    pub fn detect_conflicts(&mut self, constraints: &HashMap<String, Vec<VersionConstraint>>) -> Vec<Conflict>;
    pub fn suggest_resolutions(&self, conflict: &Conflict) -> Vec<String>;
}

pub struct Conflict {
    pub package: String,
    pub constraints: Vec<ConflictingConstraint>,
}
```

#### BuildOrderComputer

```rust
pub struct BuildOrderComputer;

impl BuildOrderComputer {
    pub fn new(resolution: &Resolution) -> Self;
    pub fn compute_build_order(&self) -> BuildOrderResult<Vec<String>>;
    pub fn parallel_build_groups(&self) -> BuildOrderResult<Vec<Vec<String>>>;
}
```

---

## Examples

### Basic Dependency Resolution

```rust
use atlas_package::{Resolver, PackageManifest};

// Load manifest
let manifest = PackageManifest::from_file("atlas.toml".as_ref())?;

// Resolve dependencies
let mut resolver = Resolver::new();
let resolution = resolver.resolve(&manifest)?;

// Generate lockfile
let lockfile = resolver.generate_lockfile(&resolution);
lockfile.write_to_file("atlas.lock".as_ref())?;

println!("Resolved {} packages", resolution.packages.len());
```

### With Lockfile

```rust
use atlas_package::{Resolver, PackageManifest, Lockfile};

let manifest = PackageManifest::from_file("atlas.toml".as_ref())?;
let lockfile = Lockfile::from_file("atlas.lock".as_ref()).ok();

let mut resolver = Resolver::new();
let resolution = resolver.resolve_with_lockfile(&manifest, lockfile.as_ref())?;

// Resolution uses lockfile if valid, otherwise re-resolves
```

### Conflict Detection

```rust
use atlas_package::{Resolver, ConflictResolver};

let mut resolver = Resolver::new();
match resolver.resolve(&manifest) {
    Ok(resolution) => println!("Success!"),
    Err(e) => {
        // Check for conflicts
        let mut conflict_resolver = ConflictResolver::new();
        let conflicts = conflict_resolver.detect_conflicts(&resolver.get_all_constraints());

        for conflict in conflicts {
            println!("{}", conflict.report());
            for suggestion in conflict_resolver.suggest_resolutions(&conflict) {
                println!("  - {}", suggestion);
            }
        }
    }
}
```

### Build Order Computation

```rust
use atlas_package::{BuildOrderComputer, Resolver};

let mut resolver = Resolver::new();
let resolution = resolver.resolve(&manifest)?;

let computer = BuildOrderComputer::new(&resolution);

// Sequential build order
let order = computer.compute_build_order()?;
println!("Build order: {:?}", order);

// Parallel build groups
let groups = computer.parallel_build_groups()?;
for (i, group) in groups.iter().enumerate() {
    println!("Group {}: {:?} (can build in parallel)", i+1, group);
}
```

### Multiple Registries

```rust
use atlas_package::{RemoteRegistry, LocalRegistry, RegistryManager};

let remote = RemoteRegistry::new("https://packages.atlas-lang.org")?;
let local = LocalRegistry::new("/path/to/mirror")?;

let mut manager = RegistryManager::new();
manager.add_registry(Box::new(local));  // Try local first
manager.add_registry(Box::new(remote)); // Fallback to remote

// Resolver will try local, then remote
```

### Caching

```rust
use atlas_package::{PackageCache, Downloader};

let cache = PackageCache::new("/tmp/atlas-cache", 1024 * 1024 * 1024)?;
let downloader = Downloader::new(cache);

// Download and cache
downloader.download("serde", &Version::new(1, 0, 0), registry)?;

// Future requests use cache
```

---

## Troubleshooting

### Common Errors

#### Version Conflict

**Error:**
```
No version of package 'serde' satisfies constraints: ^1.0 (from myapp), ^2.0 (from other-lib)
```

**Solutions:**

1. Update dependencies to compatible versions
2. Check if newer versions exist
3. Use dependency overrides (last resort)

#### Circular Dependency

**Error:**
```
Circular dependency detected: pkg-a → pkg-b → pkg-a
```

**Solutions:**

1. Refactor package boundaries
2. Extract shared code to new package
3. Break cycle with dependency injection

#### Missing Package

**Error:**
```
Package not found: unknown-pkg
```

**Solutions:**

1. Check package name spelling
2. Verify registry configuration
3. Check network connectivity (remote registry)
4. Ensure package exists in configured registries

#### Checksum Mismatch

**Error:**
```
Checksum verification failed for serde@1.0.0
```

**Solutions:**

1. Clear cache and re-download
2. Check for corrupted download
3. Verify registry integrity
4. Report to package maintainer

### Debug Mode

Enable verbose logging:

```rust
env::set_var("ATLAS_LOG", "debug");
```

### Clearing Cache

```bash
rm -rf ~/.atlas/cache
```

### Regenerating Lockfile

```bash
rm atlas.lock
atlas build  # Generates new lockfile
```

---

## Performance

### Resolution Complexity

- **Best case**: O(n) where n = number of packages
- **Average case**: O(n log n)
- **Worst case**: O(n²) with many conflicts and backtracking

### Optimization Tips

1. **Use lockfiles**: Skip resolution when possible
2. **Cache packages**: Enable LRU cache for offline mode
3. **Local registry**: Mirror commonly used packages
4. **Parallel builds**: Use `parallel_build_groups()` for faster builds

---

## Implementation Details

### PubGrub State Machine

```
State: Partial Solution
  ↓
Add next package decision
  ↓
Unit propagation (derive forced choices)
  ↓
Conflict? → Backtrack and try alternative
No conflict? → Continue
  ↓
All packages decided? → Done
```

### Lockfile Version

Current lockfile version: `1`

**Version 1 format:**

- Package list with exact versions
- SHA256 checksums
- Dependency graph
- Source metadata
- Generation timestamp

Future versions may add:

- Platform-specific dependencies
- Optional features tracking
- Build script hashes

---

## Security

### Checksum Verification

All downloaded packages verified with SHA256:

```rust
downloader.download_with_checksum(
    "serde",
    &version,
    "abc123...", // Expected checksum
    registry
)?;
```

### Registry HTTPS

Remote registries enforce HTTPS:

```rust
RemoteRegistry::new("http://...")?;  // Error: HTTP not allowed
RemoteRegistry::new("https://...")?; // OK
```

### Sandbox

Future: Package installation sandboxing to prevent malicious scripts.

---

## Future Enhancements

Planned features for future versions:

1. **Git sources**: Direct git repository dependencies
2. **Path dependencies**: Local workspace packages
3. **Optional features**: Cargo-style feature flags
4. **Platform-specific deps**: OS/arch conditional dependencies
5. **Build scripts**: Pre/post build hooks
6. **Workspace support**: Multi-package monorepos
7. **Patch versions**: Temporary package patches

---

## See Also

- [Package Manifest Documentation](package-manifest.md)
- [Atlas Build System](build-system.md) (future)
- [CLI Package Commands](cli-package-manager.md) (future)
- [Creating Packages](creating-packages.md) (future)

---

**Last Updated:** 2026-02-15
**Atlas Version:** 0.1.0
**Package Manager Version:** Phase-08c (Complete)
