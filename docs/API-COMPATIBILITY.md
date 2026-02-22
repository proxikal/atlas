# Atlas Runtime API Compatibility Policy

This document defines strict compatibility rules for the Atlas runtime API to ensure stability for applications embedding Atlas.

## Version Policy

Atlas follows [Semantic Versioning 2.0.0](https://semver.org/) with strict enforcement for the runtime API:

```
MAJOR.MINOR.PATCH
```

- **MAJOR** - Breaking changes to public API
- **MINOR** - New features, backward compatible
- **PATCH** - Bug fixes only, no API changes

## Runtime API Surface

The **runtime API** includes all public items in `atlas-runtime`:

- `Atlas` struct and its methods
- `RuntimeResult` type alias
- `Value` enum and its variants
- `Diagnostic` struct and fields
- `DiagnosticLevel` enum
- Re-exported public types

Changes to any of these require versioning consideration.

---

## Breaking Changes (Require MAJOR Version Bump)

### Function Signatures

❌ **Breaking:**
```rust
// v1.0.0
pub fn eval(&self, source: &str) -> RuntimeResult<Value>

// v2.0.0 - BREAKING: Changed return type
pub fn eval(&self, source: &str) -> RuntimeResult<Option<Value>>
```

❌ **Breaking:**
```rust
// v1.0.0
pub fn new() -> Self

// v2.0.0 - BREAKING: Changed to Result
pub fn new() -> Result<Self, Error>
```

### Removing Public API

❌ **Breaking:**
```rust
// v1.0.0
pub fn eval_file(&self, path: &str) -> RuntimeResult<Value>

// v2.0.0 - BREAKING: Method removed
// (no eval_file)
```

### Renaming

❌ **Breaking:**
```rust
// v1.0.0
pub struct Atlas { ... }

// v2.0.0 - BREAKING: Renamed
pub struct AtlasRuntime { ... }
```

### Changing Behavior

❌ **Breaking:**
```rust
// v1.0.0: Returns null for variable declarations
runtime.eval("let x: int = 42;") // -> Ok(Value::Null)

// v2.0.0 - BREAKING: Now returns the value
runtime.eval("let x: int = 42;") // -> Ok(Value::Int(42))
```

### Type Changes

❌ **Breaking:**
```rust
// v1.0.0
pub enum Value {
    Int(i64),
    // ...
}

// v2.0.0 - BREAKING: Changed type
pub enum Value {
    Int(i128),  // Changed from i64
    // ...
}
```

### Error Semantics

❌ **Breaking:**
```rust
// v1.0.0: Returns single diagnostic
Err(vec![diagnostic])

// v2.0.0 - BREAKING: May return multiple diagnostics
Err(vec![diagnostic1, diagnostic2])
```

---

## Non-Breaking Changes (MINOR Version Bump)

### Adding New Methods

✅ **Non-Breaking:**
```rust
// v1.0.0
impl Atlas {
    pub fn new() -> Self { ... }
    pub fn eval(&self, source: &str) -> RuntimeResult<Value> { ... }
}

// v1.1.0 - OK: New method added
impl Atlas {
    pub fn new() -> Self { ... }
    pub fn eval(&self, source: &str) -> RuntimeResult<Value> { ... }
    pub fn reset(&mut self) { ... }  // New!
}
```

### Adding Optional Parameters (via builder pattern)

✅ **Non-Breaking:**
```rust
// v1.0.0
Atlas::new()

// v1.1.0 - OK: Optional configuration via builder
Atlas::builder()
    .with_stdout(writer)
    .build()
```

### Adding New Error Codes

✅ **Non-Breaking:**
```rust
// v1.0.0: Errors AT0001-AT0050

// v1.1.0 - OK: New error codes added
// Errors AT0001-AT0075
```

### Adding Enum Variants (with #[non_exhaustive])

✅ **Non-Breaking (if marked):**
```rust
// v1.0.0
#[non_exhaustive]
pub enum Value {
    Null,
    Int(i64),
    String(Rc<String>),
}

// v1.1.0 - OK: New variant added
#[non_exhaustive]
pub enum Value {
    Null,
    Int(i64),
    String(Rc<String>),
    Float(f64),  // New!
}
```

### Expanding Traits

✅ **Non-Breaking:**
```rust
// v1.0.0
// (no traits)

// v1.1.0 - OK: New trait implementations
impl Clone for Atlas { ... }
impl Debug for Atlas { ... }
```

---

## Bug Fixes (PATCH Version Bump)

✅ **Allowed:**
- Fixing incorrect behavior to match documented specification
- Security fixes
- Performance improvements (if behavior identical)
- Internal refactoring
- Documentation improvements

❌ **Not Allowed:**
- Changing public API signatures
- Changing observable behavior (even if "fixing" it)
- Adding new public methods

---

## Deprecation Policy

### Minimum Support Window

- **Deprecation window:** At least **1 MINOR version**
- **Removal:** Only in **MAJOR version** bumps

### Deprecation Process

1. **Mark as deprecated** with `#[deprecated]` attribute
2. **Add deprecation notice** in documentation
3. **Provide migration path** in deprecation message
4. **Update CHANGELOG** with deprecation notice
5. **Wait minimum 1 MINOR release**
6. **Remove in next MAJOR version**

### Example

```rust
// v1.5.0 - Deprecate old API
#[deprecated(since = "1.5.0", note = "Use `eval_with_context` instead")]
pub fn eval(&self, source: &str) -> RuntimeResult<Value> {
    self.eval_with_context(source, Context::default())
}

// v1.6.0 - Still available but deprecated
// ... 1 minor version passes ...

// v2.0.0 - Can now be removed
// (old eval method removed)
```

---

## API Change Checklist

Before making any API change, verify:

### For All Changes

- [ ] Change documented in CHANGELOG.md
- [ ] Version number updated appropriately
- [ ] All existing tests still pass
- [ ] New tests added for new behavior
- [ ] Documentation updated

### For Breaking Changes (MAJOR)

- [ ] Migration guide written
- [ ] Users notified in advance (GitHub issue/discussion)
- [ ] Alternative non-breaking approach considered
- [ ] Deprecation period observed (if applicable)
- [ ] CHANGELOG clearly marks as BREAKING

### For New Features (MINOR)

- [ ] Backward compatibility verified
- [ ] No existing behavior changed
- [ ] Feature flag considered for experimental features
- [ ] Examples added to documentation
- [ ] Integration tests added

### For Bug Fixes (PATCH)

- [ ] Behavior change matches documented specification
- [ ] No API signature changes
- [ ] Regression test added
- [ ] Security advisory if applicable

### Documentation Updates

- [ ] `docs/runtime-api.md` updated
- [ ] `docs/runtime-api-evolution.md` reviewed
- [ ] `API-COMPATIBILITY.md` reviewed (this file)
- [ ] Code examples updated
- [ ] CHANGELOG.md entry added

---

## Stability Guarantees

### Stable (v1.0.0+)

Once Atlas reaches v1.0.0:

- ✅ **API signatures** - Will not change without MAJOR bump
- ✅ **Error codes** - Existing codes will not change meaning
- ✅ **Diagnostic format** - JSON structure stable
- ✅ **Value representation** - Enum variants stable (if not #[non_exhaustive])

### Unstable (v0.x.x)

Before v1.0.0:

- ⚠️ **MINOR versions** may include breaking changes
- ⚠️ **API is experimental** - expect changes
- ⚠️ **Document breaking changes** in CHANGELOG
- ⚠️ **Minimize breakage** where possible

### Never Guaranteed

- ❌ **Internal implementation** - May change at any time
- ❌ **Private items** - Not part of public API
- ❌ **Undocumented behavior** - May be fixed as bugs
- ❌ **Performance characteristics** - May improve/regress

---

## Version Planning

### v0.1.0 (Current)

- Initial runtime API scaffold
- Stub implementations
- API shape defined

### v0.2.0 (Planned)

- Full implementation of eval()
- Full implementation of eval_file()
- Complete diagnostic system

### v0.3.0 (Planned)

- REPL integration
- stdout redirection
- Configuration options

### v1.0.0 (Future)

- Stable API commitment
- Full test coverage
- Production ready
- Breaking changes require v2.0.0

---

## Examples of Good API Evolution

### Adding Optional Configuration

```rust
// v1.0.0
let runtime = Atlas::new();

// v1.1.0 - Backward compatible
let runtime = Atlas::new();
// OR with configuration
let runtime = Atlas::builder()
    .with_stdout(my_writer)
    .build();
```

### Extending Error Information

```rust
// v1.0.0
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Span,
}

// v1.1.0 - Add optional field
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Span,
    pub code: Option<String>,  // New!
    pub related: Vec<(Span, String)>,  // New!
}
```

### Deprecating and Replacing

```rust
// v1.0.0
impl Atlas {
    pub fn eval_string(&self, s: &str) -> RuntimeResult<Value> { ... }
}

// v1.1.0 - Add better name, deprecate old
impl Atlas {
    #[deprecated(since = "1.1.0", note = "Use `eval` instead")]
    pub fn eval_string(&self, s: &str) -> RuntimeResult<Value> {
        self.eval(s)
    }

    pub fn eval(&self, source: &str) -> RuntimeResult<Value> { ... }
}

// v2.0.0 - Remove deprecated
impl Atlas {
    pub fn eval(&self, source: &str) -> RuntimeResult<Value> { ... }
}
```

---

## Enforcement

### CI Checks

- API surface changes trigger review
- Breaking changes block merge without version bump
- All public items must have documentation

### Review Process

- API changes require maintainer approval
- Breaking changes require RFC for v1.0.0+
- Deprecations require migration guide

### Tooling

- `cargo semver-checks` to detect breaking changes (future)
- Documentation tests verify examples compile
- Integration tests verify backward compatibility

---

## References

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [docs/versioning.md](docs/versioning.md) - General versioning policy
- [docs/runtime-api.md](docs/runtime-api.md) - Runtime API specification
- [docs/runtime-api-evolution.md](docs/runtime-api-evolution.md) - Evolution guidelines

---

**Document Version:** 1.0
**Last Updated:** 2026-02-12
**Applies to:** Atlas v0.1.0+
