# Phase 11c: Build System - Configuration & Integration

**Part 3 of 3: Build System Infrastructure**

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Phase-11a (Build System Core) and Phase-11b (Incremental Compilation) must be complete.

**Verification Steps:**
1. Check STATUS.md: Foundation section
   - phase-11a (Build System Core & Targets) should be ✅
   - phase-11b (Build System Incremental & Cache) should be ✅

2. Verify phase-11a complete:
   ```bash
   ls crates/atlas-build/src/builder.rs
   ls crates/atlas-build/src/targets.rs
   grep "test result" <(cargo test -p atlas-build builder 2>&1)
   ```

3. Verify phase-11b complete:
   ```bash
   ls crates/atlas-build/src/cache.rs
   grep "test result" <(cargo test -p atlas-build incremental 2>&1)
   ```

4. Verify combined test count:
   ```bash
   # Should have 70+ tests passing (35 from 11a, 35+ from 11b)
   cargo test -p atlas-build 2>&1 | grep "test result"
   ```

**Expected from phase-11a:**
- Core build orchestration
- Build targets working
- Parallel compilation
- 35+ tests passing

**Expected from phase-11b:**
- Incremental compilation
- Build cache working
- Change detection
- 35+ tests passing

**Decision Tree:**

a) If both phases complete (STATUS.md ✅, 70+ tests pass):
   → Proceed with phase-11c
   → Add profiles, scripts, CLI, final docs

b) If either phase incomplete (STATUS.md ⬜):
   → STOP immediately
   → Complete missing phases in order: 11a → 11b
   → Then return to phase-11c

c) If phases marked complete but tests failing:
   → One or more phases not actually complete
   → Fix failing phases first
   → Verify all tests pass
   → Then proceed with phase-11c

**No user questions needed:** Prerequisites are verifiable via STATUS.md, file existence, and cargo test.

---

## Objective

Complete the Atlas build system with build configuration profiles (dev, release, test, custom), build scripts with sandboxing, dependency building orchestration, comprehensive build output and progress reporting, CLI integration, and complete documentation. This phase delivers a world-class, production-ready build system.

**Scope:** This is part 3 of 3, the final phase. Phase-11c adds configuration, scripts, CLI integration, and comprehensive documentation to complete the build system.

## Files

**Extend:** `crates/atlas-build/src/builder.rs` (+200 lines - profile/script support)
**Create:** `crates/atlas-build/src/profile.rs` (~300 lines - build profiles)
**Create:** `crates/atlas-build/src/script.rs` (~250 lines - build script execution)
**Create:** `crates/atlas-build/src/output.rs` (~200 lines - build output formatting)
**Extend:** `crates/atlas-build/src/lib.rs` (+40 lines - new exports)
**Update:** `crates/atlas-cli/src/commands/build.rs` (~300 lines - CLI integration)
**Extend:** `crates/atlas-package/src/manifest.rs` (+100 lines - build script config)
**Create:** `crates/atlas-build/tests/profile_tests.rs` (~300 lines - profile tests)
**Create:** `crates/atlas-build/tests/script_tests.rs` (~350 lines - script tests)
**Create:** `crates/atlas-build/tests/integration_tests.rs` (~400 lines - end-to-end tests)
**Create:** `docs/build-system.md` (~800 lines - COMPLETE comprehensive docs)
**Update:** `docs/features/build-system-core.md` (+100 lines - integrate with profiles/scripts)
**Update:** `docs/features/build-system-incremental.md` (+100 lines - profile-aware caching)

**Total this phase:** ~3440 lines (code + tests + docs)

**Total phase-11 (all 3 parts): ~7910 lines**

## Dependencies

- Phase-11a: Core build system, builder, targets, build order
- Phase-11b: Incremental compilation, build cache
- Package manifest system (phase-07) - for build script config
- Security permissions (phase-15) - for sandboxing build scripts
- CLI infrastructure (atlas-cli)

## Implementation

### 1. Build Configuration Profiles (`profile.rs`)

**Profile system:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Profile {
    Dev,
    Release,
    Test,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub name: String,
    pub optimization_level: OptLevel,
    pub debug_info: bool,
    pub inline_threshold: usize,
    pub parallel: bool,
    pub incremental: bool,
    pub dependencies: DependencyProfile,
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    O0,  // No optimization (dev default)
    O1,  // Basic optimization
    O2,  // Full optimization (release default)
    O3,  // Aggressive optimization
}

