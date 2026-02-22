# Phase 16: Spec Update + Block 2 Acceptance Criteria Check

**Block:** 2 (Ownership Syntax)
**Depends on:** All previous phases complete
**Complexity:** low
**Files to modify:**
- `docs/specification/memory-model.md`
- `STATUS.md`
- `docs/internal/V03_PLAN.md` (check off Block 2 AC)
- Auto-memory `decisions/runtime.md` (log DR-B02-* decisions)

## Summary

Final gate for Block 2. Update the spec to reflect the v0.3 implementation. Verify all 5
Block 2 acceptance criteria from V03_PLAN.md. Update STATUS.md. Log decisions in auto-memory.

## Current State

After Phase 15, the implementation is complete. The spec (`memory-model.md`) has an
"Implementation Notes (v0.3 — Block 1 complete)" section. It needs a Block 2 section.
STATUS.md shows Block 2 as ⬜ Unblocked — needs to become ✅ Complete.

## Requirements

### Spec Update — `docs/specification/memory-model.md`

Add a new section after the Block 1 notes:

```markdown
## Implementation Notes (v0.3 — Block 2 complete)

### Ownership Annotation Tokens
`own`, `borrow`, `shared` are reserved keywords (added in Block 2).
They are invalid as identifiers in Atlas v0.3+.

### AST Representation
- `Param.ownership: Option<OwnershipAnnotation>` — `None` = unannotated
- `FunctionDecl.return_ownership: Option<OwnershipAnnotation>`
- `OwnershipAnnotation` enum: `Own | Borrow | Shared`

### Runtime Enforcement (v0.3 — debug assertions)
- `own` param: caller binding marked consumed; reuse → runtime error (debug mode)
- `shared` param: argument must be `Value::SharedValue(_)` (debug mode assertion)
- `borrow` param: no runtime enforcement — value semantics + CoW provide the guarantee
- Both engines (interpreter + VM) enforce identically

### Compile-Time Enforcement (v0.4)
v0.4 adds a static dataflow pass over the typed AST. No syntax changes required —
the annotation system is already complete. v0.4 only adds the verification layer.

### Diagnostic Codes
- `AT_OWN_ON_PRIMITIVE` (warning) — `own` annotation on primitive type
- `AT_BORROW_ON_SHARED` (warning) — `borrow` annotation on `shared<T>` type
- `AT_BORROW_TO_OWN` (warning) — passing borrowed value to `own` parameter
- `AT_NON_SHARED_TO_SHARED` (error) — non-`shared<T>` value to `shared` parameter
```

### Block 2 Acceptance Criteria (from V03_PLAN.md)

Verify ALL are met:
- [ ] All three annotations parse correctly
- [ ] Type checker rejects mismatched ownership (passing `borrow` where `own` required)
- [ ] Runtime assertion fires when ownership is violated (debug mode)
- [ ] Both engines enforce ownership consistently
- [ ] LSP shows ownership annotations in hover info

### STATUS.md Update
- Block 2 row: ⬜ → ✅ Complete (date)
- Update "Last Completed" line
- Update "Next" line to: Scaffold Block 3 (Trait System)
- Add Block 2 metrics section

### Auto-Memory Update (GATE 7)
Log in `decisions/runtime.md`:
```
## DR-B02-01: Ownership Annotation Implementation
- OwnershipAnnotation enum: Own | Borrow | Shared
- Param.ownership: Option<OwnershipAnnotation> (None = unannotated)
- Runtime enforcement: debug_assertions only, zero release overhead
- own: binding consumed, reuse → runtime error
- borrow: no runtime enforcement (CoW provides semantics)
- shared: Value must be SharedValue(_), enforced at call time
- v0.4 adds static dataflow pass — no syntax changes needed
```

## Acceptance Criteria

- [ ] All 5 Block 2 AC from V03_PLAN.md verified ✅
- [ ] `memory-model.md` has Block 2 implementation notes
- [ ] STATUS.md reflects Block 2 complete
- [ ] `decisions/runtime.md` updated with DR-B02-01
- [ ] Full test suite: `cargo nextest run -p atlas-runtime` 100% passing
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] `cargo fmt --check --all` clean
- [ ] Block 2 merged to local `main`, worktree branches rebased

## Notes

- This phase has no code changes. It is documentation and verification only.
- Do not begin Block 3 scaffolding until this phase is complete and merged.
- The "planned vs. actual" section of V03_PLAN.md should note any discoveries
  during Block 2 execution (e.g., if the blast radius was larger than expected,
  or if any phases merged/split during execution).
