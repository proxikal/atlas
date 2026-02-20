# Atlas Memory System

**Purpose:** Consolidated knowledge base for AI agents working on Atlas compiler.

**Auto-loaded:** Files in this directory provide patterns, decisions, and workflows.

---

## 🎯 END GOAL (Read This First)

**Atlas will become a world-class systems language rivaling Rust, Go, C.**

| Current | Future |
|---------|--------|
| Application-tier (Go-level) | Systems-tier (Rust-level) |
| Interpreter + Bytecode VM | + LLVM native backend |
| `Arc<Mutex<>>` memory | Ownership OR GC (AI decides) |

**Memory model decision:** When v1.0+ planning begins, AI agents research and decide the best approach (Rust ownership vs Go GC vs Swift hybrid). Do NOT ask user — research, analyze, propose.

**All current work enables this goal.** Frontend, types, tooling = 100% reusable for native codegen.

See `language-classification.md` for full details.

---

## File Index

**language-classification.md** - Language tier and evolution path
- Current: Application language (Go-tier), Interpreter + VM
- Future: Systems language path via LLVM backend (v1.0+)
- Frontend work is 100% reusable for native codegen

**patterns.md** - Codebase implementation patterns
- **Grammar quick reference** (syntax for if/while/for/fn/types — CHECK BEFORE WRITING ATLAS CODE)
- Collection types (`Arc<Mutex<T>>`, `.lock().unwrap()`)
- Intrinsic pattern (callback-based, interpreter + VM)
- Stdlib function pattern (non-intrinsic)
- Error handling, helper functions, test harness

**testing-patterns.md** - Testing strategies and guidelines ⚠️ READ BEFORE WRITING TESTS
- **NEVER create new test files** — add to existing domain files (see `testing-patterns.md`)
- Corpus tests (`.atlas` source files in `tests/corpus/`) — preferred for new language features
- Parity helper `assert_parity()` — always use instead of duplicate interpreter/VM functions
- `#[ignore]` must always have a reason string — bare `#[ignore]` is banned
- Fuzz targets in `crates/atlas-runtime/fuzz/` — run when modifying lexer/parser/typechecker
- Criterion benchmarks in `crates/atlas-runtime/benches/` — run when optimizing execution

**decisions.md** - Architectural decision log (DR-001 to DR-014)

**github-config.md** - Repository automation settings
- Auto-merge, auto-delete branches, ruleset config
- Required CI check: `CI Success`

---

## Quick Reference

### For AI Agents Starting Work

1. **Check STATUS.md** - Current phase and progress
2. **Read phase file** - Requirements and acceptance criteria
3. **Run GATE -1** - Sanity check before starting
4. **Reference patterns.md** - Implementation patterns
5. **Check decisions.md** - Architectural context
6. **Follow gates** - Quality checkpoints (defined in atlas skill)
7. **Update STATUS.md** - On completion

### Project Structure

```
atlas/
├── crates/atlas-runtime/       # Core runtime
│   ├── src/
│   │   ├── value.rs            # Value enum (all types)
│   │   ├── interpreter/        # Interpreter engine (expr.rs for intrinsics)
│   │   ├── vm/                 # VM engine (mod.rs for VM intrinsics)
│   │   └── stdlib/             # Standard library
│   │       ├── mod.rs          # Function registration (is_builtin, is_array_intrinsic)
│   │       ├── collections/    # HashMap, HashSet, Queue, Stack
│   │       └── {module}.rs     # Other stdlib modules
│   └── tests/                  # Integration tests
├── phases/                     # Work queue (~100 lines each)
├── docs/specification/         # Language spec
├── memory/                     # This directory
└── STATUS.md                   # Single source of truth
```

### Key File Locations

**Runtime core:**
- `crates/atlas-runtime/src/value.rs` - All Atlas types
- `crates/atlas-runtime/src/stdlib/mod.rs` - Function registration
- `crates/atlas-runtime/src/interpreter/expr.rs` - Interpreter intrinsics
- `crates/atlas-runtime/src/vm/mod.rs` - VM intrinsics

**Collections:**
- `crates/atlas-runtime/src/stdlib/collections/hashmap.rs`
- `crates/atlas-runtime/src/stdlib/collections/hashset.rs`
- `crates/atlas-runtime/src/stdlib/collections/hash.rs`
- `crates/atlas-runtime/src/stdlib/collections/queue.rs`
- `crates/atlas-runtime/src/stdlib/collections/stack.rs`

**Specifications:**
- `docs/specification/syntax.md` - Grammar and syntax
- `docs/specification/types.md` - Type system
- `docs/specification/runtime.md` - Runtime behavior

---

## Decision Quick Lookup

**Hash function design?** → DR-003
**Collection API design?** → DR-005
**Why Arc<Mutex<T>>?** → DR-009 (migrated from Rc<RefCell<T>> in phase-18)
**Why interpreter + VM?** → DR-001
**Phase file issues?** → DR-007

---

## Pattern Quick Lookup

**Writing Atlas code/tests?** → patterns.md "Grammar Quick Reference" (ALWAYS CHECK FIRST)
**Implementing intrinsic?** → patterns.md "Intrinsic Pattern"
**Implementing stdlib function?** → patterns.md "Stdlib Function Pattern"
**Error handling?** → patterns.md "Error Pattern"
**Type checking?** → patterns.md "Helper Pattern"

---

## Gate Quick Lookup

**Starting phase?** → GATE -1 (sanity check)
**Implementation done?** → GATE 1-2 (tests)
**Tests pass?** → GATE 3 (parity)
**Before handoff?** → GATE 4-6 (quality, docs, status)

**Testing protocol:**
- During dev: `cargo nextest run -p atlas-runtime -E 'test(name)'` (single test)
- Domain file: `cargo nextest run -p atlas-runtime --test <domain_file>`
- Before handoff: `cargo nextest run -p atlas-runtime` (full suite)

---

## AI Workflow Summary

**Execution Mode (Default):**
1. User says "Next: Phase-XX" or STATUS.md shows next phase
2. Create feature branch: `git checkout -b phase/{category}-{number}`
3. Run GATE -1 (sanity check)
4. Execute gates 0-7 without asking
5. Commit, push, create PR with auto-merge: `gh pr merge --squash --auto`
6. PR auto-merges when CI passes, branch auto-deleted
7. Report completion summary

**Branch naming:** `phase/`, `fix/`, `feat/`, `ci/`

**Key Principles:**
- **100% autonomous** — user gives direction, AI handles everything
- User is architect only — no Git knowledge required
- AI owns entire Git lifecycle (branch, PR, CI, merge, cleanup)
- Main branch stays clean (PRs only, CI must pass)
- Zero shortcuts (no TODOs, no stubs), world-class quality

---

## Maintenance

- Update patterns.md when new patterns emerge
- Add decisions to decisions.md (use DR-XXX format)
- Update testing-patterns.md for new testing approaches
- Gates are defined in `.claude/skills/atlas/gates/` — not duplicated in memory
- **This file must stay under 200 lines** (injected into system prompt)
