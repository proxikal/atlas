# Phase 05: Incremental Compilation

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Build system and module system must exist.

**Verification:**
```bash
ls crates/atlas-build/src/builder.rs
ls crates/atlas-runtime/src/modules/mod.rs
cargo test --package atlas-build
```

**What's needed:**
- Build system from foundation/phase-11
- Module system from foundation/phase-06
- Dependency graph
- File change detection

**If missing:** Complete foundation phases 06 and 11 first

---

## Objective
Implement incremental compilation rebuilding only changed modules and dependents - dramatically improving build times for iterative development enabling fast feedback loops for large projects.

## Files
**Update:** `crates/atlas-build/src/builder.rs` (~400 lines)
**Create:** `crates/atlas-build/src/incremental.rs` (~700 lines)
**Create:** `crates/atlas-build/src/fingerprint.rs` (~400 lines)
**Update:** `crates/atlas-build/src/cache.rs` (~300 lines)
**Create:** `docs/incremental-compilation.md` (~600 lines)
**Tests:** `crates/atlas-build/tests/incremental_tests.rs` (~700 lines)

## Dependencies
- Build system infrastructure
- Module dependency graph
- File hashing for change detection
- Build cache management

## Implementation

### Change Detection
Hash source file contents for change detection. Fingerprint includes file content, dependencies, compiler version. Compare fingerprints to detect changes. Modification time as quick check. Content hash for accuracy. Track dependency fingerprints. Cascade invalidation on dep changes. Ignore comment-only changes option.

### Dependency Tracking
Build dependency graph between modules. Track file-level dependencies from imports. Compiler flags affect dependencies. Configuration changes invalidate builds. Environment variables as dependencies. OS and architecture in fingerprint. Transitive dependency tracking.

### Selective Recompilation
Recompile only changed modules. Propagate changes to direct dependents. Topological order for recompilation. Skip unchanged subtrees. Parallel recompile independent modules. Incremental linking updated modules. Optimize for minimal recompilation.

### Build Artifact Caching
Cache compiled bytecode per module. Store artifacts with fingerprints. Lookup cache by fingerprint. Reuse cached artifacts when valid. Garbage collect old cache entries. Cache size limits. Persistent cache across builds. Shared cache option for CI.

### Incremental Type Checking
Type check only changed modules. Reuse type information from cache. Propagate type changes to dependents. Invalidate on signature changes. Interface-based dependency tracking. Incremental constraint solving. Optimize type checking phase.

### Build State Persistence
Persist build state between invocations. Save dependency graph to disk. Save fingerprints database. Quick startup using saved state. Detect state corruption. Fallback to full rebuild. State versioning for compatibility.

## Tests (TDD - Use rstest)
1. Initial full build
2. No changes rebuild fast
3. Change one file rebuilds minimal
4. Dependency change propagates
5. Fingerprint detects changes
6. Cache hit reuses artifact
7. Cache miss recompiles
8. Parallel incremental build
9. Type checking incremental
10. State persistence works

**Minimum test count:** 50 tests

## Acceptance
- Incremental builds significantly faster
- Only changed modules recompile
- Dependency changes propagate
- Cache reuses artifacts
- Type checking incremental
- State persists across builds
- 50+ tests pass
- Speedup benchmarks documented
- cargo test passes
