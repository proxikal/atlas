# Architectural Decision Log

**Purpose:** Record significant architectural decisions made during implementation.

**Rule:** Only log decisions that:
1. Chose between 2+ viable approaches
2. Affect future work or system design
3. Would be valuable context for future phases

---

## Decision 001: FFI Extern Type System Design (Phase-10a)

**Date:** 2026-02-15
**Phase:** foundation/phase-10a-ffi-core-types
**Status:** Implemented

### Context

Phase-10a implements FFI type marshaling infrastructure. Need to define C-compatible types and marshaling strategy.

### Decision

**Extern Types (6 primitives):**
- `CInt` - C int (i32, platform-specific)
- `CLong` - C long (i64 on 64-bit, i32 on 32-bit)
- `CDouble` - C double (f64)
- `CCharPtr` - C char* (null-terminated string)
- `CVoid` - C void (no value)
- `CBool` - C bool (u8: 0 or 1)

**Runtime Representation:**
- Atlas `Value::Null` represents C void (no `Value::Void` variant exists)
- C strings tracked in `MarshalContext.allocated_strings` for cleanup
- Type conversions validated at marshal time

**Type Mapping:**
```
Atlas Type    → Extern Type → C Type
Number        → CInt        → i32
Number        → CLong       → i64
Number        → CDouble     → f64
String        → CCharPtr    → *const i8
Bool          → CBool       → u8 (0 or 1)
Null (void)   → CVoid       → void
```

### Alternatives Considered

1. **Use Value::Void for void:**
   - Rejected: Value enum has no Void variant
   - Null is semantically equivalent for FFI purposes

2. **More C types (short, unsigned, etc.):**
   - Rejected for v0.2: Keep minimal set (6 types)
   - Can expand in future if needed

3. **Manual memory management for strings:**
   - Rejected: Too error-prone
   - Chose: RAII with MarshalContext tracking

### Rationale

- Minimal viable set for v0.2 FFI
- Matches common C FFI patterns (Rust's libc, Python's ctypes)
- Simple bidirectional marshaling
- Safe memory management with RAII

### Impact

- Phase-10b will use these types for extern function calls
- Phase-10c will use these types for callbacks
- Future phases can add more types if needed (arrays, structs)

### References

- Implementation: `crates/atlas-runtime/src/ffi/types.rs`
- Marshaling: `crates/atlas-runtime/src/ffi/marshal.rs`
- Tests: `crates/atlas-runtime/tests/ffi_types_tests.rs`

---

**Template for future decisions:**
```markdown
## Decision XXX: Title (Phase-YY)

**Date:** YYYY-MM-DD
**Phase:** category/phase-XX
**Status:** Implemented | Proposed | Superseded

### Context
[What problem are we solving?]

### Decision
[What did we choose?]

### Alternatives Considered
[What other approaches did we consider? Why rejected?]

### Rationale
[Why is this the right choice?]

### Impact
[How does this affect future work?]

### References
[Files, docs, related decisions]
```
