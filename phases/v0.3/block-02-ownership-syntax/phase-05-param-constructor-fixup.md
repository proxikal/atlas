# Phase 05: Param Constructor Fixup (Blast Radius Repair)

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 02 (Param struct changed)
**Complexity:** low (mechanical, not architectural)
**Files to modify:**
- `crates/atlas-runtime/src/parser/mod.rs` (line 145)
- `crates/atlas-runtime/tests/frontend_integration.rs` (lines 1635, 1643)
- Any other file that constructs `Param { ... }` directly (verify with grep at execution time)

## Summary

Phase 02 adds `ownership: Option<OwnershipAnnotation>` to `Param`, which breaks every
construction site. This phase makes the build green again by adding `ownership: None` to
all existing construction sites. It is purely mechanical — no logic changes.

## Current State

After Phase 02, `cargo build -p atlas-runtime` will fail with:
```
missing field `ownership` in initializer of `Param`
```
at a small number of sites (verified: 2 in production, 2 in tests).

## Requirements

1. Run `cargo build -p atlas-runtime 2>&1 | grep "missing field"` to find ALL sites.
2. Add `ownership: None` to every existing `Param { ... }` construction site.
3. Phase 03 will later replace the `None` with parsed values where appropriate —
   this phase only adds the field with `None`.
4. Update test constructions in `frontend_integration.rs` similarly.

## Acceptance Criteria

- [ ] `cargo build --workspace` passes with zero errors
- [ ] No `Param { name, type_ref, span }` construction sites remain (all have `ownership`)
- [ ] All existing tests pass (behavior identical to pre-Phase-02 — `ownership` is `None`
      everywhere, which is the unannotated default)
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

No new tests needed. This phase exists only to restore compilation. The passing test suite
is the acceptance gate.

## Notes

- Execute this phase immediately after Phase 02, before any other phases that depend on
  `Param` compiling. Phases 03 and 06 both require a green build.
- This is the recommended execution order: 01 → 02 → 05 → 03 → 04 → 06 → ...
