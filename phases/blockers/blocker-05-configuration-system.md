# BLOCKER 05: Configuration System

**Category:** Foundation - Infrastructure
**Blocks:** Foundation Phase 4, Phase 7 (Package Manifest), CLI phases
**Estimated Effort:** 1-2 weeks
**Complexity:** Medium

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Core runtime stable, no major refactors planned.

**Verification:**
```bash
cargo build --release
cargo test --all --no-fail-fast
ls crates/atlas-*/Cargo.toml
```

**What's needed:**
- Atlas runtime crate stable
- CLI crate exists
- No pending major restructuring

**If missing:** Stabilize crate structure first.

---

## Objective

Create `atlas-config` crate for configuration management. Enables project-level and global settings, package manifest support, and CLI behavior customization. Foundation for package management and developer experience.

**Core capability:** Load, validate, and manage configuration from multiple sources.

---

## Background

Atlas currently has no configuration system. Need config for:
- Project settings (atlas.toml in project root)
- Global settings (~/.atlas/config.toml)
- Package manifest (name, version, dependencies)
- CLI behavior (formatting, linting, etc.)
- Runtime behavior (security, permissions)

**Design reference:** Cargo's config system (layered, TOML-based, hierarchical).

---

## Files

### Create
- `crates/atlas-config/` - New crate (~1200 lines total)
  - `Cargo.toml` - Crate manifest with serde, toml dependencies
  - `src/lib.rs` (~200 lines) - Public API
  - `src/project.rs` (~300 lines) - Project config (atlas.toml)
  - `src/global.rs` (~200 lines) - Global config (~/.atlas/)
  - `src/manifest.rs` (~300 lines) - Package manifest
  - `src/loader.rs` (~200 lines) - Config loading logic

### Modify
- `crates/atlas-cli/Cargo.toml` - Add atlas-config dependency
- `crates/atlas-cli/src/main.rs` (~30 lines) - Load config at startup
- `crates/atlas-cli/src/commands/*.rs` - Use config in commands

### Tests
- `crates/atlas-config/tests/config_tests.rs` (~400 lines)
- `crates/atlas-config/tests/manifest_tests.rs` (~300 lines)

**Minimum test count:** 60+ tests

---

## Implementation

### Step 1: Crate Structure
Create new `atlas-config` crate in workspace. Add to workspace Cargo.toml. Dependencies: serde, serde_derive, toml, dirs (for home directory).

**Cargo.toml:**
```toml
[package]
name = "atlas-config"
version = "0.2.0"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "5.0"
```

### Step 2: Project Configuration Schema
Define ProjectConfig struct for atlas.toml in project root. Fields: name, version, edition, source directories, output directories, compiler settings.

**Configuration hierarchy:**
1. Global config (~/.atlas/config.toml) - defaults
2. Project config (./atlas.toml) - overrides global
3. Environment variables (ATLAS_*) - overrides all
4. CLI flags - final override

### Step 3: Global Configuration
Define GlobalConfig for user-level settings. Located at `~/.atlas/config.toml`. Contains: default edition, formatting preferences, linting settings, permission defaults.

### Step 4: Package Manifest
Define Manifest struct for package metadata. Fields: name, version, authors, description, dependencies, dev-dependencies. Dependencies stored as HashMap<String, Dependency> where Dependency has version, source (registry/git/path).

**Example manifest:**
```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2026"

[dependencies]
http = "1.0"
json = "0.5"
```

### Step 5: Configuration Loader
Implement loader with search algorithm. Start from current directory, walk up to find atlas.toml. Load global config from ~/.atlas/. Merge configs with proper precedence. Validate all fields. Clear error messages for invalid config.

**Loading flow:**
1. Detect project root (find atlas.toml)
2. Load global config
3. Load project config
4. Apply environment overrides
5. Validate merged config
6. Return Config struct

### Step 6: Validation
Validate all config fields. Check version formats (semver). Validate paths exist or are valid. Check dependency specs are well-formed. Error on unknown fields (strict mode). Warnings for deprecated fields.

### Step 7: CLI Integration
Integrate into atlas-cli. Load config early in main(). Make config available to all commands. Commands can query config for behavior. Support --config flag to override config file location.

### Step 8: Error Handling
Clear error messages for config issues. Point to exact line/field in TOML file. Suggest fixes for common mistakes. Document all config options. Provide config templates/examples.

### Step 9: Comprehensive Testing
Test loading from all sources. Test precedence (global < project < env < CLI). Test validation catches errors. Test missing config (graceful defaults). Test malformed TOML. Test config merging logic. Test path resolution.

---

## Architecture Notes

**Layered config:** Multiple sources with clear precedence order. Mirrors Cargo/rustc approach.

**TOML format:** Human-readable, well-supported, standard in Rust ecosystem.

**Config discovery:** Walk directory tree up to find project root (atlas.toml marker). Standard pattern.

**Validation:** Strict by default - unknown fields are errors. Prevents typos and outdated configs.

**Separation:** Config crate independent of CLI/runtime. Can be used by other tools (LSP, formatters).

---

## Acceptance Criteria

**Functionality:**
- ✅ Load config from project (atlas.toml)
- ✅ Load config from global (~/.atlas/config.toml)
- ✅ Merge configs with correct precedence
- ✅ Environment variable overrides work
- ✅ CLI flag overrides work
- ✅ Validation catches invalid configs
- ✅ Clear error messages for config issues
- ✅ Package manifest parsing works

**Quality:**
- ✅ 60+ tests pass
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ Config validation comprehensive
- ✅ Error messages helpful
- ✅ Documentation complete

**Documentation:**
- ✅ Config schema documented
- ✅ All fields explained
- ✅ Examples provided
- ✅ Migration guide (if breaking changes)
- ✅ CLI integration documented

---

## Dependencies

**Requires:**
- Stable crate structure
- CLI crate exists
- No major refactors pending

**Blocks:**
- Foundation Phase 4: Configuration System (this phase)
- Foundation Phase 7: Package Manifest (uses Manifest struct)
- Foundation Phase 8: Package Manager (needs dependency specs)
- CLI enhancements needing config
- LSP server (may need config)

---

## Rollout Plan

1. Create crate structure (1 day)
2. Define config schemas (2 days)
3. Implement loader (2 days)
4. Add validation (2 days)
5. CLI integration (1 day)
6. Testing (3 days)
7. Documentation (2 days)

**Total: ~13 days (2 weeks with polish)**

---

## Known Limitations

**No workspace support yet:** Only single-project configs. Workspaces (multiple packages) come later.

**No registry config yet:** Dependency resolution uses default registry. Custom registries in Phase 8.

**No build profiles yet:** Debug/release profiles come with compiler optimizations phase.

**No config validation plugins:** All validation built-in. Plugin system deferred.

These are acceptable for initial config system. Can extend later.

---

## Examples

**atlas.toml (project config):**
```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2026"

[dependencies]
http = "1.0"
json = "0.5"

[build]
output = "target"
source = "src"
```

**~/.atlas/config.toml (global config):**
```toml
[defaults]
edition = "2026"

[formatting]
indent = 4
max_line_length = 100

[permissions]
network = "deny"
filesystem = "prompt"
```

**Environment override:**
```bash
ATLAS_EDITION=2027 atlas build
```

---

## Risk Assessment

**Medium risk:**
- TOML parsing edge cases
- Path resolution cross-platform
- Config precedence bugs
- Validation complexity

**Mitigation:**
- Use battle-tested toml crate
- Test on all platforms
- Clear precedence rules
- Comprehensive validation tests
- Reference Cargo implementation

**This is infrastructure. Test thoroughly.**
