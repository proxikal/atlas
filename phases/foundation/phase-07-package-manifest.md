# Phase 07: Package Manifest System

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Configuration system (phase-04) and Module system (phase-06) must be complete.

**Verification Steps:**
1. Check STATUS.md: Foundation section
   - phase-04 (Configuration) should be ✅
   - phase-06 (Module System) should be ✅

2. Verify phase-04 (Configuration) complete:
   ```bash
   ls crates/atlas-config/src/lib.rs
   ls crates/atlas-config/src/config.rs
   cargo test -p atlas-config 2>&1 | grep "test result"
   grep -n "pub struct AtlasConfig" crates/atlas-config/src/config.rs
   ```

3. Verify phase-06 (Module System) complete:
   ```bash
   ls crates/atlas-runtime/src/modules/mod.rs
   ls crates/atlas-runtime/src/modules/resolver.rs
   cargo test modules 2>&1 | grep "test result"
   ```

**Expected from phase-04 (per acceptance criteria):**
- atlas-config crate with AtlasConfig struct
- TOML parsing using serde + toml crates
- Configuration loading from atlas.toml files
- 60+ tests passing

**Expected from phase-06 (per acceptance criteria):**
- Module system with import/export
- Module resolver finding modules by path
- Dependency graph construction
- 120+ tests passing (60 unit, 60 integration)

**Decision Tree:**

a) If both phase-04 AND phase-06 complete (STATUS.md ✅, tests pass):
   → Proceed with phase-07
   → Use existing TOML parsing from phase-04
   → Use module resolution from phase-06

b) If phase-04 complete but phase-06 incomplete:
   → STOP immediately
   → Report: "Foundation phase-06 required before phase-07"
   → Complete phase-06 first
   → Then return to phase-07

c) If phase-06 complete but phase-04 incomplete:
   → STOP immediately
   → Report: "Foundation phase-04 required before phase-07"
   → Complete phase-04 first
   → Then return to phase-07

d) If both incomplete:
   → ERROR: Wrong phase ordering
   → Check STATUS.md next phase
   → Should be phase-04 or phase-06, not phase-07
   → Fix phase ordering

**No user questions needed:** Phase completion is verifiable via STATUS.md, file existence, and cargo test.

---

## Objective
Implement package manifest system using atlas.toml for project metadata, dependencies, build configuration, and package publishing - enabling dependency management and package ecosystem foundation.

## Files
**Create:** `crates/atlas-package/` (new crate ~1200 lines total)
**Create:** `crates/atlas-package/src/lib.rs` (~150 lines)
**Create:** `crates/atlas-package/src/manifest.rs` (~500 lines)
**Create:** `crates/atlas-package/src/validator.rs` (~300 lines)
**Create:** `crates/atlas-package/src/lockfile.rs` (~250 lines)
**Update:** `Cargo.toml` (add atlas-package to workspace)
**Create:** `docs/package-manifest.md` (~800 lines)
**Create:** `atlas.toml.package-example` (~200 lines)
**Tests:** `crates/atlas-package/tests/manifest_tests.rs` (~600 lines)
**Tests:** `crates/atlas-package/tests/lockfile_tests.rs` (~400 lines)

## Dependencies
- Configuration system for TOML parsing
- Module system for entry point validation
- serde and toml crates
- semver crate for version parsing

## Implementation

### Manifest Schema Design
Define comprehensive atlas.toml schema for packages. Package section with name, version, description, authors, license, repository, homepage fields. Dependencies section with package names and version constraints. Dev-dependencies for test and development dependencies. Build section with compiler options, target settings, optimization level. Scripts section for custom build commands. Workspace section for multi-package projects. Lib section defining library entry point. Bin section defining binary entry points. Features section for conditional compilation. Use semver for version constraints supporting exact, range, caret, tilde syntax.

