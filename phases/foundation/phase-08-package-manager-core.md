# Phase 08: Package Manager - Dependency Resolution

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Package manifest system must be complete.

**Verification:**
```bash
ls crates/atlas-package/src/manifest.rs
cargo test --package atlas-package
grep -n "PackageManifest" crates/atlas-package/src/manifest.rs
```

**What's needed:**
- Package manifest from foundation/phase-07
- Lockfile support from foundation/phase-07
- Network access for future registry

**If missing:** Complete foundation/phase-07 first

---

## Objective
Implement dependency resolution engine finding compatible dependency versions using constraint satisfaction, resolving conflicts, and downloading packages - enabling automated dependency management for Atlas projects.

## Files
**Create:** `crates/atlas-package/src/resolver.rs` (~1000 lines)
**Create:** `crates/atlas-package/src/registry.rs` (~600 lines)
**Create:** `crates/atlas-package/src/downloader.rs` (~400 lines)
**Create:** `crates/atlas-package/src/cache.rs` (~300 lines)
**Update:** `crates/atlas-package/src/lib.rs` (~100 lines exports)
**Create:** `docs/dependency-resolution.md` (~600 lines)
**Tests:** `crates/atlas-package/tests/resolver_tests.rs` (~800 lines)
**Tests:** `crates/atlas-package/tests/registry_tests.rs` (~400 lines)

## Dependencies
- Package manifest system
- semver for version comparison
- pubgrub or similar SAT solver for resolution
- HTTP client for downloads (reqwest)
- tar/gzip for archive extraction

## Implementation

### Dependency Resolution Algorithm
Implement PubGrub algorithm or similar SAT solver for dependency resolution. Start with direct dependencies from manifest. For each dependency find available versions satisfying constraints. Recursively resolve transitive dependencies. Track all version constraints from dependency tree. Find versions satisfying all constraints simultaneously. Backtrack on conflicts trying alternative versions. Report unsolvable conflicts with explanation of conflicting requirements. Optimize resolution minimizing dependency updates. Cache resolution results for performance. Support pre-release versions with explicit opt-in.

### Version Constraint Solving
Parse semver version constraints from manifest. Support exact version matching with equals operator. Support range constraints greater-than, less-than, ranges. Implement caret ranges compatible with minor versions. Implement tilde ranges compatible with patch versions. Handle pre-release version semantics. Compare versions according to semver rules. Find maximum satisfying version within constraints. Report version conflicts with constraint sources.

### Registry Interface
Design registry abstraction supporting multiple sources. Primary registry for published packages. Git registry for packages in repositories. Local registry for testing and development. Filesystem registry for cached packages. Query registry for package metadata. Fetch available versions for package. Download package archives from registry. Verify package checksums from registry. Cache registry responses. Handle registry network errors gracefully. Support authenticated registries for private packages.

### Package Downloading and Caching
Download package archives from registry or git. Verify archive checksums ensuring integrity. Extract archives to cache directory. Cache packages by name and version. Deduplicate identical packages. Respect cache size limits with LRU eviction. Support offline mode using cache only. Clean cache removing old packages. Verify cached package integrity. Parallel downloads for performance.

### Lockfile Integration
Use existing lockfile for reproducible builds when available. Verify lockfile matches manifest dependencies. Update lockfile with new resolution. Preserve lockfile structure and comments. Detect lockfile drift from manifest. Generate lockfile from scratch if missing. Support lockfile migrations between versions. Validate lockfile integrity checksums.

### Conflict Resolution Strategies
Detect version conflicts from incompatible constraints. Report conflicts with dependency chain showing conflict source. Suggest conflict resolution strategies upgrade, downgrade, or remove dependency. Support dependency overrides forcing specific versions. Warn about override risks. Enable conflict visualization for debugging. Provide machine-readable conflict reports for tooling.

### Build Order Computation
Compute topological build order from dependency graph. Build dependencies before dependents. Support parallel builds of independent packages. Detect circular dependencies reporting error with cycle. Handle diamond dependencies building shared dependency once. Optimize build order for maximum parallelism. Support incremental builds skipping unchanged packages.

## Tests (TDD - Use rstest)

**Resolution algorithm tests:**
1. Resolve simple dependency tree
2. Resolve transitive dependencies
3. Find maximum compatible versions
4. Version conflict detected and reported
5. Backtracking on conflicts
6. Unsolvable conflict with explanation
7. Pre-release version handling
8. Cached resolution result reuse
9. Multiple resolution strategies
10. Complex dependency graph

**Version constraint tests:**
1. Exact version match
2. Caret range compatibility
3. Tilde range compatibility
4. Range constraints
5. Pre-release version semantics
6. Version comparison
7. Constraint intersection
8. Conflicting constraints
9. Empty constraint set error
10. Malformed version error

**Registry tests:**
1. Query package metadata
2. Fetch available versions
3. Download package archive
4. Verify package checksum
5. Registry network error handling
6. Cache registry responses
7. Multiple registry sources
8. Git registry support
9. Local registry for testing
10. Authenticated registry

**Caching tests:**
1. Cache downloaded package
2. Reuse cached package
3. Verify cached integrity
4. Cache size limits
5. LRU eviction policy
6. Offline mode using cache
7. Cache cleaning
8. Parallel downloads

**Lockfile tests:**
1. Use lockfile for reproducible build
2. Update lockfile on changes
3. Detect lockfile drift
4. Generate missing lockfile
5. Preserve lockfile structure
6. Lockfile integrity validation

**Conflict resolution tests:**
1. Simple version conflict
2. Transitive conflict
3. Diamond dependency
4. Dependency override
5. Conflict visualization
6. Suggested resolutions

**Build order tests:**
1. Topological sort dependencies
2. Parallel build opportunities
3. Diamond dependency handled once
4. Circular dependency error
5. Incremental build optimization

**Minimum test count:** 100 tests

## Integration Points
- Uses: Package manifest from phase-07
- Uses: Lockfile from phase-07
- Uses: Module system from phase-06
- Creates: Dependency resolver
- Creates: Registry interface
- Creates: Package cache
- Output: Automated dependency management

## Acceptance
- Resolve dependency trees correctly
- Find compatible versions satisfying constraints
- Download packages from registry
- Cache packages efficiently
- Use lockfile for reproducible builds
- Detect and report conflicts clearly
- Compute correct build order
- Handle network errors gracefully
- 100+ tests pass
- Offline mode works
- Parallel downloads functional
- Documentation comprehensive
- No clippy warnings
- cargo test passes
