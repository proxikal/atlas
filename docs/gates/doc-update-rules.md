# Documentation Update Rules

**GATE 5: 3-tier strategy (immediate, batched, never)**

**Problem:** Can't update 15 files every phase (wallet dies). Can't wait until end (drift catastrophic).

**Solution:** Selective, intelligent updates.

---

## Tier 1: IMMEDIATE (Do NOW)

**Update immediately if:**

1. **Spec changed** → Update `Atlas-SPEC.md` now
2. **Architectural decision made** → Add to `docs/decision-logs/[component]/` now (see template)
3. **Breaking API change** → Update `docs/api/` now

**Cost:** ~$0.15/phase (if needed)

---

## Tier 2: BATCHED (Queue for Later)

**Queue in `docs/reference/pending-updates.md` if:**

1. **New features added** (stdlib functions, language features)
2. **Minor spec clarifications**

**Process batches when:**
- Category complete (all stdlib/strings phases done)
- OR every 10-20 phases
- OR at checkpoints

**Cost:** ~$0.40 per batch (every 10-20 phases)

---

## Tier 3: NEVER (Skip)

**DON'T update for:**

1. **Implementation details** (refactors, optimizations, internal changes)
2. **Bug fixes** (unless spec ambiguity revealed)
3. **Refactors** (unless public API changed)

**Cost:** $0 (don't do it)

---

## Decision Tree

```
Spec changed? → YES → Tier 1 (update now)
              → NO ↓

Architectural decision? → YES → Tier 1 (add to decision-log)
                       → NO ↓

Breaking API change? → YES → Tier 1 (update API docs)
                    → NO ↓

New features? → YES → Tier 2 (queue in .pending-updates.md)
              → NO ↓

Implementation only? → YES → Tier 3 (skip)
```

---

## Decision-Log Rules

**Only add if ALL of these:**
- [ ] Chose between multiple approaches
- [ ] Decision affects future work
- [ ] Someone would wonder "why this way?"

**Examples:**
- ✅ "Use panic mode error recovery (not error productions)"
- ✅ "String interning: use string pool"
- ❌ "Used Vec instead of HashMap" (implementation detail)
- ❌ "Fixed off-by-one error" (bug fix)

---

## Cost at 100 Phases

**Old way:** 15 files × 100 phases = $150
**New way:** ~20 immediate + 5 batches = $5
**Savings:** $145

---

**Reference:** .pending-updates.md for queued updates
