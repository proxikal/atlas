# GATE 1.5: Foundation & Approach Health Check

**Condition:** Size estimated, plan ready

**Action:** Verify foundation and approach BEFORE writing any code

---

## Why This Gate Exists

**CRITICAL:** This gate prevents wasting tokens. Check existing code and approach BEFORE implementation, not after.

- ❌ Without GATE 1.5: Write 500 lines → discover bad foundation → delete all → rewrite = $15 wasted
- ✅ With GATE 1.5: Check foundation first (0 lines written) → fix issues → implement correctly = $0 wasted

**Cost savings:** Every issue caught here = $10-20 saved

---

## 5-Point Checklist

### 1. Existing Code Audit

**If building on or modifying existing code:**
- Read existing code FIRST
- Check against relevant specs (use `Atlas-SPEC.md` routing to find which)
- Check against skill quality rules (no stubs, parity, quality)
- Look for violations: implicit types, stubs, TODOs, parity breaks
- **Fix violations BEFORE adding new code**

---

### 2. Dependency Verification

**Check prerequisites exist:**
- Required components implemented?
- Infrastructure ready?
- Both engines support prerequisites?
- Phase BLOCKERS satisfied?
- **Dependencies not stubbed?** → Verify in codebase (grep for `unimplemented!`, `todo!`)

**If dependency is stubbed:**
- Cannot use stubbed component
- Report to user with options

---

### 3. Architectural Decision Check

**Read `memory/decisions.md`:**
- Any documented decisions apply?
- Does planned approach align?
- If making new decision: document it

---

### 4. Real Compiler Comparison

**How do production compilers handle this?**
- rustc approach?
- Go compiler approach?
- TypeScript/Clang approach?
- Is our approach justified?

---

### 5. Anti-Pattern Detection

**Check your plan for red flags:**
- [ ] Planning stubs? (BANNED)
- [ ] Planning single-engine? (Breaks parity)
- [ ] Planning to oversimplify for line counts? (Quality over metrics)
- [ ] Planning to skip tests? (Required, not optional)
- [ ] Assuming existing code is correct? (Verify first)

---

## If Issues Found

```
⚠️ GATE 1.5: FOUNDATION/APPROACH ISSUES

About to implement: [feature]

Issues:
1. [Issue with detail]
2. [Issue with detail]

ACTION: Fix BEFORE implementation
1. [Fix step]
2. [Fix step]
3. Re-run GATE 1.5
4. THEN implement [feature]

Should I address these first?
```

---

## If All Checks Pass

```
✅ GATE 1.5: VERIFIED

Foundation solid. Approach sound.

Proceeding to GATE 2: Implementation
```

---

**BLOCKING:** If issues found, MUST fix BEFORE GATE 2. Don't waste tokens writing code on bad foundations.

**Reference:** `memory/gates.md` (quality gate rules)

**Next:** GATE 2 (only if GATE 1.5 passes)
