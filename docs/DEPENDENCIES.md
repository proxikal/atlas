# Atlas Dependencies

This document lists all external dependencies used in Atlas v0.1, with rationale and version justification.

## Dependency Policy

- **Minimize dependencies:** Only include crates that provide significant value
- **Prefer mature crates:** Prioritize well-maintained, widely-used libraries
- **Security-first:** All dependencies must pass `cargo audit` checks
- **License compliance:** MIT, Apache-2.0, or compatible licenses only
- **No parser generators:** Hand-written lexer/parser for full control

---

## Runtime Dependencies (atlas-runtime)

### thiserror (2.0)

**Purpose:** Ergonomic error type derivation

**Justification:**
- Standard for error handling in Rust libraries
- Zero runtime cost (compile-time macro)
- Provides `Error` trait implementation automatically
- Used for `RuntimeError` types in value.rs

**Alternatives Considered:**
- Manual `impl Error` - Too verbose, error-prone
- `anyhow` - Too heavy for library code, loses type information
- `snafu` - More complex API, less widely adopted

**Version Rationale:** 2.0 is latest stable, maintains MSRV compatibility

---

### serde (1.0) + serde_json (1.0)

**Purpose:** Serialization for diagnostics and bytecode

**Justification:**
- De facto standard for serialization in Rust
- Required for JSON output of diagnostics (Atlas spec requirement)
- Used for bytecode serialization/deserialization
- Highly optimized, minimal overhead
- Features: `derive` for automatic implementation

**Alternatives Considered:**
- Manual JSON serialization - Error-prone, not worth the complexity
- `miniserde` - Too limited, lacks features we need
- `simd-json` - Unnecessary complexity for our use case

**Version Rationale:** 1.0 is mature, stable, with excellent ecosystem support

---

### insta (1.40) [dev-dependency]

**Purpose:** Snapshot testing (golden tests)

**Justification:**
- Essential for end-to-end testing (Atlas spec requirement)
- Automatic snapshot management and review workflow
- Widely used in Rust compiler and tools
- Makes tests easier to write and maintain
- Dev-only dependency (no runtime impact)

**Alternatives Considered:**
- Manual golden tests - Too much boilerplate, hard to maintain
- `k9` - Less mature, fewer features
- `pretty_assertions` - Not sufficient for full snapshot testing

**Version Rationale:** 1.40 is latest stable with all features we need

---

## CLI Dependencies (atlas-cli)

### clap (4.5)

**Purpose:** Command-line argument parsing

**Justification:**
- Industry standard for CLI apps in Rust
- Derive API reduces boilerplate significantly
- Automatic help generation and validation
- Excellent error messages out of the box
- Features: `derive` for declarative CLI definition

**Alternatives Considered:**
- Manual parsing - Too error-prone, lots of boilerplate
- `structopt` - Merged into clap, superseded
- `argh` - Too minimal, lacks features we need
- `bpaf` - Less mature, smaller ecosystem

**Version Rationale:** 4.5 is latest stable, has derive macros stabilized

---

### rustyline (14.0)

**Purpose:** Line editing for REPL

**Justification:**
- Mature line editor with readline-like behavior
- Provides history, completion, and editing support
- Used by major Rust REPLs (evcxr, etc.)
- Cross-platform support (Windows, Linux, macOS)
- Essential for REPL UX (Atlas spec requirement)

**Alternatives Considered:**
- `reedline` - More modern but less mature, overkill for v0.1
- `liner` - Unmaintained, lacks features
- `termion` - Too low-level, would need to build editor ourselves

**Version Rationale:** 14.0 is latest stable, proven reliability

---

### anyhow (1.0)

**Purpose:** Error handling in CLI binary

**Justification:**
- Perfect for application (not library) error handling
- Provides context and backtrace support
- Simplifies error propagation in main()
- Widely used for CLI tools
- Only used in atlas-cli (not in library)

**Alternatives Considered:**
- `eyre` - Similar but adds complexity we don't need
- `thiserror` alone - Not ideal for application code
- Manual Result types - Too much boilerplate for CLI glue code

**Version Rationale:** 1.0 is stable and mature

---

## Transitive Dependencies

We accept transitive dependencies from the above crates. Key ones include:

- **unicode-width, unicode-segmentation** - Needed by rustyline for proper text display
- **bitflags** - Used by clap for flag management
- **proc-macro2, quote, syn** - Compile-time only, for derive macros

All transitive dependencies are audited via `cargo audit` and `cargo deny`.

---

## Version Pinning Strategy

### Workspace (Cargo.toml)
```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.70"
```

### Direct Dependencies
- Use caret requirements (e.g., `1.0`) for semantic versioning compliance
- Pin major versions to avoid breaking changes
- Allow patch updates for security fixes
- All versions tested and verified in CI

### Lockfile (Cargo.lock)
- Committed to repository for reproducible builds
- Updated regularly for security patches
- Reviewed in PR process

---

## Rejected Dependencies

### Parser Generators (EXPLICITLY NOT USED)
- `pest`, `nom`, `lalrpop`, `chumsky` - Atlas uses hand-written parser for:
  - Full control over error messages
  - Better error recovery
  - Clearer code for AI agents
  - No DSL or magic syntax

### Async Runtime (NOT NEEDED)
- `tokio`, `async-std` - Not needed for v0.1
  - No networking in v0.1
  - No concurrent I/O
  - Adds significant complexity

### Heavy Frameworks (NOT NEEDED)
- `regex` - No regex support in Atlas v0.1
- `reqwest` - No HTTP in v0.1
- `sqlx` - No database in v0.1

---

## Security Auditing

All dependencies are regularly audited:

```bash
# Run before every release
cargo audit

# Check licenses and sources
cargo deny check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for full security policy.

---

## Dependency Review Checklist

When adding a new dependency, verify:

- [ ] Actively maintained (commits in last 6 months)
- [ ] Compatible license (MIT/Apache-2.0/BSD)
- [ ] No known security vulnerabilities
- [ ] Reasonable transitive dependency count (< 20)
- [ ] Justification documented in this file
- [ ] Alternatives considered and documented
- [ ] Version pinned appropriately
- [ ] Passes `cargo deny check`

---

**Last Updated:** 2026-02-12
**Total Direct Dependencies:** 7 (4 runtime + 3 CLI)
**Total Transitive Dependencies:** ~85 (typical for Rust CLI app)
