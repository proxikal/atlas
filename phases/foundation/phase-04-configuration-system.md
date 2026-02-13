# Phase 04: Configuration System

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** CLI must exist with basic command structure.

**Verification:**
```bash
ls crates/atlas-cli/src/main.rs
cargo run --bin atlas -- --help
grep -n "enum Command" crates/atlas-cli/src/main.rs
```

**What's needed:**
- atlas-cli crate with command structure
- Basic CLI functionality run compile etc
- Workspace Cargo.toml for adding new crate

**If missing:** CLI should exist from v0.1 - verify workspace structure

---

## Objective
Implement hierarchical configuration system supporting project-level atlas.toml and user-level .atlasrc settings with precedence rules, validation, and integration across all Atlas tools CLI formatter LSP compiler.

## Files
**Create:** `crates/atlas-config/` (new crate ~1000 lines total)
**Create:** `crates/atlas-config/src/lib.rs` (~200 lines)
**Create:** `crates/atlas-config/src/config.rs` (~400 lines)
**Create:** `crates/atlas-config/src/loader.rs` (~300 lines)
**Create:** `crates/atlas-config/src/schema.rs` (~100 lines)
**Update:** `Cargo.toml` (add atlas-config to workspace)
**Update:** `crates/atlas-cli/Cargo.toml` (~5 lines add dependency)
**Update:** `crates/atlas-cli/src/main.rs` (~50 lines load config)
**Tests:** `crates/atlas-config/tests/config_tests.rs` (~400 lines)
**Create:** `docs/configuration.md` (~500 lines)
**Create:** `atlas.toml.example` (~150 lines)

## Dependencies
- serde for serialization
- toml for TOML parsing
- dirs for finding config directories
- Existing CLI structure

## Implementation

### Configuration Structure
Create AtlasConfig struct with sections for compiler formatter lsp warnings runtime. Define CompilerConfig with optimization_level target debug_info source_maps fields. Define FormatterConfig with indent_size max_width trailing_comma semicolons line_ending fields. Define LspConfig with inlay_hints semantic_tokens completion_triggers diagnostics fields. Define WarningConfig with level deny allow fields. Define RuntimeConfig with profile timeout allow_io allow_network fields. Provide sensible defaults for all configurations. Use serde for TOML serialization and deserialization.

### Configuration Loader
Create ConfigLoader struct managing hierarchical loading. Implement load method with precedence order CLI args then project config then user config then defaults. Implement load_user_config searching home directory for .atlasrc and .config/atlas/config.toml. Implement load_project_config searching current and parent directories for atlas.toml. Implement merge function combining configurations with proper precedence. Handle missing files gracefully using defaults. Define ConfigError enum for IO and parse errors. Provide clear error messages on invalid configuration.

### Library Interface
Export public API from lib.rs. Provide load_config convenience function. Export AtlasConfig and all config structs. Export ConfigLoader and ConfigError. Make API ergonomic for library consumers. Document all public interfaces.

### CLI Integration
Update CLI main to load configuration on startup. Use loaded config for all commands. Pass config to run compile format commands. Allow CLI flags to override config settings. Handle config load errors gracefully with warnings. Ensure backwards compatibility with existing CLI usage.

### Example Configuration
Create comprehensive atlas.toml.example file. Document all configuration sections and fields. Provide example values and explanations. Include common use cases strict warnings production build sandboxed runtime. Show proper TOML syntax. Make it easy to copy and customize.

### Documentation
Write complete configuration guide. Explain project config atlas.toml versus user config .atlasrc. Document precedence rules CLI over project over user over defaults. Describe all configuration sections with field descriptions and defaults. Provide examples for common scenarios. Explain CLI overrides for config settings. Include troubleshooting section.

## Tests (TDD - Use rstest)

**Configuration loading tests:**
1. Load default configuration
2. Load from TOML file
3. Parse valid TOML
4. Handle invalid TOML with error
5. Handle missing fields using defaults
6. User config loading from home directory
7. Project config loading from current directory
8. Search parent directories for atlas.toml
9. Config file not found uses defaults

**Configuration merging tests:**
1. Project config overrides user config
2. CLI args override project config
3. Partial configs merge with defaults
4. Multiple sections merge independently

**Configuration validation tests:**
1. Validate optimization level range
2. Validate target enum values
3. Validate formatter settings
4. Reject invalid values with errors

**Minimum test count:** 60 tests

## Integration Points
- Uses: serde toml dirs crates
- Updates: CLI to load and use config
- Creates: atlas-config crate
- Output: Configurable Atlas tooling

## Acceptance
- Configuration files loaded correctly atlas.toml .atlasrc
- Hierarchical precedence works project over user over defaults
- All settings validated invalid values rejected
- CLI respects config settings
- Formatter uses config settings
- Compiler respects optimization level
- LSP uses config preferences
- 60+ tests pass
- Documentation complete with examples
- atlas.toml.example provided
- Config errors have helpful messages
- Missing config files use defaults gracefully
- All tools CLI formatter LSP integrated with config
- No clippy warnings
- cargo test passes
