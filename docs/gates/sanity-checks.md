# Atlas Sanity Checks

**GATE -1: Pre-work discussion to catch user mistakes BEFORE work starts**

**Philosophy:** Discussion-oriented, not blocking. Help user understand implications.

---

## 7 Check Categories

### 1. Version Scope Check
**Is request in current version's phases?**
- Search phase files in `phases/*/`
- Check `STATUS.md` for current version
- Check `Atlas-SPEC.md` for "under research" features

**If not in scope:** Open discussion about v0.3, research, or brainstorming.

---

### 2. Dependency Check
**Are prerequisites met?**
- Read phase file BLOCKERS section
- Check `STATUS.md` for what's implemented
- Verify required components exist

**If missing:** Discuss implementing dependencies first.

---

### 3. Design Consistency Check
**Does request align with Atlas identity?**
- Read `docs/gates/compiler-principles.md`
- Check if conflicts with "What Atlas IS"
- Check if matches "Anti-Patterns"

**If conflicts:** Discuss why it breaks core principles.

---

### 4. Workload Check
**Is this too ambitious for one session?**
- Estimate scope
- Compare to typical phase size
- Check if multiple phases

**If too large:** Discuss starting with first phase or breaking down.

---

### 5. Priority Check
**Is there more important work pending?**
- Check `STATUS.md` current phase
- Evaluate if request is higher priority

**If wrong priority:** Discuss completing current work first.

---

### 6. Approach Check
**Does approach align with compiler practices?**
- Check if strict TDD requested for features (implementation-driven is correct)
- Check if oversimplifying for line counts (quality over metrics)
- Compare to real compilers (rustc, Go, Clang)

**If conflicts:** Discuss correct compiler approach.

---

### 7. Consistency Check
**Will this break other components?**
- Check if breaks parity
- Check if creates inconsistency

**If breaks consistency:** Discuss maintaining parity/consistency.

---

## Discussion Format

```
🤔 SANITY CHECK DISCUSSION

Request: [user request]

Potential concern: [what might be problematic]

Current context:
- Version: [from STATUS.md]
- Progress: [from STATUS.md]

Discussion points:
1. [Concern with reasoning]
2. [Alternative if applicable]

Questions:
- [Clarifying question]

This might be fine if [conditions]. Want to discuss?
```

**NOT a blocker** - User decides after discussion.

---

**Reference:** `docs/gates/compiler-principles.md`, STATUS.md, Atlas-SPEC.md, phase files
