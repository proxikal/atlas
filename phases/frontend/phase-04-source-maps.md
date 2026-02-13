# Phase 04: Source Maps Generation

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Compiler and bytecode must track source locations.

**Verification:**
```bash
grep -n "Span\|SourceLocation" crates/atlas-runtime/src/ast.rs
ls crates/atlas-runtime/src/compiler/mod.rs
cargo test compiler
```

**What's needed:**
- AST with span information from v0.1
- Compiler tracking source positions
- Bytecode format supports debug info
- Source maps format (JSON)

**If missing:** Core infrastructure should exist from v0.1

---

## Objective
Implement source map generation mapping compiled bytecode back to original source code - enabling accurate debugging, error reporting with original locations, and integration with development tools.

## Files
**Create:** `crates/atlas-runtime/src/sourcemap/mod.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/sourcemap/encoder.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/sourcemap/vlq.rs` (~200 lines)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (~200 lines emit source maps)
**Create:** `docs/source-maps.md` (~500 lines)
**Tests:** `crates/atlas-runtime/tests/sourcemap_tests.rs` (~500 lines)

## Dependencies
- Source location tracking in AST
- Compiler emitting position mappings
- VLQ encoding for compact representation
- JSON serialization

## Implementation

### Source Map Format
Use Source Map v3 specification. JSON format with mappings field. VLQ encoding for compact mappings. Map bytecode positions to source locations. Track original source files. Include source content optionally. Names array for identifiers. Version field set to 3.

### Mapping Generation
Track source location during compilation. Map each bytecode instruction to source span. Generate mapping entries during codegen. Optimize mappings removing redundant entries. Sort mappings by generated position. Handle multiple source files. Relative position encoding.

### VLQ Encoding
Encode position differences with variable-length quantities. Base64 VLQ for compact representation. Encode column, source file, line, column differences. Optimize for small values. Decode VLQ for debugging tools. Validate VLQ encoding correctness.

### Source Map Consumption
Parse source maps from JSON. Decode VLQ mappings. Lookup original position from compiled position. Resolve to source file and location. Handle missing mappings gracefully. Cache parsed source maps. Efficient binary search for lookups.

### Integration with Compiler
Generate source maps during compilation. Emit .map file alongside bytecode. Inline source maps option. Reference source map from bytecode header. Command-line flag to enable/disable. Development mode generates maps, release skips.

### Debugger Integration
Use source maps in debugger. Show original source in stack traces. Breakpoints set on original lines map to bytecode. Step through original source code. Variable names from source not bytecode. Error messages reference original code.

## Tests (TDD - Use rstest)
1. Generate source map for simple program
2. VLQ encoding correctness
3. VLQ decoding matches encoding
4. Map bytecode position to source
5. Handle multiple source files
6. Optimize redundant mappings
7. Parse generated source map
8. Lookup original position
9. Source map with inlined sources
10. Integration with debugger

**Minimum test count:** 40 tests

## Acceptance
- Source maps generated during compilation
- VLQ encoding correct and compact
- Bytecode positions map to source
- Source maps parseable
- Debugger uses source maps
- 40+ tests pass
- Documentation complete
- cargo test passes
