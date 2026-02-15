# Phase 11b: Build System - Incremental Compilation & Cache

**Part 2 of 3: Build System Infrastructure**

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Phase-11a (Build System Core) must be complete.

**Verification Steps:**
1. Check STATUS.md: Foundation section
   - phase-11a (Build System Core & Targets) should be ✅

2. Verify phase-11a complete:
   ```bash
   ls crates/atlas-build/src/builder.rs
   ls crates/atlas-build/src/targets.rs
   ls crates/atlas-build/src/build_order.rs
   grep "test result" <(cargo test -p atlas-build 2>&1)
   ```

3. Verify core build functionality:
   ```bash
   # Should have 35+ tests passing
   cargo test -p atlas-build --lib 2>&1 | grep "test result"
   ```

**Expected from phase-11a:**
- Complete atlas-build crate infrastructure
- Builder with core orchestration
- Build targets (library, binary, bytecode, test)
- Build order computation
- Parallel compilation working
- 35+ tests passing

**Decision Tree:**

a) If phase-11a complete (STATUS.md ✅, 35+ tests pass):
   → Proceed with phase-11b
   → Add incremental compilation and cache system

b) If phase-11a incomplete (STATUS.md ⬜):
   → STOP immediately
   → Complete phase-11a first
   → Then return to phase-11b

c) If phase-11a marked complete but tests failing:
   → Phase-11a not actually complete
   → Fix phase-11a tests
   → Verify all tests pass
   → Then proceed with phase-11b

**No user questions needed:** Prerequisites are verifiable via STATUS.md, file existence, and cargo test.

---

## Objective

Implement incremental compilation and build cache management for Atlas build system. Enable fast rebuilds by tracking file changes, caching compiled modules, intelligently invalidating cache on changes, and recompiling only affected modules. This phase transforms the build system from correct-but-slow to production-ready fast.

**Scope:** This is part 2 of 3. Phase-11b adds incremental compilation and cache to the core build system from 11a. Build profiles, scripts, and CLI integration come in 11c.

## Files

**Extend:** `crates/atlas-build/src/builder.rs` (+200 lines - cache integration)
**Create:** `crates/atlas-build/src/cache.rs` (~300 lines - cache management)
**Create:** `crates/atlas-build/src/cache/metadata.rs` (~200 lines - cache metadata)
**Create:** `crates/atlas-build/src/cache/invalidation.rs` (~150 lines - invalidation logic)
**Extend:** `crates/atlas-build/src/lib.rs` (+30 lines - cache exports)
**Create:** `crates/atlas-build/tests/incremental_tests.rs` (~500 lines - incremental compilation tests)
**Create:** `crates/atlas-build/tests/cache_tests.rs` (~300 lines - cache management tests)
**Create:** `docs/features/build-system-incremental.md` (~300 lines - incremental build docs)

**Total this phase:** ~1980 lines (code + tests + docs)

## Dependencies

- Phase-11a: Core build system, builder, targets
- File change detection: timestamps, content hashing (sha2)
- Serialization: serde for cache persistence
- File system: std::fs for cache storage

## Implementation

### 1. Cache Infrastructure (`cache.rs`)

**Core cache structure:**
```rust
pub struct BuildCache {
    cache_dir: PathBuf,
    metadata: CacheMetadata,
    entries: HashMap<String, CacheEntry>,
}

pub struct CacheEntry {
    pub module_path: PathBuf,
    pub source_hash: String,        // SHA-256 of source content
    pub timestamp: SystemTime,       // Last modified time
    pub bytecode: Vec<u8>,          // Compiled bytecode
    pub dependencies: Vec<String>,   // Module dependencies
    pub compile_time: Duration,      // Compilation duration
}

pub struct CacheMetadata {
    pub version: String,             // Cache format version
    pub atlas_version: String,       // Atlas compiler version
    pub created: SystemTime,
    pub last_updated: SystemTime,
    pub total_entries: usize,
    pub total_size: u64,
}
```

**Cache operations:**

**Initialize cache:**
- Create cache directory: `target/cache/` by default
- Load existing cache metadata or create new
- Validate cache version compatibility
- Check Atlas version compatibility
- Load all cache entries into memory

