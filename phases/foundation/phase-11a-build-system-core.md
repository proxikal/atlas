# Phase 11a: Build System - Core & Targets

**Part 1 of 3: Build System Infrastructure**

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Module system (phase-06), Package manifest (phase-07), and Package manager (phase-08) must be complete.

**Verification Steps:**
1. Check STATUS.md: Foundation section
   - phase-06 (Module System) should be ✅
   - phase-07 (Package Manifest) should be ✅
   - phase-08 (Package Manager - all 3 parts) should be ✅

2. Verify phase-06 complete:
   ```bash
   ls crates/atlas-runtime/src/module_loader.rs
   ls crates/atlas-runtime/src/module_executor.rs
   ls crates/atlas-runtime/src/resolver/mod.rs
   grep "test result" <(cargo test -p atlas-runtime module_loader 2>&1)
   ```

3. Verify phase-07 complete:
   ```bash
   ls crates/atlas-package/src/manifest.rs
   grep "test result" <(cargo test -p atlas-package manifest 2>&1)
   ```

4. Verify phase-08 complete (all 3 parts):
   ```bash
   ls crates/atlas-package/src/resolver.rs
   ls crates/atlas-package/src/registry.rs
   ls crates/atlas-package/src/resolver/build_order.rs
   grep "test result" <(cargo test -p atlas-package 2>&1)
   ```

5. Verify compiler from v0.1 exists:
   ```bash
   ls crates/atlas-runtime/src/compiler/mod.rs
   grep "test result" <(cargo test -p atlas-runtime compiler 2>&1)
   ```

**Expected from phases 06-08:**
- Phase-06: Module resolution, dependency graph, module loader/executor
- Phase-07: PackageManifest, lockfile validation, 66+ tests
- Phase-08a: PubGrub resolver, dependency solver, 52+ tests
- Phase-08b: Registry interface, downloader, cache, 31+ tests
- Phase-08c: Conflict resolution, build order computation, lockfile integration, 32+ tests
- v0.1: Complete compiler pipeline (lexer → parser → compiler → bytecode)

**Decision Tree:**

a) If all phases complete (STATUS.md ✅, all tests pass):
   → Proceed with phase-11a
   → Build system core orchestrates: resolver → modules → compiler

b) If any phase incomplete (STATUS.md ⬜):
   → STOP immediately
   → Report which phases are incomplete
   → Complete missing phases in order: 06 → 07 → 08a → 08b → 08c
   → Then return to phase-11a

c) If phases marked complete but tests failing:
   → One or more phases not actually complete
   → Fix failing phases first
   → Verify all tests pass
   → Then proceed with phase-11a

d) If v0.1 compiler missing or broken:
   → CRITICAL ERROR: v0.1 should be complete
   → Verify v0.1 completion in STATUS.md
   → Fix v0.1 compiler issues
   → Then proceed with phase-11a

**No user questions needed:** Prerequisites are verifiable via STATUS.md, file existence, and cargo test.

---

## Objective

Implement core build system infrastructure for Atlas projects: build pipeline orchestration, multiple build targets (library, binary, bytecode, test), dependency ordering with topological sort, and parallel compilation of independent modules. This phase establishes the foundation for professional project builds.

**Scope:** This is part 1 of 3. Phase-11a focuses on core build orchestration and targets. Incremental compilation/cache (11b) and build configuration/scripts (11c) follow.

## Files

**Create:** `crates/atlas-build/` (new crate - this phase starts it)
**Create:** `crates/atlas-build/Cargo.toml` (~40 lines - crate definition)
**Create:** `crates/atlas-build/src/lib.rs` (~200 lines - public API, re-exports)
**Create:** `crates/atlas-build/src/builder.rs` (~400 lines - core build orchestration)
**Create:** `crates/atlas-build/src/targets.rs` (~400 lines - build target types)
**Create:** `crates/atlas-build/src/build_order.rs` (~200 lines - topological sort, parallelization)
**Create:** `crates/atlas-build/src/error.rs` (~150 lines - build errors)
**Create:** `crates/atlas-build/tests/builder_tests.rs` (~400 lines - core build tests)
**Create:** `crates/atlas-build/tests/target_tests.rs` (~300 lines - target-specific tests)
**Create:** `docs/features/build-system-core.md` (~400 lines - core build documentation)

