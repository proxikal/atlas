# Phase 05: Package Manager CLI Commands

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Package manager and manifest system must exist.

**Verification:**
```bash
ls crates/atlas-package/src/resolver.rs
ls crates/atlas-package/src/manifest.rs
cargo test --package atlas-package
```

**What's needed:**
- Package manager from foundation/phase-08
- Package manifest from foundation/phase-07
- CLI framework from v0.1
- Build system from foundation/phase-11

**If missing:** Complete foundation phases 07-08 and 11 first

---

## Objective
Implement package manager CLI commands enabling dependency installation, package publishing, version management, and project initialization - providing npm-like package management for Atlas ecosystem.

## Files
**Update:** `crates/atlas-cli/src/main.rs` (~100 lines add commands)
**Create:** `crates/atlas-cli/src/commands/install.rs` (~400 lines)
**Create:** `crates/atlas-cli/src/commands/publish.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/commands/init.rs` (~250 lines)
**Create:** `crates/atlas-cli/src/commands/add.rs` (~200 lines)
**Create:** `crates/atlas-cli/src/commands/remove.rs` (~150 lines)
**Create:** `crates/atlas-cli/src/commands/update.rs` (~200 lines)
**Create:** `docs/package-manager-cli.md` (~700 lines)
**Tests:** `crates/atlas-cli/tests/package_cli_tests.rs` (~600 lines)

## Dependencies
- Package manager infrastructure
- Package manifest system
- Build system for compiling deps
- Registry client (future: real registry)
- CLI framework

## Implementation

### atlas install Command
Install all dependencies from manifest. Read atlas.toml dependencies. Resolve dependency versions. Download packages from registry or git. Build dependencies in order. Generate or update lockfile. Install to dependencies directory. Show progress during install. Handle install errors gracefully.

### atlas add Command
Add new dependency to project. Specify package name and optional version. Resolve latest compatible version. Update atlas.toml with dependency. Run install for new dependency. Update lockfile. Support --dev flag for dev dependencies. Interactive mode for version selection.

### atlas remove Command
Remove dependency from project. Remove from atlas.toml. Optionally remove from disk. Update lockfile removing transitive deps. Warn about orphaned dependencies. Cleanup unused packages.

### atlas update Command
Update dependencies to latest versions. Check for updates respecting constraints. Update lockfile with new versions. Install updated dependencies. Show changelog or release notes. Selective update specific packages. Update all or interactive mode.

### atlas init Command
Initialize new Atlas project. Create atlas.toml manifest. Scaffold project structure. Create src directory and main file. Initialize git repository optionally. Templates for library or binary. Interactive prompts for metadata. Set reasonable defaults.

### atlas publish Command
Publish package to registry (future). Validate package before publish. Build and test package. Check version not already published. Package sources into archive. Upload to registry with metadata. Authentication required. Semantic versioning enforcement.

### Progress and Output
Show clear progress during operations. Spinner for long operations. Progress bars for downloads. Colorized output for status. Verbose mode for debugging. Quiet mode for scripts. Error messages with suggestions. Success confirmations.

## Tests (TDD - Use rstest)
1. atlas init creates project
2. atlas add adds dependency
3. atlas install installs deps
4. atlas remove removes dependency
5. atlas update updates deps
6. Lockfile generated correctly
7. Error handling missing registry
8. Dependency resolution
9. Progress output
10. Dry run mode

**Minimum test count:** 50 tests

## Acceptance
- atlas init scaffolds projects
- atlas add adds dependencies
- atlas install works correctly
- atlas remove removes deps
- atlas update updates to latest
- Lockfile maintained properly
- Progress shown clearly
- Error messages helpful
- 50+ tests pass
- Documentation comprehensive
- cargo test passes
