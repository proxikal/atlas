# BLOCKER 04-B: Module Loading & Caching

**Part:** 2 of 4 (Loading & Caching)
**Category:** Foundation - Language Feature
**Estimated Effort:** 1 week
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** BLOCKER 04-A complete.

**Verification:**
```bash
cargo test module_resolution_tests --no-fail-fast
grep -n "ImportDecl\|ExportDecl" crates/atlas-runtime/src/ast.rs
```

**What's needed:**
- ✅ Import/export parsing works
- ✅ Resolution algorithm works

**If missing:** Complete BLOCKER 04-A first.

---

## Objective

**THIS PHASE:** Implement module loader, dependency graph, topological sort, and caching. Modules parsed but not yet type checked.

**Success criteria:** Load modules in correct order. Cache prevents reloading.

---

## Implementation

### Step 1-3: Loader (Days 1-5)

Load and parse module files. Build dependency graph. Topological sort for initialization order.

### Step 4: Caching (Days 6-7)

Module cache by absolute path. Avoid reloading/re-executing.

---

## Acceptance Criteria

- ✅ Modules load in order
- ✅ Caching works
- ✅ Dependency graph correct
- ✅ 30+ tests pass

**Next:** blocker-04-c-type-system-integration.md
