# Atlas Architectural Decisions

**Purpose:** Record of key architectural decisions for AI agents. Use DR-XXX to reference.

---

## Active Decisions

| DR | Title | Date | Summary |
|----|-------|------|---------|
| 001 | Dual execution engines | 2024 | Interpreter (dev/debug) + VM (production). 100% parity required. |
| 003 | Hash function design | 2025-02 | `DefaultHasher` with value-based hashing. Hashable: String, Number, Bool. |
| 004 | HashMap key equality | 2025-02 | Value equality (structural), not reference. Hash-then-compare. |
| 005 | Collection iteration API | 2025-02 | Functional: `forEach`/`map`/`filter` as intrinsics. HashMap cb: `fn(value, key)`. |
| 007 | Phase file accuracy | 2025-02-16 | All phase files must reference actual files. Gate -1 verifies. |
| 008 | Phase scope sizing | 2025-02-16 | ~200-300 lines impl, 10-20 tests per phase. Split if larger. |
| 009 | Arc<Mutex<T>> migration | 2026-01 | Replaced `Rc<RefCell<T>>` (DR-002). Required for tokio/async. Access: `.lock().unwrap()`. |
| 010 | Type alias resolution | 2026-02 | Structural resolution, alias names preserved for display. Circular = rejected. |
| 011 | Built-in constraints | 2026-02 | `Comparable`/`Numeric` → number. `Equatable` → number\|string\|bool\|null. No trait system. |
| 012 | Structural width subtyping | 2026-02-17 | Extra fields OK for assignability. Missing required fields = fail. |
| 013 | ModuleExecutor borrows interpreter | 2026-02-18 | ModuleExecutor takes `&mut Interpreter` to ensure imports populate caller's interpreter. |

## Superseded

| DR | Title | Replaced by |
|----|-------|-------------|
| 002 | `Rc<RefCell<T>>` collections | DR-009 (`Arc<Mutex<T>>`) |
| 006 | Collection benchmarking (deferred) | Infra phase-07 (benchmark suite) |

---

## Detail: DR-003 — Hash Function

```rust
pub struct HashKey { hash: u64, value: Value }
// compute_hash: String → s.hash(), Number → n.to_bits().hash(), Bool → b.hash()
// Unhashable: Null, Array, HashMap, HashSet, Function, Object → RuntimeError
```

## Detail: DR-004 — Key Equality

```rust
impl PartialEq for HashKey {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && values_equal(&self.value, &other.value) // hash-first fast path
    }
}
```

## Detail: DR-005 — Collection Iteration

```atlas
hashMapForEach(map, fn(value, key) { ... });     // → null
hashMapMap(map, fn(value, key) { value * 2; });  // → new HashMap
hashMapFilter(map, fn(value, key) { v > 10; });  // → new HashMap
hashSetForEach(set, fn(elem) { ... });            // → null
hashSetMap(set, fn(elem) { elem * 2; });          // → Array (NOT Set — allows dupes)
hashSetFilter(set, fn(elem) { elem > 10; });      // → new HashSet
```

## Detail: DR-009 — Arc Migration

```rust
// Old (DR-002): Array(Rc<RefCell<Vec<Value>>>)  — !Send, breaks tokio
// New (DR-009): Array(Arc<Mutex<Vec<Value>>>)   — Send + Sync
// Access: .lock().unwrap()   NOT .borrow()/.borrow_mut()
```

## Detail: DR-011 — Built-in Constraints

| Constraint | Resolves to |
|-----------|-------------|
| `Comparable`, `Numeric` | `number` |
| `Equatable` | `number \| string \| bool \| null` |
| `Serializable` | `number \| string \| bool \| null \| json` |
| `Iterable` | `Array<unknown>` |

---

## New Decision Template

```markdown
## DR-XXX: Title

**Date:** YYYY-MM-DD | **Status:** ✅ Active | ⏳ Deferred | 🔄 Superseded

**Decision:** [What]
**Rationale:** [Why]
**Alternatives:** ❌ Alt1: [why rejected]
**Impact:** [What it affects]
```

---

## Detail: DR-013 — ModuleExecutor Borrows Interpreter

**Date:** 2026-02-18 | **Status:** ✅ Active

**Decision:** Refactor `ModuleExecutor` to borrow `&mut Interpreter` instead of owning one.

**Rationale:**
- `ModuleExecutor` previously owned its own `Interpreter` instance
- Imports processed by ModuleExecutor populated ITS interpreter, not Runtime's
- This caused imports to silently fail when using `Runtime.eval()`
- Borrowing ensures imports populate the SAME interpreter that callers use

**Alternatives:**
❌ Merge interpreters after execution: Complex, error-prone, state sync issues
❌ Share via Arc<Mutex<Interpreter>>: Overkill, adds locking overhead

**Impact:**
- `ModuleExecutor::new()` signature changes to accept `&mut Interpreter`
- `Runtime` must pass its interpreter to ModuleExecutor for file execution
- Enables proper import execution for both Runtime and standalone use
