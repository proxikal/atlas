# Foundation & Approach Health Checks

**GATE 1.5: Pre-implementation verification (BEFORE writing code)**

**Purpose:** Prevent wasting tokens on bad foundations. Check BEFORE implementing.

---

## 5-Point Checklist

### ✅ 1. Existing Code Audit
**If building on existing code:**
- [ ] Read existing code first
- [ ] Check against Atlas-SPEC.md (grammar, types, semantics)
- [ ] Check against `docs/gates/compiler-principles.md` (no stubs, parity, quality)
- [ ] Look for: implicit types, stubs, TODOs, parity breaks

**If violations found:** Fix existing code FIRST, don't build on broken foundation.

---

### ✅ 2. Dependency Verification
**Check prerequisites:**
- [ ] Required components exist?
- [ ] Infrastructure ready?
- [ ] Both engines support prerequisites?
- [ ] Phase BLOCKERS satisfied?
- [ ] **Dependencies not stubbed?** → Check `docs/reference/intentional-stubs.md`

**If missing:** Implement dependencies FIRST, then feature.
**If stubbed:** Can't build on stub - report to user with options.

---

### ✅ 3. Architectural Decision Check
**Search `docs/decision-logs/` (by component):**
- [ ] Any documented decisions apply?
- [ ] Planned approach aligns?
- [ ] If making new decision: document it (see template in README)

**If conflict:** Follow documented decision or discuss with user.

---

### ✅ 4. Real Compiler Comparison
**How do production compilers handle this?**
- [ ] rustc approach?
- [ ] Go compiler approach?
- [ ] TypeScript/Clang approach?
- [ ] Our approach justified?

**If different from all:** Justify difference or use proven approach.

---

### ✅ 5. Anti-Pattern Detection
**Check plan for red flags:**
- [ ] Planning stubs? (BANNED - implement fully or split phase)
- [ ] Planning single-engine? (Breaks parity - both engines always)
- [ ] Planning to oversimplify for line counts? (Quality over metrics)
- [ ] Planning to skip tests? (Required, not optional)
- [ ] Assuming existing code is correct? (Verify first)

**If any detected:** Fix plan before implementing.

---

## Output Format

**If issues found:**
```
⚠️ GATE 1.5: ISSUES DETECTED

About to: [feature]

Issues:
1. [Issue type]: [specific problem]
2. [Evidence from specs/docs]

Action: Fix BEFORE implementation
```

**If all good:**
```
✅ GATE 1.5: VERIFIED
Foundation solid. Approach sound.
Proceeding to GATE 2.
```

---

## Cost Savings

**Without GATE 1.5:** Write 500 lines → discover bad foundation → rewrite = $15 wasted

**With GATE 1.5:** Check first (0 lines) → fix foundation → implement correctly = $0 wasted

---

**Reference:** `docs/gates/compiler-principles.md`, `docs/decision-logs/`, Atlas-SPEC.md