**Note:** Phase-11b will add `cache.rs` (~300 lines). Phase-11c will add profile/script support to builder.rs (~200 lines), CLI integration (~300 lines), and complete documentation (~800 lines total).

**Total this phase:** ~2490 lines (code + tests + docs)

## Dependencies

- Package manifest system (phase-07) - read atlas.toml configuration
- Module system and dependency graph (phase-06) - resolve module imports
- Compiler and bytecode emitter (v0.1) - compile Atlas source to bytecode
- Dependency resolver (phase-08) - resolve package dependencies
- Build order computation (phase-08c) - topological sort algorithm
- File system operations - read source files, write artifacts

## Implementation

### 1. Build Crate Infrastructure

**Create:** `crates/atlas-build/Cargo.toml`
```toml
[package]
name = "atlas-build"
version = "0.2.0"
edition = "2021"

[dependencies]
atlas-runtime = { path = "../atlas-runtime" }
atlas-package = { path = "../atlas-package" }
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
rayon = "1.10"  # Parallel compilation
walkdir = "2.5"  # Directory traversal
sha2 = "0.10"  # Content hashing (used in 11b, added now)

[dev-dependencies]
tempfile = "3.14"
rstest = "0.23"
pretty_assertions = "1.4"
```

**Create:** `crates/atlas-build/src/lib.rs`
- Public API re-exports
- Core types: `BuildResult`, `BuildConfig`, `BuildContext`
- Error types
- Builder interface

**Structure:**
```rust
// Public API
pub use builder::{Builder, BuildContext};
pub use targets::{BuildTarget, TargetKind, BuildArtifact};
pub use error::{BuildError, BuildResult};

mod builder;
mod targets;
mod build_order;
mod error;

// Re-export for convenience
pub use atlas_package::manifest::PackageManifest;
```

### 2. Build Pipeline Orchestration (`builder.rs`)

**Core struct:**
```rust
pub struct Builder {
    root_dir: PathBuf,
    manifest: PackageManifest,
    config: BuildConfig,
}

pub struct BuildConfig {
    pub target_dir: PathBuf,
    pub optimization_level: OptLevel,  // Basic for now, profiles in 11c
    pub parallel: bool,
    pub verbose: bool,
}

pub struct BuildContext {
    pub manifest: PackageManifest,
    pub modules: Vec<CompiledModule>,
    pub artifacts: Vec<BuildArtifact>,
    pub stats: BuildStats,
}
```

**Implementation:**

**Load package manifest:**
- Read `atlas.toml` from project root
- Parse using `PackageManifest::from_path()` (phase-07)
- Validate manifest completeness
- Extract build configuration

**Resolve dependencies:**
- Use dependency resolver from phase-08
- Download required packages to cache
- Build dependency graph
- Handle workspace dependencies (local path deps)

**Build dependency graph from modules:**
- Scan source files in `src/` directory
- Parse import statements from each file
- Build module dependency graph
- Detect circular dependencies (error)
- Integrate package dependencies into graph

**Compute topological build order:**
- Use `build_order::compute_build_order()` (see below)
- Topological sort of module graph
- Identify independent modules (parallel groups)
- Return ordered compilation sequence

**Compile modules in dependency order:**
- For each module in build order:
  - Load source file
  - Compile to bytecode using `atlas_runtime::Compiler`
  - Collect compile errors
  - Track compilation progress
- Handle compilation errors gracefully
- Abort build on first error (fail-fast)

**Parallel compilation of independent modules:**
- Use `rayon` for parallel execution
- Identify modules with no dependencies between them
- Compile parallel groups concurrently
- Maintain build order for dependent modules
- Thread-safe error collection

**Link compiled modules into artifacts:**
- Combine compiled bytecode from all modules
- Generate artifact based on target type (see targets.rs)
- Write artifacts to target directory
- Generate metadata files

**Track build progress:**
- Count total modules to compile
- Track modules compiled so far
- Compute percentage complete
- Report progress to user (stdout)
- Track build timing statistics

### 3. Build Targets and Artifacts (`targets.rs`)

**Support multiple build targets:**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TargetKind {
    Library,      // Reusable package
    Binary,       // Executable program
    Bytecode,     // .atl.bc bytecode files
    Test,         // Test suite
    Benchmark,    // Benchmark suite (minimal for now)
}