#[derive(Debug, Clone)]
pub enum DependencyProfile {
    Dev,      // Use dev versions of dependencies
    Release,  // Use release versions of dependencies
}
```

**Built-in profiles:**

**Development profile (default):**
- Optimization: O0 (none)
- Debug info: true
- Parallel compilation: true
- Incremental compilation: true
- Fast compile times for rapid iteration
- Larger bytecode, slower runtime

**Release profile:**
- Optimization: O2 (full)
- Debug info: false
- Parallel compilation: true
- Incremental compilation: false (clean builds for releases)
- Slow compile times, fast runtime
- Smaller bytecode, optimized performance

**Test profile:**
- Optimization: O0 (none)
- Debug info: true
- Parallel compilation: true
- Incremental compilation: true
- Test-specific environment variables
- Include test dependencies

**Custom profiles:**
- Defined in `atlas.toml` under `[profile.custom_name]`
- Override any profile settings
- Inherit from base profile (dev/release)
- Example: benchmark profile (O3, no debug, specific flags)

**Profile configuration in manifest:**
```toml
# atlas.toml
[profile.dev]
opt_level = 0
debug_info = true
incremental = true

[profile.release]
opt_level = 2
debug_info = false
incremental = false

[profile.bench]
inherits = "release"
opt_level = 3
env_vars = { BENCHMARK_MODE = "1" }
```

**Profile selection:**
- Default: dev profile
- CLI flag: `atlas build --release`
- CLI flag: `atlas build --profile=bench`
- Environment variable: `ATLAS_PROFILE=release`

**Profile-aware caching:**
- Separate cache per profile
- Cache key includes profile name
- Different bytecode for different opt levels
- Cache invalidation on profile change

### 2. Build Scripts and Custom Steps (`script.rs`)

**Build script support:**
```rust
pub struct BuildScript {
    pub name: String,
    pub script: ScriptKind,
    pub phase: ScriptPhase,
    pub timeout: Duration,
}

pub enum ScriptKind {
    Atlas(PathBuf),      // Atlas source file
    Shell(String),       // Shell command
}

pub enum ScriptPhase {
    PreBuild,   // Before compilation
    PostBuild,  // After compilation, before linking
    PostLink,   // After linking, final step
}

pub struct ScriptContext {
    pub profile: Profile,
    pub target_dir: PathBuf,
    pub source_dir: PathBuf,
    pub env_vars: HashMap<String, String>,
}
```

**Script execution:**

**Execute build scripts:**
- Run in order: all pre-build, then compile, then all post-build, then all post-link
- Provide script context (environment)
- Capture stdout/stderr
- Report script output to user
- Abort build on script failure

**Script access to build context:**
```rust
// Environment variables available to scripts:
// ATLAS_PROFILE=dev|release|test
// ATLAS_TARGET_DIR=/path/to/target
// ATLAS_SOURCE_DIR=/path/to/src
// ATLAS_VERSION=0.2.0
// ATLAS_PACKAGE_NAME=my-package
// ATLAS_PACKAGE_VERSION=1.0.0
```

**Script timeout enforcement:**
- Default timeout: 60 seconds
- Configurable per script in manifest
- Kill script process on timeout
- Report timeout error clearly

**Script output capture:**
- Capture stdout and stderr separately
- Stream output to user in real-time (verbose mode)
- Buffer output for error reporting
- Preserve output in build log

**Security considerations:**
- Sandbox build scripts using capability system (phase-15)
- Limit file system access to project directory
- No network access by default (require permission)
- No process spawning by default (require permission)
- Script permissions defined in manifest

**Build script sandboxing:**
```toml
# atlas.toml
[build]
scripts = [
    { name = "generate-parser", path = "scripts/gen.atlas", phase = "pre-build",
      permissions = ["fs-read", "fs-write"] },
    { name = "optimize-assets", shell = "compress-assets.sh", phase = "post-build",
      permissions = ["fs-read", "fs-write"], timeout = 120 }
]
```

**Script error handling:**
- Clear error messages with script name
- Include script output in error
- Suggest fixes (check permissions, check timeout)
- Non-zero exit code = build failure

### 3. Dependency Building Orchestration

**Extend Builder for dependency builds:**
```rust
impl Builder {
    fn build_dependencies(&mut self) -> BuildResult<Vec<BuildArtifact>> {
        let deps = self.manifest.dependencies();

        // Separate local vs registry dependencies
        let (local, registry) = self.partition_dependencies(&deps);

        // Build local dependencies from source (workspace)
        let local_artifacts = self.build_local_dependencies(&local)?;

        // Download and build registry dependencies
        let registry_artifacts = self.build_registry_dependencies(&registry)?;

        Ok([local_artifacts, registry_artifacts].concat())
    }
}
```

**Build dependencies before dependents:**
- Topological sort includes dependencies
- Local workspace dependencies built from source
- Registry dependencies downloaded first
- Check for pre-built binaries (future)
- Build dependencies with appropriate profile

**Rebuild dependency on changes:**
- Track dependency source changes
- Invalidate cache if dependency changed
- Rebuild affected dependencies
- Propagate to dependents

**Handle diamond dependencies:**
- Module A depends on B and C
- Both B and C depend on D
- Build D once, reuse for B and C
- Efficient shared dependency builds

**Workspace dependency linking:**
- Detect workspace structure (multiple packages)
- Build packages in dependency order
- Link workspace packages efficiently
- Share build cache across workspace

### 4. Build Output and Progress Reporting (`output.rs`)

**Progress reporting:**
```rust
pub struct BuildProgress {
    total_modules: usize,
    compiled_modules: usize,
    current_module: Option<String>,
    start_time: Instant,
}

