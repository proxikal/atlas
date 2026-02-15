# Atlas Build System - Core Infrastructure

**Status:** Implemented (Phase-11a)
**Version:** 0.2.0
**Last Updated:** 2026-02-15

## Overview

The Atlas build system provides professional-grade build infrastructure for compiling Atlas projects. Phase-11a implements the core build orchestration, multiple build targets, dependency ordering with topological sort, and the foundation for incremental compilation.

## Architecture

### Components

1. **Builder** (`builder.rs`) - Core build orchestration engine
2. **Build Targets** (`targets.rs`) - Target type definitions (library, binary, bytecode, test, benchmark)
3. **Build Order** (`build_order.rs`) - Topological sort and dependency ordering
4. **Error Handling** (`error.rs`) - Comprehensive build error types

### Build Pipeline

```
Project Discovery → Dependency Graph → Build Order → Compilation → Linking → Artifacts
```

**Detailed Flow:**

1. **Project Discovery**
   - Load `atlas.toml` package manifest
   - Discover source files in `src/` directory
   - Identify target types (library/binary based on `lib.atlas`/`main.atlas`)

2. **Dependency Graph Construction**
   - Parse each source file to extract `import` statements
   - Build module dependency graph
   - Validate all dependencies exist

3. **Build Order Computation**
   - Perform topological sort (Kahn's algorithm)
   - Identify circular dependencies (error if found)
   - Group independent modules for parallel compilation (future)

4. **Compilation**
   - Compile modules in dependency order
   - Pipeline: Lex → Parse → Bind → TypeCheck → Compile to Bytecode
   - Collect compilation errors with source locations

5. **Linking**
   - Combine compiled bytecode from all modules
   - Generate build artifacts with metadata
   - Write to target directory

6. **Artifact Generation**
   - Create output files in conventional directory structure
   - Include build metadata (compile time, module count, version)

## Build Targets

Atlas supports multiple build target types:

### Library Target

Produces a reusable package that can be imported by other projects.

**Requirements:**
- Must have `src/lib.atlas` as entry point
- Exports public API via `export` declarations

**Output:**
- `target/debug/lib/<package-name>.atl.bc` - Compiled bytecode library

**Example:**
```toml
# atlas.toml
[package]
name = "my-library"
version = "1.0.0"
```

```atlas
// src/lib.atlas
export fn greet(name: string) -> string {
    "Hello, " + name
}
```

### Binary Target

Produces an executable program.

**Requirements:**
- Must have `src/main.atlas` with `main()` function entry point
- `main()` function is the program entry point

**Output:**
- `target/debug/bin/<package-name>.atl.bc` - Compiled executable bytecode

**Example:**
```atlas
// src/main.atlas
fn main() -> void {
    let x: number = 42;
    print(x);
}
```

### Bytecode Target

Produces standalone bytecode files for individual modules.

**Requirements:**
- Any Atlas source file

**Output:**
- `target/debug/bytecode/<module-name>.atl.bc` - Standalone bytecode

### Test Target

Builds test suite from `tests/` directory.

**Requirements:**
- Test files in `tests/` directory
- Entry point with test functions

**Output:**
- `target/debug/test/<test-name>.atl.bc` - Test executable

### Benchmark Target

Builds benchmarks from `benches/` directory (planned for phase-13).

**Requirements:**
- Benchmark files in `benches/` directory

**Output:**
- `target/debug/bench/<bench-name>.atl.bc` - Benchmark executable

## Build Order and Dependency Management

### Topological Sort

The build system uses **Kahn's algorithm** to compute build order:

1. **Compute In-Degrees:** Count dependencies for each module
2. **Queue Modules:** Start with modules that have no dependencies
3. **Process Queue:**
   - Remove module from queue
   - Add to build order
   - Decrement in-degree of dependents
   - Add modules with in-degree 0 to queue
4. **Cycle Detection:** If all modules not processed, circular dependency exists

### Parallel Build Groups

The build system identifies modules that can be compiled in parallel:

**Example Dependency Graph:**
```
D (no dependencies)
├─→ B (depends on D)
├─→ C (depends on D)
└─→ A (depends on B and C)
```

**Parallel Groups:**
- Group 0: `[D]` - No dependencies, compile first
- Group 1: `[B, C]` - Both depend only on D, compile in parallel
- Group 2: `[A]` - Depends on B and C, compile last

**Note:** Phase-11a computes parallel groups but compiles sequentially due to bytecode containing non-thread-safe types (`Rc<>`). Parallel compilation will be enabled in a future phase.

### Circular Dependency Detection

Circular dependencies are detected during build order computation:

**Example Error:**
```
Circular dependency detected: A → B → C → A
```

**Resolution:** Refactor to remove circular imports.

## Build Configuration

```rust
pub struct BuildConfig {
    pub target_dir: PathBuf,           // Output directory (default: target/debug)
    pub optimization_level: OptLevel,  // Optimization level (O0, O1, O2, O3)
    pub parallel: bool,                // Enable parallel compilation (future)
    pub verbose: bool,                 // Verbose output
}
```

### Optimization Levels

- **O0** - No optimization (fast compilation, larger bytecode)
- **O1** - Basic optimization
- **O2** - Full optimization (default for release builds)
- **O3** - Aggressive optimization

**Note:** Optimization is performed by the bytecode compiler. O0 disables optimizer passes.

## Build Statistics

The builder tracks comprehensive build statistics:

```rust
pub struct BuildStats {
    pub total_modules: usize,        // Number of modules in project
    pub compiled_modules: usize,     // Modules successfully compiled
    pub parallel_groups: usize,      // Number of parallel build groups
    pub total_time: Duration,        // Total build time
    pub compilation_time: Duration,  // Time spent compiling
    pub linking_time: Duration,      // Time spent linking
}
```

## Error Handling

Comprehensive error types with actionable messages:

- **ManifestReadError** - Failed to read `atlas.toml`
- **CircularDependency** - Circular dependency detected with cycle path
- **CompilationError** - Module compilation failed with diagnostics
- **MissingEntryPoint** - Binary target missing `main()` function
- **ModuleNotFound** - Imported module doesn't exist
- **BuildFailed** - General build failure with details

## Usage Example

```rust
use atlas_build::{Builder, BuildConfig, OptLevel};

// Create builder for project
let mut builder = Builder::new("/path/to/project")?;

// Configure build
let config = BuildConfig {
    optimization_level: OptLevel::O2,
    verbose: true,
    ..Default::default()
};

// Execute build
let context = builder.with_config(config).build()?;

// Access results
println!("Built {} modules in {:.2}s",
    context.stats.compiled_modules,
    context.stats.total_time.as_secs_f64());

for artifact in &context.artifacts {
    println!("Produced: {}", artifact.output_path.display());
}
```

## File Structure

**Build system crate:**
```
crates/atlas-build/
├── src/
│   ├── lib.rs          - Public API
│   ├── builder.rs      - Build orchestration
│   ├── targets.rs      - Build target types
│   ├── build_order.rs  - Topological sort
│   └── error.rs        - Error types
├── tests/
│   └── builder_tests.rs - Integration tests
└── Cargo.toml
```

**Output directory structure:**
```
target/
└── debug/              # Debug profile (default)
    ├── lib/            # Library artifacts
    ├── bin/            # Binary artifacts
    ├── bytecode/       # Bytecode artifacts
    └── test/           # Test artifacts
```

## Limitations (Phase-11a)

Current limitations to be addressed in future phases:

1. **No Parallel Compilation** - Bytecode contains `Rc<>` types which are not thread-safe
2. **No Incremental Compilation** - Every build is from scratch (Phase-11b adds this)
3. **No Build Profiles** - Dev/release profiles added in Phase-11c
4. **No Build Scripts** - Custom build steps added in Phase-11c
5. **No CLI Integration** - Command-line interface added in Phase-11c
6. **Simplified Linking** - Bytecode concatenation (proper linking in future)
7. **No Artifact Serialization** - Placeholder for bytecode serialization

## Future Phases

**Phase-11b: Incremental Compilation & Cache**
- Build cache system
- Change detection (timestamps + content hashing)
- Smart cache invalidation
- Incremental rebuilds

**Phase-11c: Configuration & Integration**
- Build profiles (dev, release, test, custom)
- Build scripts with sandboxing
- CLI integration (`atlas build`)
- Progress reporting
- Comprehensive documentation

## Testing

**Test Coverage:** 36 tests (target: 35-40)

**Test Categories:**
- Build order computation (14 tests)
- Build targets (14 tests)
- Builder configuration (3 tests)
- Integration tests (5 tests)

**Running Tests:**
```bash
cargo test -p atlas-build
```

## Performance Characteristics

**Phase-11a Performance:**
- Build time scales linearly with module count
- Topological sort: O(V + E) where V = modules, E = dependencies
- No caching: full recompilation on every build
- Sequential compilation

**Expected Performance (after Phase-11b):**
- Cold build: baseline (full compilation)
- Warm build (no changes): < 100ms
- Incremental build (1 file changed): < 500ms
- Cache hit rate: > 80% in typical development

## Best Practices

1. **Organize code into modules** - Better dependency management
2. **Avoid circular dependencies** - Refactor to acyclic imports
3. **Use library + binary pattern** - Separate API from executable
4. **Keep modules focused** - Smaller modules compile faster
5. **Minimize dependencies** - Faster build order computation

## Troubleshooting

**Build fails with "Circular dependency detected":**
- Check import chains in error message
- Refactor to remove circular imports
- Consider extracting shared code to separate module

**Build fails with "Module not found":**
- Verify import path matches file structure
- Check for typos in import statements
- Ensure imported file exists in src/

**Build fails with compilation errors:**
- Read diagnostic messages carefully
- Fix syntax/type errors in source code
- Ensure all dependencies are valid Atlas code

## References

- **Package Manifest:** `docs/features/package-manifest.md`
- **Module System:** `docs/features/module-system.md`
- **Dependency Resolution:** `docs/features/dependency-resolution.md`
- **Build System Implementation:** `docs/implementation/08-build-system.md` (to be created)

---

**Phase-11a Complete:** Core build infrastructure implemented with 36 passing tests, zero clippy warnings, and production-ready quality.
