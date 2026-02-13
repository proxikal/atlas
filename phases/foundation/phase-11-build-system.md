# Phase 11: Build System Infrastructure

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Package manifest, module system, and compiler must exist.

**Verification:**
```bash
ls crates/atlas-package/src/manifest.rs
ls crates/atlas-runtime/src/modules/mod.rs
ls crates/atlas-runtime/src/compiler/mod.rs
cargo test --package atlas-package
```

**What's needed:**
- Package manifest from foundation/phase-07
- Module system from foundation/phase-06
- Compiler from v0.1
- Dependency resolver from foundation/phase-08

**If missing:** Complete foundation phases 06-08 first

---

## Objective
Implement comprehensive build system orchestrating compilation of multi-file projects, managing dependencies, producing build artifacts, and supporting different build configurations - enabling professional project builds and releases.

## Files
**Create:** `crates/atlas-build/` (new crate ~1500 lines total)
**Create:** `crates/atlas-build/src/lib.rs` (~200 lines)
**Create:** `crates/atlas-build/src/builder.rs` (~600 lines)
**Create:** `crates/atlas-build/src/targets.rs` (~400 lines)
**Create:** `crates/atlas-build/src/cache.rs` (~300 lines)
**Update:** `crates/atlas-cli/src/commands/build.rs` (~300 lines)
**Create:** `docs/build-system.md` (~800 lines)
**Tests:** `crates/atlas-build/tests/builder_tests.rs` (~600 lines)
**Tests:** `crates/atlas-build/tests/incremental_tests.rs` (~500 lines)

## Dependencies
- Package manifest system
- Module system and dependency graph
- Compiler and bytecode emitter
- Dependency resolver
- File system operations

## Implementation

### Build Pipeline Orchestration
Create Builder coordinating entire build process. Load package manifest reading project configuration. Resolve dependencies downloading required packages. Build dependency graph from module imports. Compute topological build order. Compile modules in dependency order. Link compiled modules into final artifacts. Run build scripts from manifest. Apply build configuration optimization levels. Track build progress reporting to user. Parallel compilation of independent modules. Incremental compilation rebuilding only changes.

### Build Targets and Artifacts
Support multiple build targets and artifact types. Library target produces reusable package. Binary target produces executable. Bytecode target produces .atl.bc bytecode files. Native target produces platform binaries (future). WebAssembly target (future). Test target builds test suite. Benchmark target builds benchmarks. Documentation target generates docs. Custom targets from build scripts. Output artifacts to target directory with conventional structure.

### Incremental Compilation
Implement incremental builds for fast iteration. Track file modification times. Hash file contents for change detection. Cache compiled module bytecode. Invalidate cache on source changes. Propagate invalidation to dependents. Recompile only affected modules. Link cached and new modules. Persist cache across builds. Cache metadata for debugging. Configurable cache directory. Cache size limits with cleanup.

### Build Configuration Profiles
Support multiple build profiles with different settings. Development profile optimized for build speed. Release profile optimized for runtime performance. Test profile with debug information. Custom profiles in manifest. Profile-specific compiler flags. Profile-specific dependencies. Environment variable configuration. Override profiles via CLI flags. Default to dev profile.

### Build Scripts and Custom Steps
Execute custom build scripts from manifest. Scripts in Atlas or shell commands. Pre-build and post-build hooks. Access to build context from scripts. Environment variables for scripts. Script timeout limits. Capture script output. Report script errors clearly. Security considerations for script execution. Sandboxing build scripts.

### Dependency Building
Build dependencies before dependents. Download dependencies using package manager. Build dependencies from source if needed. Use pre-built binaries when available. Cache built dependencies. Link against dependency artifacts. Propagate build flags to dependencies. Handle diamond dependencies efficiently. Workspace dependency linking.

### Build Cache Management
Implement persistent build cache for speed. Cache compiled modules by content hash. Store intermediate build artifacts. Track cache metadata timestamps, hashes. Invalidate cache intelligently. Share cache across projects when safe. Clean stale cache entries. Configurable cache strategies. Cache statistics reporting. Cold cache vs warm cache performance.

### Build Output and Reporting
Provide clear build progress feedback. Show compilation progress per-file. Estimate remaining build time. Colorized output for errors and warnings. Summary statistics compilation time, file count. Verbose mode for debugging. Quiet mode for CI. JSON output for tooling. Build logs to file. Error message formatting.

## Tests (TDD - Use rstest)

**Build pipeline tests:**
1. Build simple single-file project
2. Build multi-file project with imports
3. Build with dependencies
4. Build library target
5. Build binary target
6. Build test target
7. Parallel module compilation
8. Build order respects dependencies
9. Build script execution
10. Build configuration profiles

**Incremental compilation tests:**
1. Initial build compiles all files
2. Rebuild no changes fast
3. Change one file only recompiles affected
4. Dependency change propagates
5. Cache invalidation correctness
6. Cache persistence across builds
7. Cold cache build
8. Warm cache build
9. Cache cleanup

**Build targets tests:**
1. Library artifact structure
2. Binary artifact structure
3. Bytecode artifact format
4. Test target output
5. Multiple targets in one build
6. Target-specific configuration

**Build scripts tests:**
1. Execute pre-build script
2. Execute post-build script
3. Script access to build context
4. Script failure aborts build
5. Script timeout enforcement
6. Script output capture
7. Sandboxing build scripts

**Dependency building tests:**
1. Build local path dependency
2. Download and build registry dependency
3. Use cached dependency
4. Rebuild dependency on changes
5. Diamond dependency handling
6. Workspace dependency linking

**Configuration tests:**
1. Dev profile settings
2. Release profile optimization
3. Custom profile from manifest
4. CLI override profile
5. Profile-specific dependencies

**Cache management tests:**
1. Cache compiled module
2. Cache hit reuses bytecode
3. Cache miss recompiles
4. Cache invalidation on change
5. Cache size limits
6. Cache cleanup
7. Cache statistics

**Build output tests:**
1. Progress reporting
2. Error formatting
3. Warning formatting
4. Build summary
5. Verbose mode output
6. Quiet mode output
7. JSON output format

**Minimum test count:** 100 tests

## Integration Points
- Uses: Package manifest from phase-07
- Uses: Module system from phase-06
- Uses: Dependency resolver from phase-08
- Uses: Compiler from v0.1
- Creates: atlas-build crate
- Creates: Build orchestration system
- Output: Professional build infrastructure

## Acceptance
- Build multi-file projects successfully
- Incremental compilation works correctly
- Build cache speeds up rebuilds significantly
- Multiple build targets supported
- Build profiles configure compilation
- Build scripts execute correctly
- Dependencies built automatically
- Parallel compilation functional
- Build output clear and informative
- 100+ tests pass
- Build time acceptable for large projects
- Documentation comprehensive
- No clippy warnings
- cargo test passes
