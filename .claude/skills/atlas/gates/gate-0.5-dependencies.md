# GATE 0.5: Check Dependencies & Phase Context

**Condition:** Docs read, requirements understood

---

## Action: Phase Intelligence Protocol

### 1. Understand Current Context (CRITICAL)

**Query and verify:**
- `atlas-dev context current` - Phase context, dependencies, blockers, related decisions (single command)
- `atlas-dev summary` - Project overview (version, category progress, total completion)
- `atlas-dev blockers` - See what phases are blocked by dependencies
- `atlas-dev decision search "relevant-keyword"` - Find decisions related to your work

**Extract:**
- Version context (what's available in this version)
- Completion status (what phases are done)
- Dependencies and blockers (from context command)
- Existing patterns (how similar features were implemented)

### 2. Verify Each Dependency EXISTS in Codebase

**For EACH dependency in phase file:**

```
Dependency: [feature name]

Check 1: Does it exist in codebase?
- Grep for implementation: `grep -r "feature" crates/`
- Check if tests exist: `ls tests/*feature*`
- Verify with `atlas-dev phase info <path>` as complete

Check 2: Does it match spec?
- Compare implementation to spec: `atlas-dev spec search "feature"`
- Verify behavior matches documented semantics

Check 3: Is it complete or partial?
- Run its tests: `cargo test feature_tests`
- Check with `atlas-dev phase info <path>` if marked complete
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

**Check version and progress:**
- `atlas-dev summary` - Shows current version, category completion %
- `atlas-dev phase list -s completed -c <category>` - See what's done in category
- `atlas-dev phase count -s completed` - Total completed phases

**v0.1 complete?**
- All v0.1 features available as building blocks
- Can reference v0.1 implementation as examples
- Don't re-implement what v0.1 already has

**v0.2 in progress?**
- Some v0.2 features may be partial
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

- [ ] Run `atlas-dev context current` - get phase context, dependencies, blockers
- [ ] Run `atlas-dev summary` - know current version and overall progress
- [ ] Run `atlas-dev decision search "<relevant-topic>"` - find related decisions
- [ ] Run `atlas-dev blockers` - see if current phase or dependencies are blocked
- [ ] Read phase dependencies from context - ALL of them
- [ ] Verified EACH dependency exists in codebase (grep, phase info)
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
