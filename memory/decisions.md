# Atlas Architectural Decisions

**Purpose:** Record of key architectural decisions for AI agents.

---

## DR-001: Interpreter + VM Dual Execution

**Date:** 2024 (v0.1)
**Status:** ✅ Active

**Decision:** Atlas maintains TWO execution engines with 100% feature parity.

**Engines:**
1. **Interpreter:** AST-walking interpreter (developer-friendly, debugging)
2. **VM:** Bytecode virtual machine (production, performance)

**Requirements:**
- Every feature MUST work in both engines
- Outputs MUST be identical
- Tests run against both engines

**Rationale:**
- Interpreter: Fast development, easy debugging, REPL
- VM: Performance, optimization, production deployments

**Impact:**
- Every intrinsic implemented twice (interpreter + VM)
- Parity testing required for all features
- Increased maintenance but better DX and performance

---

## DR-002: Reference Semantics for Collections

**Date:** 2025-02 (v0.2, Phase 07a)
**Status:** 🔄 Superseded by DR-009 (phase-18 Arc migration)

**Decision:** Collections use `Rc<RefCell<T>>` for reference semantics.
**NOTE:** This was replaced with `Arc<Mutex<T>>` in phase-18. See DR-009.

**Pattern:**
```rust
pub enum Value {
    Array(Rc<RefCell<Vec<Value>>>),
    HashMap(Rc<RefCell<HashMap>>),
    HashSet(Rc<RefCell<HashSet>>),
    Queue(Rc<RefCell<Queue>>),
    Stack(Rc<RefCell<Stack>>),
}
```

**Rationale:**
- Multiple bindings to same collection (JavaScript-like behavior)
- Mutations visible across all references
- Enables shared state patterns

**Example:**
```atlas
let map1 = hashMapNew();
let map2 = map1;  // Same reference
hashMapPut(map1, "key", 100);
print(hashMapGet(map2, "key"));  // Prints 100
```

**Alternatives Considered:**
- ❌ Copy-on-write: Confusing semantics, unexpected copies
- ❌ Value semantics: Breaking change for arrays, inconsistent

**Impact:**
- All collection operations use `.borrow()` and `.borrow_mut()`
- Careful management to avoid RefCell panics (borrow conflicts)
- Memory managed via Rc (reference counting)

---

## DR-003: Hash Function Design

**Date:** 2025-02 (v0.2, Phase 07a)
**Status:** ✅ Active

**Decision:** Use Rust's `DefaultHasher` with value-based hashing.

**Implementation:**
```rust
pub struct HashKey {
    hash: u64,
    value: Value,
}

impl HashKey {
    pub fn new(value: Value) -> Result<Self, RuntimeError> {
        let hash = Self::compute_hash(&value)?;
        Ok(HashKey { hash, value })
    }

    fn compute_hash(value: &Value) -> Result<u64, RuntimeError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        match value {
            Value::String(s) => s.hash(&mut hasher),
            Value::Number(n) => n.to_bits().hash(&mut hasher),
            Value::Bool(b) => b.hash(&mut hasher),
            _ => return Err(/* unhashable type */),
        }
        Ok(hasher.finish())
    }
}
```

**Hashable Types:** String, Number, Bool
**Unhashable Types:** Null, Array, HashMap, HashSet, Function, Object

**Rationale:**
- Simple, predictable hashing
- Standard Rust hasher (well-tested)
- Structural equality (not reference equality)

**Alternatives Considered:**
- ❌ Custom hash: Complexity, security concerns (hash flooding)
- ❌ Reference-based hash: Breaks semantic equality

**Impact:**
- HashMap/HashSet only accept hashable types as keys/elements
- Runtime error for unhashable types
- Consistent hashing across interpreter and VM

---

## DR-004: HashMap Key Equality

**Date:** 2025-02 (v0.2, Phase 07a)
**Status:** ✅ Active

**Decision:** HashMap keys use **value equality** (structural), not reference equality.

**Implementation:**
```rust
impl PartialEq for HashKey {
    fn eq(&self, other: &Self) -> bool {
        // Fast path: compare hashes first
        if self.hash != other.hash {
            return false;
        }

        // Slow path: compare values (handle hash collisions)
        values_equal(&self.value, &other.value)
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        _ => false,
    }
}
```

