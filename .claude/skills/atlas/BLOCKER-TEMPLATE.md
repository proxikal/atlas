# Phase Blocker Section Template

**Purpose:** Show AI agents exactly how to verify dependencies autonomously without asking user

---

## ❌ BAD Blocker Sections (Don't Do This)

### Example 1: Vague "May Need"
```markdown
## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Type system must support extern types.

**What's needed:**
- Type system can represent C types
- Runtime can load dynamic libraries
- Calling convention compatibility

**If missing:** May need type system enhancements for extern types
```

**Problems:**
- "May need" is vague - who decides?
- No reference to spec
- Forces AI to ask user "is type system ready?"
- Causes hours of wrong implementation

---

### Example 2: Asks User to Decide
```markdown
## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Nested function support for closures.

**If missing:** Decide if nested functions should be implemented
```

**Problems:**
- AI will ask user technical question
- User says "make it standards compliant" (vague)
- AI builds wrong thing for 3 hours
- Still fixing errors

---

### Example 3: No Verification Path
```markdown
## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Security permissions and Result types must exist.

**If missing:** Complete foundation phases 09 and 15 first
```

**Problems:**
- Doesn't say HOW to verify
- No spec reference
- AI can't determine if "missing" or just not found
- Causes confusion

---

## ✅ GOOD Blocker Sections (Do This)

### Example 1: Spec-Referenced with Verification
```markdown
## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Type system must support extern type declarations.

**Verification Steps:**
1. Check spec: `docs/specification/types.md` section 7 "Foreign Types"
2. Verify implementation: `crates/atlas-runtime/src/typechecker/types.rs`
   - Look for `Type::Extern` or `Type::Foreign` variant
   - Verify extern type checking logic exists
3. Run: `cargo test typechecker | grep -i extern`

**Spec Requirements (from types.md section 7):**
- Extern type syntax: `extern type Name`
- C-compatible type mapping (int, double, char*, etc.)
- Type safety validation for extern boundaries
- No generic types in extern (for v0.2)

**If spec feature is missing from code:**
- Implement per spec section 7 exactly
- Add tests per spec examples
- Log decision: "Implemented extern types per specification/types.md section 7"

**If spec doesn't define it:**
- Feature is out-of-scope for current version
- Consult STATUS.md for version scope
- Flag as architectural question if blocking
```