**Store compiled module in cache:**
- Compute content hash (SHA-256) of source file
- Record file modification timestamp
- Store compiled bytecode
- Record module dependencies
- Track compilation time
- Persist cache entry to disk

**Retrieve cached module:**
- Look up module by path
- Validate source hash matches current file
- Validate timestamp matches current file
- Check dependency cache validity
- Return cached bytecode if valid
- Return None if invalid (cache miss)

**Cache persistence:**
- Serialize cache entries to JSON
- Store in `target/cache/metadata.json`
- Individual bytecode files: `target/cache/modules/{hash}.bc`
- Atomic writes to prevent corruption
- Load cache on build start
- Save cache on build completion

**Cache size management:**
- Track total cache size (bytes)
- Implement size limits (configurable, default 1GB)
- LRU eviction when size limit exceeded
- Clean stale entries (not accessed in 30 days)
- Provide cache statistics (hit rate, size, entry count)

**Cache invalidation:**
- Source file content changed (hash mismatch)
- Source file timestamp changed
- Dependency changed (propagate invalidation)
- Compiler version changed
- Build configuration changed
- Manual cache clear requested

### 2. Change Detection (`cache.rs` and `cache/metadata.rs`)

**Timestamp tracking:**
```rust
pub struct ChangeDetector {
    previous_state: HashMap<PathBuf, FileState>,
}

pub struct FileState {
    timestamp: SystemTime,
    size: u64,
    hash: String,  // Computed lazily only if timestamp changed
}

impl ChangeDetector {
    pub fn detect_changes(&mut self, files: &[PathBuf]) -> Vec<ChangedFile>;
}

pub struct ChangedFile {
    pub path: PathBuf,
    pub change_type: ChangeType,
}

pub enum ChangeType {
    Modified,
    Added,
    Removed,
    Moved { from: PathBuf },
}
```

**Change detection algorithm:**
1. **Quick check:** Compare file modification timestamps
2. **If timestamp changed:** Compute content hash (SHA-256)
3. **If hash changed:** File actually modified
4. **If hash same:** False positive (timestamp changed but content same)
5. **Track file additions:** New files not in previous state
6. **Track file deletions:** Previous files no longer exist
7. **Track file moves:** Detect by matching content hashes

**Content hashing:**
- Use SHA-256 for file content
- Hash entire source file
- Cache hash with timestamp
- Recompute only when timestamp changes
- Store hash in cache entry

**Handle edge cases:**
- Timestamps not reliable (some filesystems)
- Clock skew (build server time different)
- Rapid successive builds (timestamp granularity)
- **Solution:** Always hash when timestamp changed, compare hashes

### 3. Incremental Compilation (`builder.rs` extension)

**Extend Builder with incremental support:**
```rust
impl Builder {
    pub fn build_incremental(&mut self) -> BuildResult<BuildContext> {
        // Load build cache
        let mut cache = BuildCache::load(&self.config.cache_dir)?;

        // Detect file changes
        let changes = self.detect_source_changes(&cache)?;

        // Compute modules to recompile
        let to_recompile = self.compute_invalidation_set(&changes, &cache)?;

        // Compile only affected modules
        for module in &to_recompile {
            let bytecode = self.compile_module(module)?;
            cache.store(module, bytecode);
        }

        // Use cached bytecode for unchanged modules
        let cached_modules = self.load_cached_modules(&cache, &to_recompile)?;

        // Link all modules (cached + recompiled)
        let artifacts = self.link_modules(cached_modules, recompiled_modules)?;

        // Save updated cache
        cache.save()?;

        Ok(artifacts)
    }
}
```

**Incremental build algorithm:**
1. **Load cache:** Read existing build cache from disk
2. **Detect changes:** Compare current files to cached state
3. **Compute invalidation set:** Modules that need recompilation
   - Changed modules
   - Modules that depend on changed modules (transitive)
4. **Recompile affected:** Compile only invalidated modules
5. **Reuse cached:** Load bytecode for unchanged modules from cache
6. **Link:** Combine cached and recompiled modules into artifacts
7. **Update cache:** Save new cache state with recompiled modules