impl BuildProgress {
    pub fn report(&self) {
        let percent = (self.compiled_modules as f64 / self.total_modules as f64) * 100.0;
        let elapsed = self.start_time.elapsed();
        let eta = self.estimate_remaining_time();

        println!("Compiling {} ({}/{}) [{:.1}%] - ETA: {:.1}s",
                 self.current_module.as_deref().unwrap_or(""),
                 self.compiled_modules,
                 self.total_modules,
                 percent,
                 eta.as_secs_f64());
    }
}
```

**Show compilation progress per-file:**
- Real-time progress updates
- Module name currently compiling
- Percentage complete
- Modules compiled / total modules
- Estimated time remaining

**Estimate remaining build time:**
- Average compile time per module
- Modules remaining
- Simple linear estimation
- Account for parallel compilation

**Colorized output for errors and warnings:**
- Red for errors
- Yellow for warnings
- Green for success
- Gray for informational
- Use `colored` crate

**Summary statistics:**
```rust
pub struct BuildSummary {
    pub total_time: Duration,
    pub compile_time: Duration,
    pub link_time: Duration,
    pub module_count: usize,
    pub cache_hit_rate: f64,
    pub artifacts: Vec<BuildArtifact>,
}

impl BuildSummary {
    pub fn display(&self) {
        println!("Build succeeded in {:.2}s", self.total_time.as_secs_f64());
        println!("  {} modules compiled ({} from cache)",
                 self.module_count,
                 (self.module_count as f64 * self.cache_hit_rate) as usize);
        println!("  Artifacts: {}", self.artifacts.len());
    }
}
```

**Output modes:**

**Verbose mode (`--verbose`):**
- Show all compiler output
- Display cache hits/misses
- Print build script output
- Show timing for each module
- Print debug information

**Quiet mode (`--quiet`):**
- Suppress progress output
- Show only errors
- No build statistics
- Suitable for CI environments

**JSON output mode (`--json`):**
```json
{
  "success": true,
  "total_time": 4.2,
  "modules": 24,
  "cache_hit_rate": 0.83,
  "artifacts": [
    {
      "target": "library",
      "path": "target/debug/libmy-package.atl.bc",
      "size": 45678
    }
  ],
  "errors": [],
  "warnings": []
}
```

**Build logs:**
- Write build log to `target/build.log`
- Include full build output
- Timestamp each entry
- Useful for debugging build issues

**Error message formatting:**
- Clear, actionable error messages
- Source location for compile errors
- Suggestion for fixes
- Related information (dependency chain)

### 5. CLI Integration (`atlas-cli/src/commands/build.rs`)

**Build command implementation:**
```rust
// atlas-cli/src/commands/build.rs

use atlas_build::{Builder, Profile};
use clap::Parser;

#[derive(Parser)]
pub struct BuildCommand {
    /// Build profile (dev, release, test, or custom)
    #[arg(long, short = 'p', default_value = "dev")]
    profile: String,

    /// Build in release mode (shorthand for --profile=release)
    #[arg(long)]
    release: bool,

    /// Specific target to build
    #[arg(long)]
    target: Option<String>,

    /// Clean build (ignore cache)
    #[arg(long)]
    clean: bool,

    /// Verbose output
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Quiet output (errors only)
    #[arg(long, short = 'q')]
    quiet: bool,

    /// JSON output
    #[arg(long)]
    json: bool,

    /// Number of parallel jobs
    #[arg(long, short = 'j')]
    jobs: Option<usize>,

    /// Target directory
    #[arg(long)]
    target_dir: Option<PathBuf>,
}

