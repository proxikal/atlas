---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas - AI Workflow

**Type:** Rust compiler | **Progress:** STATUS.md | **Spec:** docs/specification/
**Memory:** Claude auto-memory (patterns, decisions) | **Gates:** skill `gates/` directory

---

## On Skill Activation (EVERY SESSION)

```bash
cat .worktree-id 2>/dev/null || echo "unknown"   # Detect worktree identity
```

**Full state audit runs in GATE -1** — worktree state, uncommitted work, unmerged branches, other worktrees. See `gates/gate-minus1-sanity.md`.

---

## Mode: EXECUTION (Default)

**You:** Autonomous Lead Developer (full authority, execute immediately)
**User:** Overseer (catch mistakes only, has "no technical experience")
**Phase directive = START NOW** (no permission needed)

**Never ask during execution:** "Ready?" "What's next?" "Should I proceed?" "Is this correct?"
**Answer source:** STATUS.md, phases/, auto-memory/, docs/specification/

**Triggers:** "Next: Phase-XX" | "Start Phase-XX" | User pastes handoff

---

## Core Rules (NON-NEGOTIABLE)

### 1. Autonomous Execution
1. **Run GATE -1** — full state audit (worktree state, unfinished work, stale branches, build verification, security scan)
2. Check STATUS.md (verify phase not complete)
3. **Git Setup:** GATE -1 determines branch state — create new branch or resume existing
4. Declare workflow type
5. **Execute applicable gates** 0→1→2→3→4→5→6→7 (see `gates/gate-applicability.md` for which to run)
6. **Git Finalize:** Commit locally → merge to main → clean up feature branch (see Git Workflow)
7. Deliver completion summary

### 2. Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no "good enough", no partial implementations.

### 3. Acceptance Criteria (SACRED)
ALL must be met. Phase says "50+ tests" → deliver 50+ (not 45).
**ALL tests MUST pass** → 0 failures before handoff.

### 4. Intelligent Decisions (When Spec Silent)
1. Analyze codebase patterns
2. Decide intelligently
3. Log decision in auto-memory `decisions/{domain}.md` (use DR-XXX format)

**Never:** Ask user | Leave TODO | Guess without analysis

### 5. World-Class Quality (NO SHORTCUTS)
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### 6. Interpreter/VM Parity (100% REQUIRED)
Both engines MUST produce identical output. Parity break = BLOCKING.

### 7. Testing Protocol
**Source of truth:** auto-memory `testing-patterns.md` — READ BEFORE WRITING ANY TESTS

**CRITICAL:** Different crates have different patterns:
- **atlas-runtime:** Consolidated domain files (NO new test files)
- **atlas-lsp:** Inline server creation (NO helper functions - see testing-patterns.md)
- **atlas-cli:** Integration tests with assert_cmd

**Always check existing test files in the target crate before writing new tests.**

---

## Git Workflow (REQUIRED)

**Branch naming:**
```
phase/{category}-{number}     # Phase work (e.g., phase/correctness-11)
fix/{short-description}       # Bug fixes (e.g., fix/parser-float-format)
feat/{short-description}      # Features (e.g., feat/array-slice)
ci/{short-description}        # CI/infra (e.g., ci/optimize-workflows)
```

**START of phase (after GATE -1 state audit):**
```bash
git rebase main                                  # Sync home branch to LOCAL main
git checkout -b phase/{category}-{number}        # Create feature branch
# (or continue existing feature branch if GATE -1 detected unfinished work)
```

**DURING phase (multi-part):**
```bash
# Commit locally between parts — never leave uncommitted work at session end
cargo build --workspace                          # Must pass before committing
cargo nextest run -p atlas-runtime               # Must be 100%
git add -A && git commit -m "feat(phase-XX): Part A - description"
# Continue Part B, C, etc.
```

**END of phase (local merge):**
```bash
# 1. Final verification — all must pass
cargo build --workspace
cargo nextest run -p atlas-runtime
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt --check -p atlas-runtime

# 2. Commit
git add -A && git commit -m "feat(phase-XX): Description

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

# 3. Merge to local main
git checkout main
git merge --no-ff phase/{category}-{number} -m "feat(phase-XX): Description"
git branch -d phase/{category}-{number}

# 4. Return worktree to home branch
git checkout worktree/dev
```

**Weekly push (user says "push to GitHub"):**
```bash
# From atlas/ (main worktree) only
git push origin main
```

