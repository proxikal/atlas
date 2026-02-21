# Typing Phase Restructure Plan

## Problem
Current structure separates implementation from tests, leading to test accumulation without functionality. We have 22 typing phases where 10+ are pure test phases.

## Solution
Merge tests into implementation phases. Each phase should deliver working functionality + comprehensive tests.

---

## Current Structure (22 phases)

### ✅ Completed (6 phases)
1. phase-01-binder.md - Binder implementation
2. phase-02-typechecker.md - Type checker core
3. phase-05-type-rules-tests.md - **TESTS ONLY**
4. phase-06-scope-shadowing-tests.md - **TESTS ONLY**
5. phase-07-nullability-rules.md - Nullability implementation + tests
6. phase-10-function-return-analysis.md - Return analysis implementation + tests

### ⬜ Remaining (16 phases)
7. phase-11-typecheck-dump-versioning.md - Dump versioning
8. phase-12-control-flow-legality.md - Control flow rules
9. phase-13-related-spans.md - Related span support
10. phase-14-warnings.md - Warning implementation
11. phase-15-warning-tests.md - **TESTS ONLY**
12. phase-16-top-level-order-tests.md - **TESTS ONLY**
13. phase-17-operator-rule-tests.md - **TESTS ONLY**
14. phase-18-string-semantics-tests.md - **TESTS ONLY**
15. phase-19-related-span-coverage.md - **TESTS ONLY**
16. phase-20-diagnostic-normalization-tests.md - **TESTS ONLY**
17. phase-21-numeric-edge-tests.md - **TESTS ONLY**
18. phase-22-diagnostic-ordering-tests.md - **TESTS ONLY**

**8 out of 16 remaining phases are pure tests with no implementation.**

---

## New Structure (9 phases)

### Phase 1: Binder (✅ COMPLETE)
- **File:** `phase-01-binder.md`
- **Scope:** Symbol binding + scope resolution
- **Tests:** Basic binding tests included

### Phase 2: Type Checker Core (✅ COMPLETE)
- **File:** `phase-02-typechecker.md`
- **Scope:** Type inference, operator typing, type rules
- **Tests:** Core type checking tests
- **Absorbs:** phase-05-type-rules-tests.md, phase-17-operator-rule-tests.md

### Phase 3: Scopes & Shadowing (✅ COMPLETE)
- **File:** `phase-03-scopes-shadowing.md` (rename phase-06)
- **Scope:** Scope rules, shadowing, top-level order
- **Tests:** Scope tests, shadowing tests, top-level order tests
- **Absorbs:** phase-06-scope-shadowing-tests.md, phase-16-top-level-order-tests.md

### Phase 4: Nullability (✅ COMPLETE)
- **File:** `phase-04-nullability.md` (rename phase-07)
- **Scope:** Nullable types, null checking
- **Tests:** Nullability edge cases included

### Phase 5: Function Returns & Control Flow (✅ COMPLETE)
- **File:** `phase-05-function-returns.md` (merge phase-10 + phase-12)
- **Scope:** Return analysis, break/continue/return legality
- **Tests:** Return path tests, control flow tests
- **Absorbs:** phase-10-function-return-analysis.md, phase-12-control-flow-legality.md

### Phase 6: Warnings (⬜ NEXT)
- **File:** `phase-06-warnings.md` (merge phase-14 + phase-15)
- **Scope:** Unused variables, unreachable code warnings
- **Tests:** Warning behavior, ordering, suppression
- **Absorbs:** phase-14-warnings.md, phase-15-warning-tests.md

### Phase 7: Diagnostics & Related Spans (⬜ TODO)
- **File:** `phase-07-diagnostics.md` (merge phase-13 + phase-19 + phase-20)
- **Scope:** Related spans, diagnostic normalization, ordering
- **Tests:** Related span coverage, normalization tests, ordering tests
- **Absorbs:** phase-13-related-spans.md, phase-19-related-span-coverage.md, phase-20-diagnostic-normalization-tests.md

### Phase 8: Semantic Edge Cases (⬜ TODO)
- **File:** `phase-08-semantic-edge-cases.md` (merge phase-18 + phase-21)
- **Scope:** String semantics, numeric edge cases
- **Tests:** String operation tests, numeric boundary tests
- **Absorbs:** phase-18-string-semantics-tests.md, phase-21-numeric-edge-tests.md

### Phase 9: Type System Stability (⬜ TODO)
- **File:** `phase-09-typecheck-stability.md` (rename phase-11 + phase-22)
- **Scope:** Typecheck dump versioning, diagnostic ordering guarantee
- **Tests:** Version field tests, diagnostic stability tests
- **Absorbs:** phase-11-typecheck-dump-versioning.md, phase-22-diagnostic-ordering-tests.md

---

## Migration Steps

### Step 1: Archive Old Test-Only Phases
Move these to `phases/typing/archive/pre-restructure/`:
- phase-05-type-rules-tests.md (absorbed into phase-02)
- phase-15-warning-tests.md (absorbed into phase-14)
- phase-16-top-level-order-tests.md (absorbed into phase-06)
- phase-17-operator-rule-tests.md (absorbed into phase-02)
- phase-18-string-semantics-tests.md (absorbed into phase-08)
- phase-19-related-span-coverage.md (absorbed into phase-13)
- phase-20-diagnostic-normalization-tests.md (absorbed into phase-07)
- phase-21-numeric-edge-tests.md (absorbed into phase-08)
- phase-22-diagnostic-ordering-tests.md (absorbed into phase-09)

### Step 2: Rename & Merge Completed Phases
Since phases 1-6 are complete, we need to:
1. Mark phase-05 and phase-06 as absorbed (already done, tests exist)
2. Update phase-07 and phase-10 descriptions to note they include tests
3. Leave them as-is since work is complete

### Step 3: Create New Combined Phase Files
For uncompleted phases (6-9 in new structure):
1. Create new phase files with merged requirements
2. Each phase must specify BOTH implementation AND test deliverables
3. Exit criteria must include "implementation complete + tests passing"

### Step 4: Update BUILD-ORDER.md
Replace Typing & Binding section with new 9-phase structure.

### Step 5: Update STATUS.md
- Mark absorbed test phases as ✅ (completed via parent phase)
- Update progress tracker to show 9 phases instead of 22
- Update current phase pointer

---

## Benefits

1. **Clear implementation focus** - Each phase delivers working features
2. **Integrated testing** - Tests validate implementation immediately
3. **Reduced phase count** - 22 → 9 phases (59% reduction)
4. **Better agent guidance** - No ambiguity about what to build
5. **Realistic progress** - 6/9 complete (67%) vs 6/22 (27%)

---

## Next Actions

1. ✅ Create this plan
2. ⬜ Archive test-only phases
3. ⬜ Create merged phase files (phase-06 through phase-09)
4. ⬜ Update BUILD-ORDER.md
5. ⬜ Update STATUS.md
6. ⬜ Continue with phase-06-warnings.md (implementation + tests)
