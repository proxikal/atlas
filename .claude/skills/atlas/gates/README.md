# GATE System Index

All gates MANDATORY and BLOCKING. Cannot skip.

---

## Gate Files

| Gate | File | When Used |
|------|------|-----------|
| **GATE -1** | `gate-minus1-sanity.md` | Communication check + pre-work discussion (every request) |
| **GATE 0** | `gate-0-read-docs.md` | Starting any task |
| **GATE 0.5** | `gate-0.5-dependencies.md` | After reading docs |
| **GATE 1** | `gate-1-sizing.md` | Size estimation (structured dev) |
| **GATE 1.5** | `gate-1.5-foundation.md` | Foundation check (before coding) |
| **GATE 2** | `gate-2-implement.md` | Implementation + testing |
| **GATE 3** | `gate-3-parity.md` | Verify interpreter/VM parity |
| **GATE 4** | `gate-4-quality.md` | Quality gates (test, clippy, fmt) |
| **GATE 5** | `gate-5-docs.md` | Doc updates (3-tier strategy) |
| **GATE 6** | `gate-6-status.md` | Update STATUS.md (structured dev only) |
| **GATE 7** | `gate-7-memory.md` | Memory check (every phase) |

---

## When to Use

**Use GATE 0-5:**
- Feature implementation (implementation-driven)
- Enhancements (implementation-driven)
- Refactoring

**Use GATE 0-6:**
- Structured development (following documented plan)

**Bug fixes use GATE 2 differently:**
- Strict TDD at GATE 2 (test FIRST, then fix)
- All other gates same

**Skip workflow:**
- Research (reading only)
- Questions (no code)
- Code review
- Investigation

**Rule:** Writing `.rs` code → Use gates. Reading only → Skip.

---

## Line Limits (Compiler-Aware)

**Soft Target:** 1000 lines
**Reality:** Compiler modules often 600-2000 lines
**Rule:** Quality and correctness over arbitrary limits

**Examples from Atlas:**
- VM: 1972 lines (complex state machine)
- Bytecode: 1421 lines (instruction encoding)
- Lexer: 908 lines (tokenization, Unicode)
- Compiler: 869 lines (code generation)
- AST: 671 lines (node types)

**Enforcement:**
- GATE 1: Estimate + justify if >1000 buffered
- GATE 2: Never simplify for line counts
- Focus: Correctness, clarity, maintainability

**CRITICAL:** Atlas is a compiler, not a CRUD app. Complex algorithms are necessary. Quality matters more than arbitrary metrics.

---

## Interpreter/VM Parity (CRITICAL)

**Philosophy:** Both execution engines MUST produce identical output.

**Testing:** Every feature has both interpreter and VM tests.

**Verification:**
```bash
cargo nextest run -p atlas-runtime interpreter_tests  # Must pass
cargo nextest run -p atlas-runtime vm_tests            # Must pass
```

**Parity Requirements:**
- Identical stdout
- Identical diagnostics
- Identical runtime errors
- Identical behavior on edge cases

**BLOCKING:** Parity breaks are critical issues. Cannot proceed without fixing.

---

## Test Requirements

**Philosophy:** Comprehensive testing, implementation-driven (NOT strict TDD for features)

**Framework:** rstest (parameterized), insta (snapshots), proptest (property-based)

**When to Write Tests:**
1. **Features:** Alongside or after implementation (compiler approach)
2. **Bugs:** Before fix (strict TDD - RED → GREEN)
3. **Both engines:** Interpreter AND VM (parity required)
4. **Comprehensive:** Basic functionality, edge cases, error handling

**Mirrors Real Compilers:**
- Rust (rustc): Implementation-driven, comprehensive tests
- Go compiler: Build first, test after
- TypeScript: Implementation-driven with extensive tests
- Clang/LLVM: Complex algorithms first, tests alongside/after

**Location:** `tests/` directory, organized by component

**Commands:**
```bash
cargo nextest run -p atlas-runtime              # All tests
cargo nextest run -p atlas-runtime --lib        # Library tests only
cargo nextest run -p atlas-runtime integration  # Integration tests
```

---

## Doc Sync

**Sources of Truth:**
- `STATUS.md` — Current state, progress tracking
- `docs/specification/` — Language spec (grammar, types, runtime, bytecode)
- `memory/` — AI knowledge base (patterns, decisions, gates)

**Update Frequency:**
- Skill/gates/workflows: Rarely (rules only)
- Memory files: After phases that reveal new patterns
- Spec docs: When language behavior changes
