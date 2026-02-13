# Phase 06: Project Scaffolding and Templates

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Package manager CLI must exist.

**Verification:**
```bash
ls crates/atlas-cli/src/commands/init.rs
cargo test package_cli
```

**What's needed:**
- Package manager CLI from cli/phase-05
- Project template system
- File generation utilities

**If missing:** Complete cli/phase-05 first

---

## Objective
Implement project templates and scaffolding enabling quick project creation with best practices, common patterns, and ready-to-use structure - providing create-react-app or cargo-new like experience for Atlas.

## Files
**Create:** `crates/atlas-cli/src/templates/mod.rs` (~400 lines)
**Create:** `crates/atlas-cli/src/templates/library.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/templates/binary.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/templates/web.rs` (~400 lines)
**Update:** `crates/atlas-cli/src/commands/new.rs` (~300 lines)
**Create:** `templates/` (template files directory)
**Create:** `docs/project-templates.md` (~500 lines)
**Tests:** `crates/atlas-cli/tests/scaffolding_tests.rs` (~500 lines)

## Dependencies
- CLI framework
- Package manager
- File system operations
- Template rendering engine

## Implementation

### Template System
Define template structure with variable substitution. Template files with placeholders. Metadata describing template. Template dependencies and setup. Support multiple template types. Template versioning. Custom templates from git.

### Library Template
Template for library projects. Exports public API. Documentation examples. Test structure. CI configuration. README with badges. License file. Contributing guide. Standard library structure.

### Binary Template
Template for executable projects. Main entry point. CLI argument parsing example. Configuration file support. Logging setup. Error handling patterns. Installation instructions. Usage documentation.

### Web Server Template
Template for web applications (future). HTTP server setup. Routing structure. Middleware examples. Static file serving. API endpoint structure. Database connection example. Environment configuration. Docker support.

### Template Variables
Project name substitution. Author information. License selection. Version initialization. Description and keywords. Custom variable support. Interactive prompts for variables. Default values.

### atlas new Command
Create new project from template. Specify template type. Interactive or non-interactive mode. Validate project name. Create directory structure. Generate files from template. Initialize git repository. Install initial dependencies. Success message with next steps.

## Tests (TDD - Use rstest)
1. Create library project
2. Create binary project
3. Template variable substitution
4. Directory structure correct
5. Files generated properly
6. Git initialization
7. Custom template support
8. Invalid name rejection
9. Existing directory handling
10. Template selection

**Minimum test count:** 40 tests

## Acceptance
- atlas new creates projects
- Library template functional
- Binary template functional
- Variables substituted correctly
- Git initialized
- Documentation generated
- 40+ tests pass
- Template guide complete
- cargo test passes