**BANNED:**
- Creating PRs for normal phase/doc work
- Pushing on every phase
- Working directly on `main` branch
- Leaving uncommitted changes at session end

**User involvement:** NONE. Agent handles entire Git lifecycle autonomously.

---

## GATE -1: Sanity Check (ALWAYS FIRST)

**See:** `gates/gate-minus1-sanity.md` for full steps.

---

## Workflow Types

After GATE -1, declare one:
- **Structured Development:** Following documented plan
- **Bug Fix:** Fixing incorrect behavior
- **Refactoring:** Code cleanup (no behavior change)
- **Debugging:** Investigation, root cause
- **Enhancement:** Adding capabilities

---

## Universal Rules

**Banned:**
- Task/Explore agents (use Glob + Read + Grep)
- Breaking parity
- Stub implementations
- **Writing code that touches AST/Type/Value without running auto-memory `domain-prereqs.md` queries first**
- Assumptions without verification (grep → verify → write)
- Testing protocol violations

**Required:**
- Rust best practices (Result<T, E>, no unwrap in production)
- Interpreter/VM parity (always)
- Grammar conformance (docs/specification/)
- Comprehensive testing (rstest, insta, proptest)
- Quality gates (test, clippy, fmt - all pass)

---

## Build Commands

**See:** auto-memory `testing-patterns.md` for all commands (test, clippy, fmt, bench, fuzz).

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass AND work is committed to local main.

**Protocol:**
1. All gates passed (build, tests, clippy, fmt, security scan)
2. STATUS.md updated
3. Memory checked (GATE 7)
4. Commit → Merge to local main → Clean up feature branch
5. Deliver summary

**Required in summary:**
- Status: "✅ PHASE COMPLETE - COMMITTED TO LOCAL MAIN"
- Final Stats (bullets)
- **Memory:** Updated X / No updates needed (MANDATORY - see GATE 7)
- Progress (X/131 phases)
- Next phase

---

## Memory System (Claude Auto-Memory)

**Location:** Claude's auto-memory directory (auto-loaded at session start)

### Structure (ENFORCED)
```
memory/
├── MEMORY.md           # Index ONLY (50 lines max, auto-loaded)
├── patterns.md         # Active patterns (150 lines max)
├── domain-prereqs.md   # Grep queries (stable)
├── testing-patterns.md # Test guidelines (stable)
├── decisions/          # Split by domain
│   ├── language.md     # Language core decisions
│   ├── runtime.md      # Runtime decisions
│   ├── stdlib.md       # Stdlib decisions
│   ├── cli.md          # CLI decisions (CRITICAL)
│   ├── typechecker.md  # Type system decisions
│   ├── vm.md           # VM decisions
│   └── lsp.md          # LSP decisions (add when needed)
└── archive/            # Deprecated content
```

### File Size Limits (ENFORCED)
| File | Max Lines | Action if exceeded |
|------|-----------|-------------------|
| MEMORY.md | 50 | Move content to topic files |
| patterns.md | 150 | Archive old patterns |
| decisions/{x}.md | 100 | Split further or archive |

### Rules
1. **MEMORY.md = index only** - No content, just pointers
2. **Load on-demand** - Don't read all files, read what you need
3. **Split when growing** - File approaching limit? Split by subtopic
4. **Archive, don't delete** - Move to `archive/` with date prefix
5. **Decisions by domain** - New domain? Create new file

### When to Update (GATE 7)
- Hit API surprise → update `patterns.md`
- Made architectural decision → update `decisions/{domain}.md`
- Found stale info → fix or archive it

**Rule:** Memories live in Claude auto-memory, NOT in repo.

---

## Quick Reference

**Project structure:**
- `crates/atlas-runtime/src/` - Runtime core
- `crates/atlas-runtime/src/stdlib/` - Standard library
- `crates/atlas-runtime/src/value.rs` - Value enum (all types)
- `crates/atlas-runtime/tests/` - Integration tests
- `phases/` - Work queue (~100 lines each)
- `docs/specification/` - Language spec (grammar, syntax, types, runtime)

**Key patterns:** See auto-memory `patterns.md`
**Decisions:** See auto-memory `decisions/*.md` (split by domain)
**Gates:** See gates/ directory in this skill

---

## Summary

**Compiler-first:** Embrace necessary complexity.
**Quality-first:** Correctness over arbitrary metrics.
**Parity is sacred:** Both engines must match.
**Autonomous:** Execute immediately on phase directive.
**World-class:** Complete implementations, 100% spec compliance.
