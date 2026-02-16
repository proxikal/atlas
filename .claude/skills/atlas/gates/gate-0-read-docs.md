# GATE 0: Environment Prep + Read Docs

**Condition:** Starting any task

---

## Step 1: Clean Build Artifacts (MANDATORY)

**Prevent disk bloat:** Cargo accumulates GB of build artifacts rapidly (51GB in ~5 hours).

```bash
cargo clean
```

**Why:** Fresh build environment prevents accumulation. Clean slate for each task.

**ONE TIME ONLY:** Run once at task start, not during implementation.

---

## Step 2: Learn CLI Tool (First Time Only)

**IF first time using atlas-dev CLI:**
- Read `docs/atlas-dev.md` (170 lines, AI-optimized)
- Learn surgical patterns: count → list → read
- Learn piping workflows
- Learn token-efficient queries

**SKIP if already familiar with atlas-dev.**

---

## Step 3: Get Context (CLI First, Then Docs)

1. **ALWAYS:** Run `atlas-dev context current` - Single command returns:
   - Next phase to work on (path + name)
   - Phase instructions (objectives, deliverables, acceptance criteria)
   - Dependencies and blockers
   - Related architectural decisions
   - Category progress and navigation

2. **IF needed:** Additional queries (use surgical patterns from docs/atlas-dev.md):
   - `atlas-dev summary` - Project dashboard (category progress, completion %)
   - `atlas-dev blockers` - See what phases are blocked
   - `atlas-dev decision search "keyword"` - Find relevant decisions

3. **ROUTING:** Read `Atlas-SPEC.md` (INDEX only - use routing table)
4. **SELECTIVE:** Read ONLY the spec files your task needs:

### Use CLI Commands (Surgical Queries)

**Query specs/APIs from database:**
- Implementing types/generics? → `atlas-dev spec read types.md`
- Parser/grammar work? → `atlas-dev spec read syntax.md`
- Type checking? → `atlas-dev spec read language-semantics.md`
- Runtime/execution? → `atlas-dev spec read runtime.md`
- Module system? → `atlas-dev spec read modules.md`
- REPL behavior? → `atlas-dev spec read repl.md`
- Bytecode/VM? → `atlas-dev spec read bytecode.md`
- Error codes? → `atlas-dev spec read diagnostics.md`
- Stdlib API? → `atlas-dev api read stdlib.md`
- Search specs: → `atlas-dev spec search "keyword"`

### Implementation Docs (As Needed)

- Implementation guide for component: `docs/implementation/<component>.md`
- Testing patterns: `docs/guides/testing-guide.md`

---

## ⚠️ CRITICAL: Lazy Loading Rules

**DO:**
- Read Atlas-SPEC.md as index/routing ONLY
- Use routing table to find exact file needed
- Read ONLY relevant spec files for task

**DON'T:**
- Read all spec files at once
- Skip the routing table
- Guess which spec to read

**Token savings:** 80-95% (read 5-15kb instead of 150kb)

---

**BLOCKING:** Cannot proceed without understanding current state and requirements.

**Next:** GATE 0.5
