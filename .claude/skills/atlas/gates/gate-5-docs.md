# GATE 5: Doc Update (Selective - 3-Tier Strategy)

**Condition:** Quality gates passed

**CRITICAL:** Use 3-tier strategy. Don't update 15 files every phase (wallet dies). Don't wait until polish (drift catastrophic).

**Reference:** `docs/gates/doc-update-rules.md` (detailed rules)

---

## Tier 1: IMMEDIATE Updates (Do NOW)

**Update immediately if ANY of these:**

1. **Spec changed:**
   - Updated any file in `docs/specification/` → Commit now
   - Grammar, types, semantics, runtime behavior changed
   - Note: Atlas-SPEC.md is just the index (rarely changes)

2. **Architectural decision made:**
   - Add to `docs/reference/decision-log.md` → Commit now
   - Only if: Chose between approaches AND affects future work

3. **Breaking API change:**
   - Update `docs/api/` → Commit now
   - Changed function signature, removed function, changed behavior

**Cost:** ~$0.10-0.25 per phase (if critical changes exist)

---

## Tier 2: BATCHED Updates (Queue for Later)

**Queue in `docs/reference/pending-updates.md` if:**

1. **New features added:**
   - Added stdlib functions
   - Added language features
   - Minor spec clarifications

2. **When to process:**
   - Category complete (all stdlib/strings phases done)
   - OR every 10-20 phases
   - OR at mini-polish checkpoints

**Cost:** ~$0.20-0.50 every 10-20 phases

---

## Tier 3: NEVER Update (Skip)

**DON'T update docs for:**

1. **Implementation details:**
   - Refactored code
   - Optimizations
   - Internal changes

2. **Bug fixes:**
   - Fixed errors
   - Corrected behavior
   - Unless spec ambiguity revealed

3. **Refactors:**
   - Code cleanup
   - Module splits
   - Unless public API changed

**Cost:** $0 (don't do it)

---

## Decision Tree

```
Spec changed? → YES → Update relevant docs/specification/*.md NOW
              → NO ↓

Architectural decision? → YES → Add to decision-log.md NOW
                       → NO ↓

Breaking API change? → YES → Update docs/api/ NOW
                    → NO ↓

New features? → YES → Queue in .pending-updates.md
              → NO ↓

Implementation detail only? → YES → SKIP
                            → NO → Done
```

---

## Token Savings

**At 100 phases:**
- Old way: Update 15 files × 100 phases = $150
- New way: ~20 immediate + 5 batches = $5
- **Savings: $145**

---

**Next:** Done (or GATE 6 if structured development)

**Reference:** `docs/gates/doc-update-rules.md`