**Behavior:**
```atlas
let map = hashMapNew();
hashMapPut(map, "key", 100);
hashMapGet(map, "key");  // ✅ Returns 100 (string equality)

let key1 = "test";
let key2 = "test";
hashMapPut(map, key1, 200);
hashMapGet(map, key2);  // ✅ Returns 200 (value equality)
```

**Rationale:**
- Intuitive for users (JavaScript/Python-like)
- Matches developer expectations
- Enables literal key lookups

**Impact:**
- Hash collisions handled correctly
- Two-step equality check (hash then value)
- Performance: O(1) average, O(n) worst case (collisions)

---

## DR-005: Collection API Design

**Date:** 2025-02 (v0.2, Phase 07d)
**Status:** ✅ Active

**Decision:** Collections provide **functional iteration** via intrinsics.

**API Design:**

### HashMap Iteration (3 functions)

```atlas
// forEach: side effects only
hashMapForEach(map, fn(value, key) { print(key, value); });

// map: transform values, new map
let doubled = hashMapMap(map, fn(value, key) { value * 2; });

// filter: selective copy
let filtered = hashMapFilter(map, fn(value, key) { value > 10; });
```

### HashSet Iteration (3 functions)

```atlas
// forEach: side effects only
hashSetForEach(set, fn(elem) { print(elem); });

// map: transform to ARRAY (not set)
let arr = hashSetMap(set, fn(elem) { elem * 2; });

// filter: selective copy (returns SET)
let filtered = hashSetFilter(set, fn(elem) { elem > 10; });
```

**Key Decisions:**
1. **Callback signatures:**
   - HashMap: `fn(value, key)` - value first (consistent with JS)
   - HashSet: `fn(elem)` - single argument
   - Array: `fn(elem)` - single argument

2. **Return types:**
   - `forEach` → `null` (side effects)
   - `map` → new collection (transform)
   - `filter` → new collection (subset)
   - HashSet `map` → **Array** (not Set) - allows duplicates

3. **Implementation:**
   - Intrinsics (not stdlib functions) - need execution context
   - Callback errors propagate immediately
   - Empty collections → empty result

**Rationale:**
- Familiar to JS/Python developers
- Functional programming style
- Composable (chain operations)

**Alternatives Considered:**
- ❌ Iterator protocol: Complex, not idiomatic for Atlas
- ❌ For-in loops: Requires language syntax changes
- ❌ External iteration: Exposes implementation details

**Impact:**
- 6 intrinsics implemented (3 HashMap + 3 HashSet)
- Intrinsics in both interpreter and VM
- Tests cover callback patterns, edge cases, parity

---

## DR-006: Collection Benchmarking

**Date:** 2025-02 (v0.2, Phase 07d - Deferred)
**Status:** ⏳ Deferred (future phase)

**Decision:** Defer comprehensive benchmarking to dedicated performance phase.

**Rationale:**
- Phase 07d scope too large (implementation + tests + benchmarks + docs)
- Benchmarking requires stable API (current API is stable)
- Performance optimization is separate concern from correctness

**Plan:**
- Complete functional implementation (Phase 07d)
- Verify correctness and parity
- Benchmarking in separate phase (Bytecode-VM category)

**Benchmark Goals (Future):**
- HashMap: insert, get, remove, iteration (1K, 10K, 100K elements)
- HashSet: add, has, set operations, iteration
- Queue/Stack: push/pop throughput
- Comparison: Atlas vs Rust std collections

---

## DR-007: Phase File Accuracy

**Date:** 2025-02-16
**Status:** ✅ Active

