# DR-001: .atb Bytecode Format with Debug Info

**Date:** 2024-03-15
**Status:** Accepted
**Component:** VM - Bytecode

## Context
VM bytecode format needed specification for serialization and execution.

## Decision
`.atb` format defined in `docs/bytecode-format.md`:
- Binary format for compiled Atlas code
- Debug info emitted by default (source maps, line numbers)
- Versioned format for future evolution
- Serializable/deserializable for compilation caching

## Rationale
**Binary format:** Faster loading than text-based formats, smaller file size.

**Debug info by default:** Development-friendly - errors show source locations. Production builds can strip if needed.

**Versioning:** Future-proof for bytecode evolution.

## Alternatives Considered
- **Text-based bytecode:** Rejected - slower parsing, larger files, no benefit over binary for execution
- **No debug info:** Rejected - terrible developer experience, error messages useless
- **Separate debug file:** Rejected - adds complexity, easy to lose debug info, default-on better for DX

## Consequences
- ✅ **Benefits:** Fast loading, small files
- ✅ **Benefits:** Excellent error messages (source locations)
- ✅ **Benefits:** Compilation caching possible
- ⚠️  **Trade-offs:** Binary format requires deserialization (minimal overhead)
- ❌ **Costs:** Larger files with debug info (acceptable trade-off for DX)

## Implementation Notes
**Format specification:** `docs/bytecode-format.md`

**Emitter:** `crates/atlas-runtime/src/compiler.rs`

**Loader:** `crates/atlas-runtime/src/vm.rs`

## References
- Spec: `docs/bytecode-format.md`
- Related: DR-001 (Value Representation)