pub struct BuildTarget {
    pub name: String,
    pub kind: TargetKind,
    pub entry_point: Option<PathBuf>,  // For binary/test
    pub sources: Vec<PathBuf>,
    pub dependencies: Vec<String>,
}

pub struct BuildArtifact {
    pub target: BuildTarget,
    pub output_path: PathBuf,
    pub bytecode: Vec<u8>,
    pub metadata: ArtifactMetadata,
}

pub struct ArtifactMetadata {
    pub compile_time: Duration,
    pub module_count: usize,
    pub bytecode_size: usize,
    pub atlas_version: String,
}
```

**Library target:**
- Produces reusable package
- Output: `target/debug/libname.atl.bc` (bytecode library)
- Exports public API from lib.atlas
- Includes all public modules
- Metadata: exported symbols, dependencies

**Binary target:**
- Produces executable program
- Output: `target/debug/binary_name` (executable wrapper)
- Requires `main()` function entry point
- Links against library dependencies
- Includes runtime bootstrap code

**Bytecode target:**
- Produces standalone `.atl.bc` bytecode files
- One file per module or combined
- Can be loaded by runtime directly
- Platform-independent format
- No runtime linking required

**Test target:**
- Builds test suite from `tests/` directory
- Output: `target/debug/test_binary`
- Includes test framework integration (basic for now)
- Links against library under test
- Collects all test functions

**Benchmark target:**
- Builds benchmarks from `benches/` directory (minimal support)
- Output: `target/debug/bench_binary`
- Similar to test target structure
- Prepared for phase-13 (Performance Benchmarking)

**Output artifacts to target directory:**
- Conventional structure: `target/{debug,release}/`
- Separate directories per target kind
- Preserve source directory structure
- Generate build metadata files
- Clean output on request

### 4. Build Order Computation (`build_order.rs`)

**Topological sort implementation:**
```rust
pub struct BuildGraph {
    modules: HashMap<String, ModuleNode>,
}

pub struct ModuleNode {
    name: String,
    path: PathBuf,
    dependencies: Vec<String>,
}

pub fn compute_build_order(graph: &BuildGraph) -> BuildResult<Vec<Vec<String>>> {
    // Returns Vec<Vec<String>> where each inner Vec is a parallel group
    // Uses Kahn's algorithm (from phase-08c)
    // Detect cycles and report circular dependencies
}
```

**Identify parallel compilation groups:**
- Modules with no dependencies between them can compile in parallel
- Group modules by dependency level
- Each group can be compiled concurrently
- Enforce ordering between groups

**Handle diamond dependencies:**
- Module A depends on B and C
- Both B and C depend on D
- D compiled first, then B and C in parallel, then A
- Efficient reuse of shared dependencies

**Detect circular dependencies:**
- Use cycle detection in topological sort
- Report full cycle path to user
- Clear error message with fix suggestions
- Abort build on circular dependency

### 5. Build Statistics and Progress (`builder.rs` extension)

```rust
pub struct BuildStats {
    pub total_modules: usize,
    pub compiled_modules: usize,
    pub parallel_groups: usize,
    pub total_time: Duration,
    pub compilation_time: Duration,
    pub linking_time: Duration,
}
```

**Track and report:**
- Total compilation time
- Per-module compile times
- Parallel efficiency (speedup factor)
- Artifact sizes
- Module count statistics

### 6. Error Handling (`error.rs`)

**Comprehensive build errors:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Failed to read manifest: {0}")]
    ManifestError(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Compilation failed for module {module}: {error}")]
    CompilationError { module: String, error: String },

    #[error("Missing entry point for binary target: {0}")]
    MissingEntryPoint(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type BuildResult<T> = Result<T, BuildError>;
```

**Error reporting:**
- Clear error messages
- Source location for compilation errors
- Actionable fix suggestions
- Error codes for categorization (future)

## Tests (TDD - Use rstest)

**Build pipeline tests (7 tests):**
1. `test_build_simple_single_file_project` - Single file compiles successfully
2. `test_build_multi_file_project_with_imports` - Multiple files with imports
3. `test_build_project_with_dependencies` - External package dependencies
4. `test_build_library_target` - Library target produces correct artifact
5. `test_build_binary_target` - Binary target with main() entry point
6. `test_build_test_target` - Test target from tests/ directory
7. `test_parallel_module_compilation` - Independent modules compile in parallel