### Manifest Parsing and Validation
Create PackageManifest struct deserializing from TOML. Validate package name follows naming rules lowercase alphanumeric with hyphens. Validate version follows semantic versioning. Validate license uses SPDX identifiers. Check entry points exist as files. Validate dependency version constraints well-formed. Detect circular dependencies in workspace. Report validation errors with line numbers from TOML. Provide helpful error messages for common mistakes. Support defaults for optional fields.

### Dependency Specification
Support flexible dependency specifications. Simple version constraint using semver syntax. Git dependencies with repository URL, branch, tag, or commit. Path dependencies for local packages. Registry dependencies from future package registry. Optional dependencies enabled by features. Platform-specific dependencies using cfg conditions. Rename dependencies for conflict resolution. Override dependencies for patching.

### Lockfile Generation
Create atlas.lock file pinning exact dependency versions. Record resolved versions for reproducible builds. Include transitive dependencies in lockfile. Store checksums for integrity verification. Track dependency source registry, git, or path. Update lockfile on dependency changes. Support manual lockfile editing. Lockfile uses TOML format for human readability.

### Workspace Support
Enable multi-package workspaces sharing dependencies. Workspace root defines member packages. Shared dependency versions across workspace. Virtual workspace without root package. Member packages reference workspace dependencies. Workspace-level scripts and configuration. Dependency deduplication in workspace. Build all workspace packages together.

### Feature Flags System
Implement conditional compilation with features. Define features in manifest with dependency lists. Enable features additively combining dependencies. Default features active by default. Optional dependencies become features. Feature combinations validated. Cascade feature activation through dependencies. Report feature conflicts clearly.

### Manifest API
Provide programmatic manifest access for tooling. Load manifest from file or string. Query package metadata name, version, authors. Access dependency lists. Resolve entry points. Validate manifest without building. Serialize manifest to TOML. Merge manifests for workspaces.

## Tests (TDD - Use rstest)

**Manifest parsing tests:**
1. Parse valid minimal manifest
2. Parse complete manifest all sections
3. Invalid TOML syntax error
4. Missing required field error
5. Invalid package name error
6. Invalid version error
7. Invalid license identifier warning
8. Unknown field warnings
9. Default values applied
10. Entry point file not found error

**Dependency specification tests:**
1. Simple semver dependency
2. Exact version constraint
3. Version range constraint
4. Git dependency with branch
5. Git dependency with tag
6. Path dependency
7. Optional dependency
8. Platform-specific dependency
9. Dependency rename
10. Invalid version constraint error

**Lockfile tests:**
1. Generate lockfile from manifest
2. Lockfile pins exact versions
3. Lockfile includes transitive dependencies
4. Lockfile checksums verified
5. Update lockfile on dependency change
6. Lockfile prevents version drift
7. Manual lockfile edit preserved
8. Corrupted lockfile detected

**Workspace tests:**
1. Workspace with multiple members
2. Shared workspace dependencies
3. Member dependency on another member
4. Virtual workspace
5. Workspace dependency deduplication
6. Circular workspace dependency error

**Feature tests:**
1. Feature enables optional dependency
2. Default features active
3. Disable default features
4. Feature combination
5. Feature cascade through dependencies
6. Conflicting features error

**Validation tests:**
1. Validate package name rules
2. Validate semver version
3. Validate entry points exist
4. Validate dependency versions
5. Detect circular dependencies
6. License identifier validation

**Minimum test count:** 80 tests

## Integration Points
- Uses: Configuration system from foundation/phase-04
- Uses: Module system from foundation/phase-06
- Creates: atlas-package crate
- Creates: Package manifest format
- Creates: Lockfile format
- Output: Dependency management foundation

## Acceptance
- Parse valid atlas.toml manifests
- Validate all manifest fields
- Generate atlas.lock lockfiles
- Pin dependency versions reproducibly
- Support workspace projects
- Feature flags work
- Entry point validation
- Helpful error messages
- 80+ tests pass
- Documentation complete with examples
- Example manifests provided
- No clippy warnings
- cargo test passes
