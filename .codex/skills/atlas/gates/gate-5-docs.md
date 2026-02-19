# GATE 5: Doc Update (Selective)

**Condition:** Quality gates passed

**CRITICAL:** Only update docs that matter. Don't update everything every phase.

---

## Tier 1: IMMEDIATE Updates (Do NOW)

**Update immediately if ANY of these:**

1. **Spec changed:**
   - Updated any file in `docs/specification/` → Commit now
   - Grammar, types, semantics, runtime behavior changed

2. **Architectural decision made:**
   - Add to `memory/decisions.md` → Commit now
   - Only if: Chose between approaches AND affects future work

3. **Breaking API change:**
   - Update `docs/specification/stdlib.md` → Commit now
   - Changed function signature, removed function, changed behavior

---

## Tier 2: BATCHED Updates (Queue for Later)

**Update when a category completes (e.g., all frontend phases done):**
- Feature status docs (e.g., `docs/frontend-status.md`)
- Phase-generated docs (e.g., `docs/source-maps.md`)

---

## Tier 3: NEVER Update (Skip)

**DON'T update docs for:**
- Internal refactors
- Bug fixes (unless spec ambiguity revealed)
- Code cleanup

---

## Decision Tree

```
Spec changed? → YES → Update docs/specification/*.md NOW
              → NO ↓

Architectural decision? → YES → Add to memory/decisions.md NOW
                       → NO ↓

Breaking API change? → YES → Update docs/specification/stdlib.md NOW
                    → NO ↓

New feature with docs? → YES → Create/update feature doc in docs/
                       → NO → SKIP
```

---

**Next:** GATE 6 (if structured development), otherwise Done
