---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas - AI Workflow

**Type:** Rust compiler | **Progress:** STATUS.md | **Spec:** docs/specification/
**Memory:** Claude auto-memory (patterns, decisions) | **Gates:** skill `gates/` directory

---

## On Skill Activation (EVERY SESSION)

### Step 1: Detect Worktree Identity

```bash
cat .worktree-id 2>/dev/null || echo "unknown"
```

| `.worktree-id` | Path | Purpose |
|----------------|------|---------|
| `main` | `~/dev/projects/atlas/` | Brainstorm, reference, read-only |
| `dev` | `~/dev/projects/atlas-dev/` | Executing any numbered phase (code, docs, spec, or mixed) |
| `docs` | `~/dev/projects/atlas-docs/` | Standalone doc work outside a numbered phase |

### Step 2: Classify the User's Request

**→ Executing a numbered phase** (any output — Rust code, spec updates, doc generation, STATUS.md, or all of the above):
- Required worktree: `dev`
- If not in `dev` worktree → switch internally (see Mismatch Protocol below)

**→ Standalone doc work** (spec rewrite, README overhaul, batch phase file creation, skill updates, CLAUDE.md — when NOT driven by a numbered phase):
- Required worktree: `docs`
- If not in `docs` worktree → switch internally (see Mismatch Protocol below)

**→ Brainstorm / planning / questions:**
- Any worktree is fine, no git ops needed

### Step 3: Sync (code/docs work only)

```bash
git fetch origin
git rebase origin/main   # Bring this worktree up to date with latest main
```

### Step 4: Branch Setup

- If on home branch (`worktree/dev` or `worktree/docs`) → create feature branch
- If already on feature branch → continue (previous session's work)
- **NEVER work directly on `main`**

---

## Mismatch Protocol (AGENT ENFORCED)

If the user's request doesn't match the current worktree, **switch internally — never ask the user to close the session.**

1. Detect the correct worktree path from the identity table
2. Announce the switch briefly (one line):
   > "Opened in `atlas/` but this is dev work — switching to `atlas-dev/` automatically."
3. Use absolute paths for all file operations pointing at the correct worktree
4. Use `git -C /correct/worktree/path <command>` for all git operations
5. Proceed immediately — no interruption to the user

**The user never needs to close or reopen a session due to wrong worktree.**
**Never ask the user which worktree to use — classify it yourself and switch.**

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
1. Detect worktree identity → classify request → handle mismatch if needed
2. Check STATUS.md (verify phase not complete)
3. **Git Setup:** Sync from origin/main, create feature branch (see Git Workflow)
4. Run GATE -1 (sanity check + local security scan)
5. Declare workflow type
6. **Execute applicable gates** 0→1→2→3→4→5→6→7 (see `gates/gate-applicability.md`)
7. **Git Finalize:** Commit locally, merge to main (see Git Workflow)
8. Deliver completion summary

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

### Branch Naming
```
phase/{category}-{number}     # Phase work (e.g., phase/correctness-11)
fix/{short-description}       # Bug fixes (e.g., fix/parser-float-format)
feat/{short-description}      # Features (e.g., feat/array-slice)
docs/{short-description}      # Doc/spec/skill updates (e.g., docs/update-spec-closures)
```

### START of Work (every session)
```bash
git fetch origin
git rebase origin/main                           # Sync this worktree with latest main
git checkout -b phase/{category}-{number}        # Create feature branch
# (or continue existing feature branch if resuming)
```

### DURING Work (multi-part phases)
```bash
# Commit locally between parts — never leave uncommitted work at session end
git add -A && git commit -m "feat(phase-XX): Part A - description"
# Continue with Part B, C, etc.
```

### END of Phase (local merge — no PR)
```bash
# 1. Verify everything passes
cargo build --workspace                          # MUST build clean
cargo nextest run -p atlas-runtime               # MUST pass 100%
cargo clippy -p atlas-runtime -- -D warnings     # MUST be clean

# 2. Commit final state
git add -A && git commit -m "feat(phase-XX): Complete description

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

# 3. Merge to main locally
git checkout main
git merge --no-ff phase/{category}-{number} -m "feat(phase-XX): Description"
git branch -d phase/{category}-{number}          # Clean up feature branch

# 4. Return worktree to home branch
git checkout worktree/dev   # (or worktree/docs for docs worktree)
```

### Weekly Push (user-initiated or milestone)
```bash
# Run from atlas/ (main worktree) — user says "push to GitHub"
git push origin main
```

### BANNED
- Creating PRs for normal phase/doc work
- Pushing on every phase (weekly cadence only)
- `git push --force` on main
- Working on `main` branch directly
- Leaving uncommitted changes at session end (this broke PR #96)

### Why No PRs for Normal Work
- No CI = PRs provide no protection
- Squash merge loses commit history
- Local quality gates (build + test + clippy) ARE the safety net
- Weekly push keeps GitHub as clean backup/showcase

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
- Uncommitted work at session end

**Required:**
- Rust best practices (Result<T, E>, no unwrap in production)
- Interpreter/VM parity (always)
- Grammar conformance (docs/specification/)
- Comprehensive testing (rstest, insta, proptest)
- Quality gates (build, test, clippy, fmt - all pass before commit)

---

## Build Commands

**See:** auto-memory `testing-patterns.md` for all commands (test, clippy, fmt, bench, fuzz).

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass AND work is committed locally on main.

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

**Worktree Paths:**
- `~/dev/projects/atlas/` → `main` worktree (brainstorm/reference)
- `~/dev/projects/atlas-dev/` → `dev` worktree (executing any numbered phase)
- `~/dev/projects/atlas-docs/` → `docs` worktree (standalone doc work outside a phase)

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
**Worktree-aware:** Detect identity, classify request, never silently work in wrong worktree.
