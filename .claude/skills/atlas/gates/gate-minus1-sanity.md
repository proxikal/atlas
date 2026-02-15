# GATE -1: Spec-First Verification & Communication Check

**Purpose:** Verify against spec/docs AUTONOMOUSLY + enforce communication rules + catch issues BEFORE work starts

**Condition:** User requests work (implementation, phases, features)

**CRITICAL:** AI is LEAD DEVELOPER - makes technical decisions using spec/docs. User is ARCHITECT - makes strategic decisions.

---

## Action Flow

### Part 1: Communication Check (ALWAYS RUNS)

**Apply Rules 1 & 2 to ALL interactions:**
- ✓ Am I making assumptions? → Verify using spec/docs FIRST, not user
- ✓ Am I being brutally honest and concise? → Short, direct, informative

**Applies to:** Questions, brainstorming, implementation, discussions, EVERYTHING

---

### Part 2: Spec-First Verification (FOR ALL IMPLEMENTATION WORK)

**DO THIS AUTONOMOUSLY - DO NOT ASK USER:**

1. **Read phase blockers/dependencies:**
   - If phase file exists: Read `🚨 BLOCKERS` section completely
   - List all "required" dependencies
   - List all "if missing" scenarios

2. **For EACH dependency, verify using spec/docs:**
   ```
   Dependency: [feature name]

   Step 1: Check specification
   - Look in `docs/specification/` for feature definition
   - Search `Atlas-SPEC.md` routing table
   - Read relevant spec section

   Step 2: Determine status
   - ✅ Spec defines it AND it's implemented → Proceed
   - ⚠️ Spec defines it BUT not implemented → Implement per spec
   - 🚫 Spec DOESN'T define it → Check if out-of-scope for current version

   Step 3: Log decision (if implementing)
   - Add entry to `docs/reference/decision-log.md`
   - Format: "Implemented X per spec section Y for phase Z"
   ```

3. **NEVER ask user these questions:**
   - ❌ "Should we implement nested functions?"
   - ❌ "Do we need async/await?"
   - ❌ "Should Result types work this way?"
   - ❌ "Is the type system ready for X?"

   **Instead: CHECK SPEC, DECIDE, LOG**

4. **Gather version/progress context:**
   - Read `STATUS.md` (current version, phase, what's complete)
   - Read current phase file dependencies
   - Verify prerequisites are met

---

### Part 3: Sanity Check (AFTER Spec Verification)

**Evaluate request against check categories:**

1. **Version scope:** Is this in current version's scope per STATUS.md?
2. **Dependencies met:** Did spec verification confirm all deps ready?
3. **Design consistency:** Does this align with Atlas compiler identity?
4. **Workload:** Is this too ambitious for one session?
5. **Priority:** Are there blockers that need fixing first?
6. **Approach:** Does approach align with compiler best practices?
7. **Consistency:** Will this break interpreter/VM parity?

**If concerns found:**
- Present concern with evidence from spec/docs
- Explain current context
- Suggest spec-compliant alternatives
- Ask ARCHITECTURAL questions only (direction, priorities, trade-offs)
- Frame as discussion, not blocking

**If no concerns:** Proceed to GATE 0

---

## Question Decision Tree

**When you have a question, ask:**

### Is this TECHNICAL or ARCHITECTURAL?

**TECHNICAL (AI decides using spec/docs):**
- How should feature X work? → **Check spec**
- What data structure to use? → **Check implementation guide**
- Should I use approach A or B? → **Check best practices, decide**
- Is dependency X ready? → **Verify using spec/docs**
- How to handle edge case Y? → **Check spec semantics**

**ARCHITECTURAL (ask user):**
- Should Atlas prioritize performance or memory? → **Ask user**
- Should we support feature X in v0.2 or defer to v0.3? → **Ask user**
- Which use case is more important? → **Ask user**
- Should we break backward compatibility for this? → **Ask user**
- What's the project priority right now? → **Ask user**

---

## Spec/Docs Verification Process

**When phase says "if missing, may need X":**

```
1. Identify what X is (e.g., "nested functions", "async runtime")

2. Search for X in specification:
   - Check `Atlas-SPEC.md` routing table
   - Read `docs/specification/syntax.md` if syntax feature
   - Read `docs/specification/types.md` if type system feature
   - Read `docs/specification/runtime.md` if runtime feature

3. Determine X status:
   a) Spec defines X completely
      → Implement per spec
      → Log: "Implemented X per specification/Y.md section Z"

   b) Spec mentions X but lacks detail
      → Check if it's v0.1 feature (implemented)
      → Check if it's planned for later version
      → If neither: Flag as potential spec gap

   c) Spec doesn't mention X at all
      → Check if out-of-scope (not in language design)
      → If blocker: Propose spec-compliant alternative
      → If critical: Ask ARCHITECTURAL question

4. Log decision in docs/reference/decision-log.md
```

---

## Example: Good vs Bad

### ❌ BAD (Asks user technical question):
```
Phase needs nested functions. Should we implement them?
User is architect, let them decide the approach.
```

### ✅ GOOD (Checks spec, decides autonomously):
```
Phase needs nested functions. Checking spec...

Verified: docs/specification/syntax.md section 4.2 defines
nested function syntax. Currently not implemented.

Decision: Implement nested functions per spec section 4.2.
Logging to decision-log.md.

Proceeding with implementation.
```

---

### ❌ BAD (Asks user if type system is ready):
```
Phase needs extern types. Is the type system ready for this?
```

### ✅ GOOD (Verifies using spec/docs):
```
Phase needs extern types. Checking type system...

Verified: docs/specification/types.md section 7.1 defines
extern type representation. Type system from v0.1 supports
all required primitives.

Decision: Type system ready. Proceeding with FFI phase.
```

---

## Decision Logging Format

**When making technical decision based on spec:**

```markdown
## [Date] - [Phase Name]

**Decision:** Implemented [feature] per specification

**Reasoning:**
- Phase [X] required [feature]
- Specification [file.md] section [N] defines [feature]
- Implementation follows spec exactly

**References:**
- Spec: docs/specification/[file].md section [N]
- Phase: phases/[category]/phase-[N].md
- Tests: [test file paths]

**Status:** Complete, [N] tests passing
```

---

## When Each Part Runs

### Part 1: Communication Check
**ALWAYS - Every interaction**

### Part 2: Spec-First Verification
**FOR ALL IMPLEMENTATION WORK:**
- Starting a phase ✓
- Implementing a feature ✓
- Adding functionality ✓
- Resolving a blocker ✓

**Process:**
1. Read phase blockers
2. Verify each dependency using spec/docs
3. Make technical decisions autonomously
4. Log decisions
5. ONLY ask architectural questions

### Part 3: Sanity Check
**AFTER spec verification, for work that changes code:**
- Implementation requests ✓
- Feature additions ✓
- Significant refactoring ✓

**SKIP for:**
- Pure questions
- Pure reading/exploration
- Documentation-only requests

---

## Philosophy

**AI = Lead Developer**
- Makes technical decisions using spec/docs as source of truth
- Implements features per specification
- Logs decisions for transparency
- Asks user ONLY for architectural direction

**User = Architect**
- Sets strategic direction
- Makes architectural trade-offs
- Defines priorities
- Reviews and approves major changes

**Spec = Law**
- Specification defines language behavior
- Implementation guides show how to build
- When spec conflicts with phase: Follow spec, flag conflict
- When spec is silent: Check if out-of-scope, propose alternatives

---

**Goal:** Autonomous technical decision-making using spec/docs, catch mistakes BEFORE hours of work, reserve user interaction for architecture only.

**Next:** GATE 0 (workflow classification)