impl BuildCommand {
    pub fn execute(&self) -> Result<(), Error> {
        // Determine profile
        let profile = if self.release {
            Profile::Release
        } else {
            Profile::from_str(&self.profile)?
        };

        // Create builder
        let mut builder = Builder::new(".")?
            .profile(profile)
            .verbose(self.verbose)
            .quiet(self.quiet);

        if let Some(jobs) = self.jobs {
            builder = builder.jobs(jobs);
        }

        if let Some(ref dir) = self.target_dir {
            builder = builder.target_dir(dir);
        }

        // Clean build if requested
        if self.clean {
            builder.clean()?;
        }

        // Execute build
        let result = builder.build()?;

        // Output results
        if self.json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if !self.quiet {
            result.summary.display();
        }

        Ok(())
    }
}
```

**CLI flags:**
- `atlas build` - Build in dev profile
- `atlas build --release` - Build in release profile
- `atlas build --profile=bench` - Build with custom profile
- `atlas build --target=binary` - Build specific target
- `atlas build --clean` - Clean build (ignore cache)
- `atlas build -v` - Verbose output
- `atlas build -q` - Quiet output
- `atlas build --json` - JSON output for tooling
- `atlas build -j 8` - 8 parallel jobs

**Build workspace:**
- Detect workspace from `atlas.toml`
- Build all packages in workspace
- Respect dependency order
- Share cache across workspace

### 6. Comprehensive Documentation (`docs/build-system.md`)

**Complete build system documentation (~800 lines):**

**Table of contents:**
1. **Introduction** - Build system overview
2. **Quick Start** - Basic build commands
3. **Build Profiles** - Dev, release, test, custom
4. **Build Targets** - Library, binary, bytecode, test
5. **Incremental Compilation** - How caching works
6. **Build Scripts** - Custom build steps
7. **Dependencies** - Building with dependencies
8. **Configuration** - atlas.toml build section
9. **CLI Reference** - All build command flags
10. **Advanced Topics** - Workspaces, parallel builds, optimization
11. **Troubleshooting** - Common build issues
12. **Performance** - Build time optimization tips

**Documentation structure:**

**Introduction:**
- What is the Atlas build system
- Design philosophy
- Comparison to other build systems (Rust cargo, Go build)

**Quick start:**
- `atlas build` - Basic build
- `atlas build --release` - Release build
- `atlas build --clean` - Clean build
- `atlas build -v` - Verbose build

**Build profiles:**
- Built-in profiles (dev, release, test)
- Custom profiles in atlas.toml
- Profile configuration options
- When to use each profile

**Build targets:**
- Library targets
- Binary targets
- Bytecode targets
- Test targets
- Multiple targets

**Incremental compilation:**
- How incremental builds work
- Cache invalidation
- Performance characteristics
- Cache management

**Build scripts:**
- Pre-build, post-build, post-link
- Atlas scripts vs shell scripts
- Script context and environment
- Sandboxing and permissions
- Script timeout

**Dependencies:**
- Local workspace dependencies
- Registry dependencies
- Dependency building order
- Diamond dependencies
- Workspace builds

**Configuration reference:**
- Complete atlas.toml build section
- All configuration options
- Examples for common scenarios

**CLI reference:**
- All command-line flags
- Examples for each flag
- Flag combinations

**Advanced topics:**
- Workspace organization
- Parallel compilation tuning
- Optimization levels
- Custom build targets
- Build caching internals

**Troubleshooting:**
- Build failures
- Cache issues
- Dependency problems
- Performance issues

**Performance:**
- Cold build vs warm build
- Incremental build best practices
- Parallel compilation tuning
- Cache optimization

## Tests (TDD - Use rstest)

**Build profile tests (5 tests):**
1. `test_dev_profile_settings` - Dev profile uses correct settings
2. `test_release_profile_optimization` - Release profile optimizes
3. `test_custom_profile_from_manifest` - Custom profile loads from atlas.toml
4. `test_cli_override_profile` - CLI flag overrides default profile
5. `test_profile_specific_dependencies` - Profile-specific deps work

**Build script tests (7 tests):**
1. `test_execute_pre_build_script` - Pre-build script runs before compilation
2. `test_execute_post_build_script` - Post-build script runs after compilation
3. `test_script_access_to_build_context` - Scripts receive environment vars
4. `test_script_failure_aborts_build` - Failed script aborts build
5. `test_script_timeout_enforcement` - Long-running script killed on timeout
6. `test_script_output_capture` - Script output captured and displayed
7. `test_sandboxing_build_scripts` - Scripts sandboxed with permissions

**Dependency building tests (3 tests):**
1. `test_rebuild_dependency_on_changes` - Dependency rebuilt when changed
2. `test_diamond_dependency_handling` - Diamond deps built efficiently
3. `test_workspace_dependency_linking` - Workspace packages linked correctly

**Build output tests (7 tests):**
1. `test_progress_reporting` - Progress updates accurate
2. `test_error_formatting` - Errors formatted clearly with colors
3. `test_warning_formatting` - Warnings formatted with colors
4. `test_build_summary_display` - Summary shows correct stats
5. `test_verbose_mode_output` - Verbose mode shows all details
6. `test_quiet_mode_output` - Quiet mode shows only errors
7. `test_json_output_format` - JSON output valid and complete

**CLI integration tests (6 tests):**
1. `test_build_command_default` - `atlas build` works
2. `test_build_with_release_flag` - `atlas build --release` works
3. `test_build_with_profile_flag` - `atlas build --profile=bench` works
4. `test_build_with_clean_flag` - `atlas build --clean` works
5. `test_build_with_verbose_flag` - `atlas build -v` works
6. `test_build_workspace` - Workspace build works

**Integration tests (7 tests):**
1. `test_end_to_end_build_single_file` - Complete build single file
2. `test_end_to_end_build_multi_file` - Complete build multi-file project
3. `test_build_with_all_features` - Build with profiles + scripts + cache
4. `test_build_performance_acceptable` - Build completes in reasonable time
5. `test_build_artifacts_complete` - All expected artifacts produced
6. `test_build_with_dependencies_and_scripts` - Complex build scenario
7. `test_workspace_build_integration` - Full workspace build

**Minimum test count for phase-11c:** 35 tests
**Target with edge cases:** 37-40 tests

**Total tests phase-11 (all 3 parts): 105-120 tests**

## Integration Points

**Uses:**
- Builder from phase-11a (core orchestration)
- Build cache from phase-11b (incremental compilation)
- Package manifest from phase-07 (build config)
- Security permissions from phase-15 (script sandboxing)
- CLI infrastructure (atlas-cli)

**Creates:**
- Build profile system
- Build script execution
- CLI build command
- Complete build system documentation

**Completes:**
- Phase-11: Complete build system infrastructure
- Foundation category: 21/21 phases

**Output:**
- World-class build system
- Professional developer experience
- Production-ready build infrastructure

## Acceptance Criteria

- ✅ Build profiles implemented (dev, release, test, custom)
- ✅ Profile configuration in atlas.toml works
- ✅ Profile selection from CLI works
- ✅ Profile-aware caching works
- ✅ Build scripts execute correctly (pre/post-build)
- ✅ Script sandboxing with permissions works
- ✅ Script timeout enforcement works
- ✅ Script output captured and displayed
- ✅ Script failure aborts build
- ✅ Dependencies built in correct order
- ✅ Diamond dependencies handled efficiently
- ✅ Workspace builds work correctly
- ✅ Build progress reporting accurate and informative
- ✅ Colorized output for errors/warnings
- ✅ Build summary displays correct stats
- ✅ Verbose mode shows all details
- ✅ Quiet mode shows only errors
- ✅ JSON output mode works
- ✅ CLI build command works with all flags
- ✅ Comprehensive documentation complete (800+ lines)
- ✅ 35+ tests pass (target: 40)
- ✅ Phase-11 total: 105+ tests pass (target: 120)
- ✅ No clippy warnings
- ✅ cargo test -p atlas-build passes (all tests)
- ✅ cargo test -p atlas-cli build passes
- ✅ Integration with all prior phases working

## Notes

**World-class compiler standards:**
- Build system rivals Rust cargo and Go build
- Professional developer experience
- Clear, actionable error messages
- Fast incremental builds
- Flexible configuration
- Comprehensive documentation

**Build system philosophy:**
- Convention over configuration
- Sensible defaults (dev profile)
- Easy to use, powerful when needed
- Clear feedback to users
- Fast iteration cycle

**Performance targets (from phase-11b):**
- Warm build (no changes): < 100ms
- Incremental build (1 file): < 500ms
- Release build: acceptable for large projects
- Parallel compilation: near-linear speedup

**Phase completion:**
- Phase-11a: Core + targets ✅
- Phase-11b: Incremental + cache ✅
- Phase-11c: Configuration + integration (this phase)
- **Phase-11 COMPLETE** - World-class build system delivered

**This phase completes the Atlas build system infrastructure and the entire Foundation category.**