**Dependency invalidation propagation:**
```rust
fn compute_invalidation_set(
    &self,
    changes: &[ChangedFile],
    cache: &BuildCache,
) -> BuildResult<HashSet<String>> {
    let mut to_invalidate = HashSet::new();

    // Direct changes
    for changed in changes {
        to_invalidate.insert(changed.module_name.clone());
    }

    // Transitive dependencies
    loop {
        let mut added_any = false;
        for module in self.all_modules() {
            if to_invalidate.contains(&module.name) {
                continue;  // Already invalidated
            }
            // If any dependency invalidated, invalidate this module
            for dep in &module.dependencies {
                if to_invalidate.contains(dep) {
                    to_invalidate.insert(module.name.clone());
                    added_any = true;
                    break;
                }
            }
        }
        if !added_any {
            break;  // Fixed point reached
        }
    }

    Ok(to_invalidate)
}
```

**Performance optimization:**
- Parallel compilation of invalidated modules (from 11a)
- Load cached modules concurrently
- Minimize cache serialization overhead
- Efficient hash computation (cached)

### 4. Cache Invalidation Logic (`cache/invalidation.rs`)

**Invalidation triggers:**

**Source file changes:**
- Content hash changed
- File added or removed
- Imports modified (new dependencies)

**Dependency changes:**
- Direct dependency changed
- Transitive dependency changed
- Dependency version changed (package update)

**Configuration changes:**
- Compiler version upgraded
- Build configuration modified
- Optimization level changed (will be in 11c)
- Feature flags changed (future)

**Selective invalidation:**
- Only invalidate affected modules
- Preserve as much cache as possible
- Efficient transitive dependency tracking
- Minimize unnecessary recompilation

**Cache versioning:**
- Track cache format version
- Invalidate entire cache on format change
- Track Atlas compiler version
- Invalidate on compiler upgrade (safety)
- Allow override for development builds

### 5. Cache Statistics and Reporting

**Collect cache statistics:**
```rust
pub struct CacheStats {
    pub total_modules: usize,
    pub cached_modules: usize,
    pub recompiled_modules: usize,
    pub cache_hit_rate: f64,
    pub cache_size_bytes: u64,
    pub cache_entries: usize,
    pub time_saved: Duration,
}

impl BuildCache {
    pub fn stats(&self) -> CacheStats;

    pub fn report(&self) {
        // Print cache statistics
        // Example: "Cache hit rate: 85% (17/20 modules)"
        // Example: "Time saved: 4.2s (from cache)"
        // Example: "Cache size: 45.3 MB (120 entries)"
    }
}
```

**Report to user:**
- Cache hit rate (percentage)
- Modules recompiled vs cached
- Time saved by cache
- Cache size and entry count
- Cold build vs warm build comparison

### 6. Cache Cleanup and Maintenance

**Automatic cleanup:**
- Remove stale cache entries (> 30 days)
- Enforce cache size limits
- LRU eviction when limit exceeded
- Clean entries for deleted files
- Validate cache integrity

**Manual cache operations:**
```rust
impl BuildCache {
    pub fn clear(&mut self) -> BuildResult<()>;
    pub fn clean_stale(&mut self) -> BuildResult<usize>;
    pub fn rebuild_index(&mut self) -> BuildResult<()>;
}
```

**Cache validation:**
- Check for corrupted entries
- Verify bytecode integrity
- Rebuild index if corrupted
- Report and fix inconsistencies

## Tests (TDD - Use rstest)

**Incremental compilation tests (9 tests):**
1. `test_initial_build_compiles_all_files` - First build compiles everything
2. `test_rebuild_no_changes_uses_cache` - Rebuild with no changes fast
3. `test_change_one_file_recompiles_only_affected` - Single file change
4. `test_dependency_change_propagates` - Change propagates to dependents
5. `test_cache_invalidation_correctness` - Invalidation logic correct
6. `test_cache_persistence_across_builds` - Cache survives across builds
7. `test_cold_cache_build_from_scratch` - Cold build (no cache)
8. `test_warm_cache_build_fast_rebuild` - Warm build (full cache)
9. `test_partial_cache_mixed_recompile` - Some cached, some recompiled

