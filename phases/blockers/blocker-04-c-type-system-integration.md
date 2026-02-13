# BLOCKER 04-C: Module Type System Integration

**Part:** 3 of 4 (Type System Integration)
**Category:** Foundation - Language Feature
**Estimated Effort:** 1 week
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** BLOCKER 04-B complete.

**Verification:**
```bash
cargo test module_loading_tests --no-fail-fast
grep -n "ModuleLoader" crates/atlas-runtime/src/modules/loader.rs
```

**What's needed:**
- ✅ Module loading works
- ✅ Caching works
- ✅ Dependency graph correct

**If missing:** Complete BLOCKER 04-B first.

---

## Objective

**THIS PHASE:** Extend binder and type checker for cross-module types. Imports create bindings. Exports mark symbols visible.

**Success criteria:** Type checking works across module boundaries.

---

## Implementation

### Step 1-2: Binder Integration (Days 1-4)

Track module boundaries. Imports create bindings pointing to exports. Symbol table per-module.

### Step 3: Type Checker Integration (Days 5-7)

Type check imports match exports. Check no duplicate exports. Cross-module type resolution.

---

## Acceptance Criteria

- ✅ Cross-module binding works
- ✅ Type checking validates imports
- ✅ 30+ tests pass

**Next:** blocker-04-d-runtime-implementation.md
