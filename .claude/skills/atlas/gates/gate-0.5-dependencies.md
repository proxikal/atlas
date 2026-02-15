# GATE 0.5: Check Dependencies & Phase Context

**Condition:** Docs read, requirements understood

---

## Action: Phase Intelligence Protocol

### 1. Understand Current Context (CRITICAL)

**Read and verify:**
- `STATUS.md` - Current version (v0.1? v0.2?), what's complete, what's in progress
- `docs/reference/decision-log.md` - Architectural decisions already made
- Current phase file - ALL dependencies and blockers

**Extract:**
- Version context (what's available in this version)
- Completion status (what phases are done)
- Existing patterns (how similar features were implemented)

### 2. Verify Each Dependency EXISTS in Codebase

**For EACH dependency in phase file:**

```
Dependency: [feature name]

Check 1: Does it exist in codebase?
- Grep for implementation: `grep -r "feature" crates/`
- Check if tests exist: `ls tests/*feature*`
- Verify in STATUS.md as complete

Check 2: Does it match spec?
- Compare implementation to `docs/specification/`
- Verify behavior matches documented semantics

Check 3: Is it complete or partial?
- Run its tests: `cargo test feature_tests`
- Check if marked complete in STATUS.md
- Verify quality (not a stub)

Status:
✅ Exists, complete, spec-compliant → Proceed
⚠️ Exists but incomplete → Flag, may need to finish first
🚫 Doesn't exist → BLOCKING, implement dependency first
```

### 3. Build on Existing Code (Don't Reinvent)

**Before implementing ANYTHING:**
- Search for similar existing code (`grep`, `rg`)
- Read how it was done before (consistency matters)
- Follow established patterns (don't introduce new style)
- Check decision log for architectural constraints

**Examples:**
- Adding new AST node? See how existing nodes are structured
- Adding typechecker rule? See how existing rules work
- Adding stdlib function? Follow existing stdlib patterns

### 4. Version-Aware Context

**v0.1 complete?**
- All v0.1 features available as building blocks
- Can reference v0.1 implementation as examples
- Don't re-implement what v0.1 already has

**v0.2 in progress?**
- Some v0.2 features may be partial
- Check STATUS.md for exact progress
- Don't assume v0.2 features exist yet

### 5. Check Affected Components

**What will this change touch?**
- Lexer, parser, AST, typechecker?
- Compiler, VM, interpreter?
- Stdlib, LSP, runtime?

**For each component:**
- Read existing code BEFORE modifying
- Understand current architecture
- Note integration points
- Plan for parity (both engines)

---

## Verification Checklist

Before proceeding to GATE 1:

- [ ] Read STATUS.md - know current version and progress
- [ ] Read decision-log.md - know architectural decisions
- [ ] Read phase dependencies - ALL of them
- [ ] Verified EACH dependency exists in codebase
- [ ] Found existing patterns to follow
- [ ] Understand version context (v0.1 vs v0.2)
- [ ] Know which components will be affected
- [ ] Read existing code for those components

---

## Anti-Patterns (DON'T DO THIS)

❌ **Assume dependency exists** - Always verify in codebase
❌ **Reinvent patterns** - Search for existing patterns first
❌ **Ignore version context** - v0.1 vs v0.2 matters
❌ **Skip decision log** - May violate architectural decisions
❌ **Write code before reading** - Always read existing code first
❌ **Duplicate existing code** - Search first, reuse patterns

---

**BLOCKING:** If dependencies missing or incomplete, STOP. Report issue. Do not proceed.

**CRITICAL:** This gate prevents wasted work. Phase intelligence = faster development.

**Next:** GATE 1
