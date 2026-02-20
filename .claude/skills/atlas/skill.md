---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas - AI Workflow

**Type:** Rust compiler | **Progress:** STATUS.md | **Spec:** docs/specification/
**Memory:** Auto-loaded from `/memory/` (patterns, decisions, gates)

---

## Mode: EXECUTION (Default)

**You:** Autonomous Lead Developer (full authority, execute immediately)
**User:** Overseer (catch mistakes only, has "no technical experience")
**Phase directive = START NOW** (no permission needed)

**Never ask during execution:** "Ready?" "What's next?" "Should I proceed?" "Is this correct?"
**Answer source:** STATUS.md, phases/, memory/, docs/specification/

**Triggers:** "Next: Phase-XX" | "Start Phase-XX" | User pastes handoff

---

## Core Rules (NON-NEGOTIABLE)

### 1. Autonomous Execution
1. Check STATUS.md (verify phase not complete)
2. **Git Setup:** Create feature branch from main (see Git Workflow below)
3. Run GATE -1 (sanity check)
4. Declare workflow type
5. Execute gates 0→1→2→3→4→5→6→7 (uninterrupted)
6. **Git Finalize:** Commit, push, create PR with auto-merge
7. Deliver completion summary (PR will auto-merge when CI passes)

### 2. Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no "good enough", no partial implementations.

### 3. Acceptance Criteria (SACRED)
ALL must be met. Phase says "50+ tests" → deliver 50+ (not 45).
**ALL tests MUST pass** → 0 failures before handoff.

### 4. Intelligent Decisions (When Spec Silent)
1. Analyze codebase patterns
2. Decide intelligently
3. Log decision in memory/decisions.md (use DR-XXX format)

**Never:** Ask user | Leave TODO | Guess without analysis

### 5. World-Class Quality (NO SHORTCUTS)
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### 6. Interpreter/VM Parity (100% REQUIRED)
Both engines MUST produce identical output. Parity break = BLOCKING.

### 7. Testing Protocol (SURGICAL)
**During:** `cargo nextest run -p atlas-runtime -E 'test(exact_name)'` (ONE test)
**Per-file:** `cargo nextest run -p atlas-runtime --test <domain_file>` (validate domain file)
**Full suite:** GATE 6 only — `cargo nextest run -p atlas-runtime`
**Banned:** Creating new `tests/*.rs` files (adds binary bloat), bare `#[ignore]` without reason string

**Test placement rules (non-negotiable):**
- New tests → existing domain file (see `memory/testing-patterns.md` for the canonical file list)
- New language behavior → also add `.atlas` corpus file in `tests/corpus/pass/` or `tests/corpus/fail/`
- Unit tests (no external deps) → `#[cfg(test)]` in source file, NOT in `tests/`
- Parity tests → use `assert_parity()` helper, NEVER write duplicate interpreter+VM functions

---

## Git Workflow (REQUIRED)

**Branch naming:**
```
phase/{category}-{number}     # Phase work (e.g., phase/correctness-11)
fix/{short-description}       # Bug fixes (e.g., fix/parser-float-format)
feat/{short-description}      # Features (e.g., feat/array-slice)
ci/{short-description}        # CI/infra (e.g., ci/optimize-workflows)
```

**Before starting work:**
```bash
git checkout main && git pull                    # Sync with remote
git checkout -b phase/{category}-{number}        # Create feature branch
```

**After GATE 7 (memory check):**
```bash
git add -A && git commit -m "feat(phase): Description"   # Commit all
git push -u origin HEAD                                   # Push branch
gh pr create --title "Phase X: Title" --body "..."       # Create PR
gh pr merge --squash --auto                               # Enable auto-merge (run ONCE)
```

**Walk away - automation handles:**
- CI runs (~3-4 min)
- Auto-adds to merge queue when CI passes
- Queue runs cross-platform tests (~6 min)
- Auto-merges and auto-deletes branch
- **Do NOT run `gh pr merge` again**

**Sync local (after merge):**
```bash
git checkout main && git pull                             # Sync local
git branch -d <old-branch>                                # Clean local ref
```

**Multi-part phases (A, B, C sub-phases):**
```bash
# Stay on SAME branch, commit locally between parts
<work on part A>
cargo nextest run -p atlas-runtime                        # Local validation
git add -A && git commit -m "feat(phase-XX): Part A - description"

<work on part B>
cargo nextest run -p atlas-runtime                        # Local validation
git add -A && git commit -m "feat(phase-XX): Part B - description"

<work on part C>
cargo nextest run -p atlas-runtime                        # Local validation
git add -A && git commit -m "feat(phase-XX): Part C - description"

# ALL parts done, ALL tests pass → push ONCE
git push -u origin HEAD && gh pr create ... && gh pr merge --squash --auto
```
- **One branch, multiple commits** = traceable history
- **Local tests between parts** = catch failures early
- **Push only when complete** = no wasted CI minutes
- **Squash merge** = atomic feature on main
- **If failure:** `git log --oneline` shows which part broke