**Problem:** Phase files referenced files that didn't exist:
- `docs/api/stdlib.md` (doesn't exist)
- `crates/atlas-runtime/src/stdlib/prelude.rs` (actual: `mod.rs`)
- `memory/patterns.md` (didn't exist - now created)

**Decision:** All phase files MUST reference actual files in codebase.

**Validation:**
1. Run all file path checks before committing phase
2. Verify all referenced patterns exist in documentation
3. Test all validation commands actually work

**Process:**
- Phase author runs validation commands
- Automated check script (`tools/validate-phase.sh`)
- Gate -1 includes file existence verification

**Impact:**
- Phase files are accurate
- Reduced wasted time during execution
- Clear error messages when dependencies missing

---

## DR-008: Scope Sizing for Phases

**Date:** 2025-02-16
**Status:** ✅ Active

**Problem:** Phase 07d was 3-4 phases compressed into one:
- 6 intrinsics (interpreter + VM)
- 33+ tests
- Benchmarks
- Documentation updates
- Integration tests

**Decision:** Phases should be **reasonably scoped** (~200 lines implementation, ~15 tests).

**Guidelines:**
- **Implementation:** Max 200-300 lines new code
- **Tests:** 10-20 tests per phase
- **Benchmarks:** Separate phase
- **Documentation:** Separate phase if substantial

**Example Split (Phase 07d):**
- 07d-1: Core iteration intrinsics (implementation only)
- 07d-2: Comprehensive testing (all test cases)
- 07d-3: Benchmarks and performance docs

**Rationale:**
- Manageable chunks (1-2 hour work sessions)
- Clear completion criteria
- Easier to validate and review

**Impact:**
- More phases (higher count) but each is smaller
- Faster feedback loops
- Reduced risk of scope creep

---

## Decision Log Format

**New decisions use this template:**

```markdown
## DR-XXX: Decision Title

**Date:** YYYY-MM-DD
**Status:** ✅ Active | ⏳ Deferred | ❌ Rejected | 🔄 Superseded

**Decision:** [What was decided]

**Rationale:** [Why this decision was made]

**Alternatives Considered:**
- ❌ Alternative 1: [Why rejected]
- ❌ Alternative 2: [Why rejected]

**Impact:** [What this affects, implementation notes]
```

---

## DR-009: Arc<Mutex<T>> Migration (Replaces DR-002)

**Date:** 2026-01 (v0.2, Phase 18a-18f)
**Status:** ✅ Active

**Decision:** All collection types migrated from `Rc<RefCell<T>>` to `Arc<Mutex<T>>`.

**New pattern:**
```rust
pub enum Value {
    Array(Arc<Mutex<Vec<Value>>>),
    HashMap(Arc<Mutex<AtlasHashMap>>),
    HashSet(Arc<Mutex<AtlasHashSet>>),
    Queue(Arc<Mutex<AtlasQueue>>),
    Stack(Arc<Mutex<AtlasStack>>),
    String(Arc<String>),
}
```

**Access:** `.lock().unwrap()` for reads and writes. NOT `.borrow()` / `.borrow_mut()`.

**Rationale:** Required for tokio async support. `Rc<RefCell<>>` is `!Send`, preventing use across thread boundaries. `Arc<Mutex<>>` is `Send + Sync`.

**Impact:** All collection code, intrinsics, stdlib functions updated in phases 18a-18f.

---

## References

- DR-001, DR-002: See `docs/specification/runtime.md`
- DR-003, DR-004: See `crates/atlas-runtime/src/stdlib/collections/hash.rs`
- DR-005: See `memory/patterns.md` (Intrinsic Pattern)
- DR-007, DR-008: See `memory/gates.md` (Gate -1: Validation)
- DR-009: See `crates/atlas-runtime/src/value.rs` (current collection types)

---

## DR-010: Type Alias Resolution and Metadata

**Date:** 2026-02 (v0.2, Phase typing-03)
**Status:** ✅ Active

**Decision:**
- Type aliases resolve structurally to their target type for assignability.
- Alias names are preserved for display and diagnostics via `Type::Alias`.
- Generic alias type arguments can be inferred from initializer context when omitted.
- Circular alias definitions are rejected with explicit cycle diagnostics.
- Doc comment tags `@deprecated` and `@since` are parsed for warnings/metadata.

**Rationale:**
- Preserves readability in errors while keeping structural type equivalence.
- Enables ergonomic alias usage without sacrificing correctness.
- Avoids infinite expansions by detecting cycles early.

**Impact:**
- `Type::Alias` added to type system.
- Type checker resolves and caches aliases; warnings emitted on deprecated aliases.
- Parser captures doc comments for alias declarations.