**Why this works:**
- AI knows EXACTLY where to look (spec section 7)
- AI knows HOW to verify (specific file, specific code)
- AI knows WHAT to do if missing (implement per spec)
- AI knows WHEN to ask user (only if spec doesn't define it)
- No ambiguity, no user questions

---

### Example 2: Clear Dependency Chain
```markdown
## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Configuration system (foundation/phase-04) must be complete.

**Verification Steps:**
1. Check STATUS.md: Foundation section, phase-04 checkbox
2. Verify crate exists: `ls crates/atlas-config/src/lib.rs`
3. Verify API: `grep -n "pub struct AtlasConfig" crates/atlas-config/src/config.rs`
4. Run tests: `cargo test --package atlas-config`

**Expected from phase-04:**
- `AtlasConfig` struct with compiler/formatter/lsp sections
- `load_config()` function loading from atlas.toml
- Hierarchical merge (CLI > project > user > defaults)
- All phase-04 tests passing

**If phase-04 incomplete:**
- Stop immediately
- Report: "Foundation phase-04 required before this phase"
- Update STATUS.md to show correct next phase
- Do NOT attempt to implement config system here

**If phase-04 complete but API different than expected:**
- Check phase-04 acceptance criteria
- Use actual API (may have evolved)
- Log any deviations from this phase's assumptions
```

**Why this works:**
- Concrete verification commands (grep, ls, cargo test)
- Clear success criteria
- Tells AI what to do if incomplete (stop and report)
- Tells AI what to do if API changed (adapt)
- No user questions needed

---

### Example 3: Spec-Driven Feature Check
```markdown
## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Result<T, E> type for error handling.

**Verification Steps:**
1. Check spec: `docs/specification/types.md` section 5.3 "Result Types"
2. Check implementation:
   ```bash
   grep -n "Result" crates/atlas-runtime/src/typechecker/types.rs
   grep -n "Ok\|Err" crates/atlas-runtime/src/value.rs
   cargo test | grep -i result
   ```
3. Verify from v0.1: Result<T,E> is built-in generic type per v0.1 completion

**Spec Defines (types.md section 5.3):**
- `Result<T, E>` generic enum with Ok(T) and Err(E) variants
- Pattern matching required for Result values
- Type checking validates T and E type parameters
- Runtime representation as tagged union

**Decision Tree:**
a) If Result types exist and match spec:
   → Proceed with phase

b) If Result types exist but incomplete:
   → Check what spec requires vs what's implemented
   → Implement missing pieces per spec
   → Log: "Extended Result types per specification/types.md section 5.3"

c) If Result types don't exist:
   → Must be foundation/phase-09 (Result implementation phase)
   → Stop and report: "Foundation phase-09 required before stdlib/phase-10"
   → Update STATUS.md next phase to foundation/phase-09

d) If spec doesn't define Result types:
   → ERROR: Spec should define this (check Atlas-SPEC.md routing)
   → If truly missing: Flag as spec gap, ask ARCHITECTURAL question
```

**Why this works:**
- Provides decision tree for all scenarios
- References spec section precisely
- Shows what to check in code
- Tells AI what to do in each case
- Only asks user if spec itself is wrong (rare)

---

## Template for New Phases

```markdown
## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** [Feature/Phase name] must [condition].

**Verification Steps:**
1. Check spec: `docs/specification/[file].md` section [N] "[Section Name]"
2. Check implementation:
   ```bash
   [specific grep/ls/cargo commands that verify existence]
   ```
3. [Additional verification if needed]

**Spec Requirements (from [file].md section [N]):**
- [Requirement 1 from spec]
- [Requirement 2 from spec]
- [Requirement 3 from spec]

**Decision Tree:**

a) If [feature] exists and matches spec:
   → Proceed with phase

b) If [feature] exists but incomplete:
   → Check spec requirements vs implementation
   → Implement missing pieces per spec section [N]
   → Log: "Extended [feature] per specification/[file].md section [N]"

c) If [feature] doesn't exist:
   → Check if this is [prerequisite phase] responsibility
   → Stop and report: "[Prerequisite phase] required before this phase"
   → Update STATUS.md to show correct phase order

d) If spec doesn't define [feature]:
   → Check if out-of-scope for current version (see STATUS.md)
   → If truly needed: Flag as spec gap
   → Ask ARCHITECTURAL question: "Should [feature] be in v[X] scope?"

**If proceeding, what's needed:**
- [Concrete item 1 from spec]
- [Concrete item 2 from spec]
- [Concrete item 3 from spec]
```

---

## Red Flags in Blocker Sections

**If you see these, the blocker section needs rewriting:**

1. ❌ "May need" - vague, forces user question
2. ❌ "Should we implement X?" - technical question, AI should decide
3. ❌ "If missing" with no verification path - how does AI check?
4. ❌ No spec reference - AI doesn't know source of truth
5. ❌ "User decides" - only for architectural decisions
6. ❌ "Check if ready" without saying HOW - no concrete verification
7. ❌ Assumes user knows implementation status - wrong role

**Green Flags (Good Blocker Sections):**

1. ✅ Spec reference with section number
2. ✅ Concrete verification commands (grep, ls, cargo test)
3. ✅ Decision tree for all scenarios
4. ✅ Clear "If X then Y" instructions
5. ✅ Tells AI what to do if spec doesn't define feature
6. ✅ Distinguishes technical vs architectural decisions
7. ✅ Autonomous - AI can verify without asking user

---

## Phase Audit Checklist

**For each phase blocker section, verify:**

- [ ] References specific spec file and section
- [ ] Provides concrete verification commands
- [ ] Has decision tree for all scenarios
- [ ] Tells AI what to do if dependency missing
- [ ] Tells AI what to do if spec doesn't define feature
- [ ] No "may need" or "should we" language
- [ ] No technical questions that should be spec-driven
- [ ] Clear separation: spec-driven (AI) vs strategic (user)

---

**Remember:** AI is Lead Developer - makes technical decisions using spec. User is Architect - makes strategic decisions. Spec is law.
