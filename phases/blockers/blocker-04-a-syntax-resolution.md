# BLOCKER 04-A: Module Syntax & Resolution

**Part:** 1 of 4 (Syntax & Resolution)
**Category:** Foundation - Language Feature
**Estimated Effort:** 1 week
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Core language stable, no major changes planned.

**Verification:**
```bash
cargo test --all --no-fail-fast
grep -c "test result: ok" target/test-results.txt
```

**What's needed:**
- v0.1 complete and stable
- Parser, binder, type checker working well
- All existing tests passing

**If missing:** Stabilize core first.

---

## Objective

**THIS PHASE:** Add import/export syntax, AST, parser, and module resolution algorithm. Does NOT load/execute modules yet.

**Success criteria:** Can parse import/export. Module resolution finds correct files.

---

## Implementation

### Step 1-2: Syntax & AST (Days 1-3)

Add ImportDecl and ExportDecl to AST. Parse ES-module style syntax (`import { x } from "./mod"`).

### Step 3-4: Resolution Algorithm (Days 4-7)

Implement path resolution (relative, absolute). Circular dependency detection. Module cache structure.

---

## Acceptance Criteria

- ✅ Parse import/export syntax
- ✅ Resolution finds modules
- ✅ Circular detection works
- ✅ 40+ tests pass

**Next:** blocker-04-b-loading-caching.md