**User involvement:** NONE. Agent handles entire Git lifecycle autonomously.

---

## GATE -1: Sanity Check (ALWAYS FIRST)

0. **Main CI health:** `gh run list --branch main --limit 1` — if failed, STOP and alert user
1. **Verify:** Check phase dependencies in phase file
2. **Git check:** Ensure on feature branch (not main), working directory clean
3. **Sanity:** `cargo clean && cargo check -p atlas-runtime`
4. **On failure:** Stop, inform user with error details

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
- Assumptions without verification
- Testing protocol violations

**Required:**
- Rust best practices (Result<T, E>, no unwrap in production)
- Interpreter/VM parity (always)
- Grammar conformance (docs/specification/)
- Comprehensive testing (rstest, insta, proptest)
- Quality gates (test, clippy, fmt - all pass)

---

## Build Commands

**During development:**
```bash
cargo clean && cargo check -p atlas-runtime                          # Verify
cargo clippy -p atlas-runtime -- -D warnings                        # Zero warnings
cargo fmt -p atlas-runtime                                          # Format
cargo nextest run -p atlas-runtime -E 'test(exact_name)'            # ONE test
cargo nextest run -p atlas-runtime --test <domain_file>             # Domain file
```

**Before handoff (GATE 6):**
```bash
cargo nextest run -p atlas-runtime --test <domain_file>  # Phase domain file
cargo nextest run -p atlas-runtime                        # Full suite
cargo clippy -p atlas-runtime -- -D warnings             # Zero warnings
```

**Specialized (when relevant):**
```bash
cargo nextest run -p atlas-runtime --run-ignored all                 # Include network/slow tests
cargo nextest run -p atlas-runtime --test corpus                     # Corpus (.atlas files)
cargo bench -p atlas-runtime --bench vm                              # VM benchmarks
cargo +nightly fuzz run fuzz_parser -- -max_total_time=60            # Fuzz (lexer/parser changes)
```

**`#[ignore]` rules:** Always requires a reason string. `#[ignore = "requires network"]` ✅ — bare `#[ignore]` ❌

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass, CI is green, AND PR is merged.

**Protocol:**
1. All gates passed (tests, clippy, fmt)
2. STATUS.md updated
3. Memory checked (GATE 7)
4. Changes committed and pushed
5. PR created with auto-merge enabled
6. CI passes → auto-merge → branch auto-deleted
7. Local main synced

**Required in summary:**
- Status: "✅ PHASE COMPLETE - MERGED TO MAIN"
- Commit: Short SHA of merge commit
- Final Stats (bullets)
- Highlights (2-3 sentences + key bullets)
- Progress (simple numbers)
- Next phase ready

---

## Memory System (Auto-Loaded)

**Location:** `/memory/`
- `MEMORY.md` - Index (always loaded, 200 line cap)
- `patterns.md` - Codebase patterns (Arc<Mutex<>>, stdlib signatures, etc.)
- `decisions.md` - Architectural decisions (search DR-XXX)
- `testing-patterns.md` - Test domain files, corpus workflow, parity helpers
- `github-config.md` - Repo settings, rulesets, automation

**Usage:** Read patterns.md for codebase patterns, decisions.md for architectural context.

---

## Quick Reference

**Project structure:**
- `crates/atlas-runtime/src/` - Runtime core
- `crates/atlas-runtime/src/stdlib/` - Standard library
- `crates/atlas-runtime/src/value.rs` - Value enum (all types)
- `crates/atlas-runtime/tests/` - Integration tests
- `phases/` - Work queue (~100 lines each)
- `docs/specification/` - Language spec (grammar, syntax, types, runtime)

**Key patterns:** See memory/patterns.md
**Decisions:** See memory/decisions.md (DR-003 to DR-006 for collections)
**Gates:** See gates/ directory in this skill

---

## Summary

**Compiler-first:** Embrace necessary complexity.
**Quality-first:** Correctness over arbitrary metrics.
**Parity is sacred:** Both engines must match.
**Autonomous:** Execute immediately on phase directive.
**World-class:** Complete implementations, 100% spec compliance.
