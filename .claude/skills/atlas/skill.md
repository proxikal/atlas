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

**Full state audit runs in GATE -1** — worktree state, uncommitted work, unmerged branches, build verification, security scan. See `gates/gate-minus1-sanity.md`.

---

## Roles

**User:** Co-Architect + Product Owner. Final authority on language design, memory model, roadmap, version scope. Technical input is VALID — they designed this system. Flag spec contradictions with evidence, respect final call.

**You (AI):** Lead Developer + Co-Architect. Full authority on implementation decisions, code quality, compiler standards, Rust patterns, test coverage. Execute immediately. Log decisions in auto-memory.

**Session types:**
- **Architecture session:** Co-architect. Produce locked decisions, updated docs. No code written.
- **Phase execution session:** AI executes autonomously. User triggers with phase directive.
- **Scaffolding session:** AI scaffolds one block. User approves kickoff doc first.

**Phase directive = START NOW** (no permission needed)
**Never ask:** "Ready?" | "What's next?" | "Should I proceed?" | "Is this correct?"
**Answer source:** STATUS.md, phases/, auto-memory/, docs/specification/

**Triggers:** "Next: Phase-XX" | "Start Phase-XX" | "Scaffold Block N" | User pastes handoff

---

## Core Rules (NON-NEGOTIABLE)

### 1. Autonomous Execution
1. **Run GATE -1** — full state audit
2. Check STATUS.md (verify phase not complete)
3. **Git Setup:** GATE -1 determines branch state — see `gates/git-workflow.md`
4. Declare workflow type
5. **Execute applicable gates** 0→1→2→3→4→5→6→7 (see `gates/gate-applicability.md`)
6. **Git Finalize:** Commit → PR → auto-merge — see `gates/git-workflow.md`
7. Deliver completion summary

### 2. Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no "good enough", no partial implementations.

### 3. Acceptance Criteria (SACRED)
ALL must be met. Phase says "50+ tests" → deliver 50+ (not 45).
**ALL tests MUST pass** → 0 failures before handoff.

### 4. Intelligent Decisions (When Spec Silent)
1. Grep codebase — verify actual patterns before deciding
2. Check auto-memory `decisions/*.md` — decision may already be made
3. Decide intelligently, consistent with Rust compiler standards
4. Log in auto-memory `decisions/{domain}.md` (use DR-XXX format)

**Never:** Leave TODO | Guess without verification | Contradict a locked decision
**Locked decisions:** `docs/specification/memory-model.md`, `ROADMAP.md`, `docs/internal/V03_PLAN.md`

### 5. World-Class Quality (NO SHORTCUTS)
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### 6. Interpreter/VM Parity (100% REQUIRED)
Both engines MUST produce identical output. Parity break = BLOCKING.
See `.claude/rules/atlas-parity.md` (auto-loaded on interpreter/VM/compiler files).

### 7. Testing Protocol
**Source of truth:** auto-memory `testing-patterns.md` — READ BEFORE WRITING ANY TESTS.
See `.claude/rules/atlas-testing.md` (auto-loaded on test files).

---

## Git Workflow

**See `gates/git-workflow.md`** for all commands.
**See `.claude/rules/atlas-git.md`** for full rules (auto-loaded everywhere).

**TL;DR:** All changes via PR → CI passes → auto-squash merge. Never push main directly.
**Single workspace:** `~/dev/projects/atlas/` — open this in Claude Code, not atlas-dev.

---

## Universal Bans

- Task/Explore agents (use Glob + Read + Grep directly)
- Writing code touching AST/Type/Value without running `domain-prereqs.md` queries first
- Assumptions without codebase verification (grep → verify → write)
- Stub implementations, partial work, skipped edge cases

---

## Workflow Types

After GATE -1, declare one:
- **Structured Development:** Following documented plan
- **Bug Fix:** Fixing incorrect behavior
- **Refactoring:** Code cleanup (no behavior change)
- **Debugging:** Investigation, root cause
- **Enhancement:** Adding capabilities

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass AND commit is made. Do NOT PR until the entire block is complete.

**Protocol:**
1. All gates passed (build, tests, clippy, fmt, security scan)
2. **Update STATUS.md** — Last Updated, Current State, Next, block table row
3. **Commit STATUS.md on the block branch** (same commit or follow-up)
4. Memory checked (GATE 7)
5. **Commit only** — no push, no PR (block-complete cadence)
6. Deliver summary

**PR flush trigger:** Block complete (final AC check phase done). Exception: blocking fix or CI issue.
**See `gates/git-workflow.md`** for batch flush commands.

**GATE V — run at two moments (see `gates/gate-versioning.md`):**
- After any `fix/` PR merges to main → patch tag check (automatic)
- After block AC check phase committed → minor version check (verify exit criteria)

**Required in summary:**
- Status: "✅ PHASE COMPLETE - COMMITTED (batch)"
- Final Stats (bullets)
- **Memory:** Updated X / No updates needed (MANDATORY)
- Progress (X/~140 phases — see STATUS.md block table)
- Next phase

---

## Scaffolding Protocol (trigger: "Scaffold Block N")

1. **Read** `docs/internal/V03_PLAN.md` — block spec, ACs, dependency rules
2. **Audit blast radius** — grep every file the block will touch
3. **Produce Block Kickoff doc:**
   ```
   Block N Kickoff: {Theme}
   Files affected: [verified list]
   Architectural decisions required: [none | list with pointers]
   Risks: [what could break outside this block]
   Phase list: [title + ~5 word description each]
   ```
4. **Present to user** — wait for approval ("looks right, go")
5. **Create block branch:** `git checkout -b block/{name}` — ALL work for this block lives here
6. **Only then** scaffold all phase files
7. **Commit scaffold — no push, no PR.** The scaffold commit is the first commit on the block branch.
   Phase execution commits follow on the same branch. PR opens only at block completion (Phase N).

**After block execution completes:**
- Verify all block ACs from V03_PLAN.md
- Update V03_PLAN.md with "planned vs. actual" discoveries
- Update auto-memory with new patterns/decisions
- Trigger next block scaffolding

---

## Codebase Pointers

- `crates/atlas-runtime/src/` — Runtime core (see `crates/atlas-runtime/src/CLAUDE.md`)
- `crates/atlas-lsp/src/` — LSP server (see `crates/atlas-lsp/src/CLAUDE.md`)
- `crates/atlas-jit/src/` — JIT (see `crates/atlas-jit/src/CLAUDE.md`)
- `phases/v0.3/` — Phase files by block
- `docs/specification/` — Language spec
- `docs/internal/V03_PLAN.md` — Block plan ← read before scaffolding
- auto-memory `decisions/*.md` — All locked architectural decisions