**Build order tests (5 tests):**
1. `test_simple_build_order` - Linear dependency chain A→B→C
2. `test_complex_dependency_graph` - Diamond dependencies handled correctly
3. `test_circular_dependency_detection` - Circular deps caught with clear error
4. `test_parallel_compilation_groups` - Independent modules grouped for parallel build
5. `test_workspace_build_order` - Local workspace dependencies ordered correctly

**Build targets tests (6 tests):**
1. `test_library_artifact_structure` - Library artifact has correct format
2. `test_binary_artifact_structure` - Binary artifact with entry point
3. `test_bytecode_artifact_format` - Standalone bytecode file format
4. `test_test_target_output` - Test target includes all tests
5. `test_multiple_targets_in_one_build` - Build multiple targets simultaneously
6. `test_target_specific_configuration` - Each target uses correct config

**Dependency building tests (3 tests):**
1. `test_build_local_path_dependency` - Local workspace dependency builds first
2. `test_build_registry_dependency_basic` - Registry dependency downloaded and built
3. `test_use_cached_dependency` - Cached dependency reused, not rebuilt

**Compilation tests (8 tests):**
1. `test_compile_single_module` - Single module compilation to bytecode
2. `test_compile_with_imports` - Module with imports compiles correctly
3. `test_link_compiled_modules` - Multiple modules linked into artifact
4. `test_compilation_error_handling` - Compilation errors reported clearly
5. `test_workspace_build` - Multi-package workspace builds correctly
6. `test_clean_build_output` - Target directory structure correct
7. `test_artifact_metadata` - Artifacts include correct metadata
8. `test_output_artifact_structure` - Output directory follows conventions

**Integration tests (5 tests):**
1. `test_full_build_pipeline_single_file` - End-to-end single file build
2. `test_full_build_pipeline_multi_file` - End-to-end multi-file project build
3. `test_build_error_propagation` - Errors propagate through pipeline correctly
4. `test_build_progress_tracking` - Progress stats accurate throughout build
5. `test_output_directory_structure` - target/ directory structure correct

**Minimum test count for phase-11a:** 34 tests
**Target with edge cases:** 35-40 tests

## Integration Points

**Uses:**
- PackageManifest from phase-07 (manifest.rs)
- Module system from phase-06 (module_loader.rs, module_executor.rs)
- Dependency resolver from phase-08 (resolver.rs, build_order.rs)
- Compiler from v0.1 (compiler/mod.rs)

**Creates:**
- atlas-build crate (new)
- Build orchestration system
- Build target infrastructure
- Parallel compilation foundation

**Prepares for:**
- Phase-11b: Incremental compilation, cache management
- Phase-11c: Build profiles, scripts, CLI integration

**Output:**
- Core build system infrastructure
- Multiple build targets working
- Parallel compilation functional
- Foundation for incremental builds (11b)

## Acceptance Criteria

- ✅ atlas-build crate created with correct structure
- ✅ Build multi-file projects successfully
- ✅ Library target produces correct artifacts
- ✅ Binary target produces executables
- ✅ Bytecode target produces .atl.bc files
- ✅ Test target builds test suites
- ✅ Topological build order computed correctly
- ✅ Circular dependencies detected with clear errors
- ✅ Parallel compilation of independent modules works
- ✅ Dependencies built before dependents
- ✅ Local workspace dependencies handled
- ✅ Registry dependencies downloaded and built
- ✅ Build progress tracked and accurate
- ✅ Build statistics collected correctly
- ✅ Error handling comprehensive and clear
- ✅ 35+ tests pass (target: 40)
- ✅ No clippy warnings
- ✅ cargo test -p atlas-build passes
- ✅ Documentation complete and accurate
- ✅ Integration with phase-06/07/08 working correctly

## Notes

**World-class compiler standards:**
- Professional build infrastructure matching Rust/Go quality
- Robust error handling with actionable messages
- Efficient parallel compilation
- Clean, maintainable architecture
- Comprehensive test coverage

**Build system philosophy:**
- Correctness over speed (optimize in 11b)
- Clear error messages over terse output
- Conventional structure over flexibility
- Fail fast on errors

**Phase boundaries:**
- 11a: Core orchestration + targets (this phase)
- 11b: Incremental compilation + cache (next)
- 11c: Profiles + scripts + CLI + docs (final)

**This phase establishes the solid foundation for a world-class build system.**
