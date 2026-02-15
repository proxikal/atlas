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

## Step 2: Read Docs (Selective Reading)

1. **ALWAYS:** Read `STATUS.md` (current state, progress, doc map with routing)
2. **IF structured development:** Read complete development plan
3. **ROUTING:** Read `Atlas-SPEC.md` (INDEX only - use routing table)
4. **SELECTIVE:** Read ONLY the spec files your task needs:

### Use Routing Table (DO NOT read all specs)

**From Atlas-SPEC.md routing table:**
- Implementing types/generics? → Read `docs/specification/types.md`
- Parser/grammar work? → Read `docs/specification/syntax.md`
- Type checking? → Read `docs/specification/language-semantics.md`
- Runtime/execution? → Read `docs/specification/runtime.md`
- Module system? → Read `docs/specification/modules.md`
- REPL behavior? → Read `docs/specification/repl.md`
- Bytecode/VM? → Read `docs/specification/bytecode.md`
- Error codes? → Read `docs/specification/diagnostics.md`
- Stdlib API? → Read `docs/api/stdlib.md`

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