**Cache management tests (7 tests):**
1. `test_cache_compiled_module` - Module cached correctly
2. `test_cache_hit_reuses_bytecode` - Cache hit returns cached bytecode
3. `test_cache_miss_recompiles` - Cache miss triggers recompilation
4. `test_cache_invalidation_on_change` - File change invalidates cache
5. `test_cache_size_limits_enforced` - Size limit causes eviction
6. `test_cache_cleanup_stale_entries` - Old entries cleaned up
7. `test_cache_statistics_accurate` - Cache stats correct

**Change detection tests (8 tests):**
1. `test_timestamp_tracking` - File timestamps tracked correctly
2. `test_content_hashing` - SHA-256 hashes computed correctly
3. `test_detect_source_file_change` - Modified file detected
4. `test_detect_dependency_change` - Dependency change detected
5. `test_ignore_non_source_changes` - Non-source files ignored
6. `test_handle_deleted_files` - Deleted files handled correctly
7. `test_handle_new_files` - New files detected and compiled
8. `test_handle_moved_files` - Moved files detected by hash

**Cache invalidation tests (6 tests):**
1. `test_invalidate_on_source_change` - Source change invalidates
2. `test_propagate_invalidation_to_dependents` - Transitive invalidation
3. `test_invalidate_on_dependency_change` - Dep change invalidates
4. `test_invalidate_on_compiler_version_change` - Compiler upgrade invalidates
5. `test_selective_invalidation` - Only affected modules invalidated
6. `test_cache_version_compatibility` - Incompatible cache rejected

**Performance tests (5 tests):**
1. `test_cold_build_timing` - Measure cold build time
2. `test_warm_build_timing` - Measure warm build time (should be fast)
3. `test_incremental_rebuild_timing` - Measure incremental rebuild time
4. `test_cache_hit_rate_tracking` - Cache hit rate computed correctly
5. `test_parallel_vs_sequential_with_cache` - Parallel cache loading

**Minimum test count for phase-11b:** 35 tests
**Target with edge cases:** 37-40 tests

## Integration Points

**Uses:**
- Builder from phase-11a (builder.rs)
- Build targets from phase-11a (targets.rs)
- Build order computation from phase-11a (build_order.rs)
- File system operations

**Creates:**
- Build cache system
- Incremental compilation
- Change detection
- Cache invalidation logic

**Prepares for:**
- Phase-11c: Build profiles affect cache key, scripts run on cache miss

**Output:**
- Fast incremental builds
- Intelligent cache management
- Production-ready build performance

## Acceptance Criteria

- ✅ Build cache infrastructure working
- ✅ Incremental compilation recompiles only affected modules
- ✅ File change detection accurate (timestamps + hashing)
- ✅ Cache invalidation propagates correctly to dependents
- ✅ Cache persists across builds
- ✅ Cold build vs warm build significant speedup
- ✅ Cache hit rate tracked and reported
- ✅ Cache size limits enforced with LRU eviction
- ✅ Stale cache entries cleaned up
- ✅ Cache statistics accurate and informative
- ✅ Corrupted cache handled gracefully
- ✅ Parallel cache loading works
- ✅ Cache versioning prevents incompatibilities
- ✅ 35+ tests pass (target: 40)
- ✅ No clippy warnings
- ✅ cargo test -p atlas-build passes
- ✅ Documentation complete and accurate
- ✅ Integration with phase-11a working correctly

## Notes

**World-class compiler standards:**
- Incremental compilation as fast as Rust/Go builds
- Robust cache invalidation (correctness over speed)
- Efficient change detection
- Clear cache statistics
- Reliable cache persistence

**Incremental build philosophy:**
- Correctness first: never use stale cache
- Cache invalidation is hard: be conservative
- Measure performance: cold vs warm builds
- User visibility: show what's cached vs recompiled

**Performance targets:**
- Warm build (no changes): < 100ms
- Incremental build (1 file changed): < 500ms
- Cold build: baseline (full compilation)
- Cache hit rate: > 80% in typical development

**Phase boundaries:**
- 11a: Core orchestration + targets ✅
- 11b: Incremental compilation + cache (this phase)
- 11c: Profiles + scripts + CLI + docs (next)

**This phase makes the build system production-ready fast.**
