# Phase 07d: Collection Integration + Iteration

## Dependencies

**Required:** Phases 07a (HashMap), 07b (HashSet), 07c (Queue/Stack) complete

**Verification:**
```bash
# Check all collection implementations exist
ls crates/atlas-runtime/src/stdlib/collections/{hashmap,hashset,queue,stack,hash}.rs

# Verify Value variants
grep "HashMap\|HashSet\|Queue\|Stack" crates/atlas-runtime/src/value.rs | grep "Rc<RefCell"

# Verify all collection tests pass
cargo test -p atlas-runtime hashmap_tests hashset_tests queue_tests stack_tests -- --nocapture

# Clean build check
cargo clean && cargo check -p atlas-runtime
```

**If missing:** Complete phases 07a, 07b, 07c first

---

## Objective

Add iteration support (forEach, map, filter) for HashMap and HashSet as interpreter/VM intrinsics. Create cross-collection integration tests. Add performance benchmarks for all collections. Complete documentation with real-world examples.

---

## Files

**Update:** `crates/atlas-runtime/src/interpreter.rs` (~200 lines - intrinsics)
**Update:** `crates/atlas-runtime/src/vm.rs` (~200 lines - intrinsics)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (~50 lines - register functions)
**Update:** `docs/api/stdlib.md` (~200 lines - iteration docs + examples)
**Tests:** `crates/atlas-runtime/tests/collection_integration_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/collection_iteration_tests.rs` (~400 lines)
**Create:** `crates/atlas-runtime/benches/collection_benchmarks.rs` (~300 lines)

**Total new code:** ~450 lines
**Total tests:** ~1,000 lines (30+ test cases)
**Total benchmarks:** ~300 lines

---

## Dependencies (Components)

- All collection types (HashMap, HashSet, Queue, Stack from 07a-c)
- Array intrinsics pattern (examine array map/filter/forEach implementation)
- Interpreter/VM execution contexts
- Criterion benchmark library (from phase 06c)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine array intrinsics in interpreter.rs and vm.rs (map, filter, forEach)
- Follow callback intrinsic pattern from memory/patterns.md
- Reference DR-005 (Collection API Design) for iteration strategy

**HashMap/HashSet iteration functions (6 total):**
- `hashMapForEach(map, fn)` - iterate with side effects
- `hashMapMap(map, fn)` - transform values, return new map
- `hashMapFilter(map, fn)` - filter entries, return new map
- `hashSetForEach(set, fn)` - iterate with side effects
- `hashSetFilter(set, fn)` - filter elements, return new set
- `hashSetMap(set, fn)` - transform elements to array (Set→Array)

**Implementation locations:**
- Interpreter intrinsics: interpreter.rs (builtin_call match)
- VM intrinsics: vm.rs (execute_call_builtin)
- Function registration: stdlib/prelude.rs (register_native macro)

**Integration tests cover:**
- Cross-collection operations (Map→Set, Set→Array, etc.)
- Mixed type scenarios
- Edge cases (empty collections, large collections)
- Error scenarios (wrong types, callback errors)

**Benchmarks cover:**
- HashMap: insert, get, remove, iteration (1K, 10K, 100K elements)
- HashSet: add, has, set operations, iteration
- Queue: enqueue/dequeue throughput
- Stack: push/pop throughput
- Compare against Rust std HashMap/HashSet/VecDeque/Vec

---

## Tests (TDD Approach)

### HashMap Iteration Tests (10 tests)
1. forEach iterates all entries with correct order
2. map transforms values, preserves keys
3. filter keeps entries matching predicate
4. Callback receives (value, key) arguments
5. Empty map iteration (no callback calls)
6. Callback errors propagate correctly
7. Nested callbacks work correctly
8. Large map iteration (1000+ entries)
9. Mixed value types in callbacks
10. Chaining operations (map then filter)

### HashSet Iteration Tests (8 tests)
1. forEach iterates all elements
2. filter keeps elements matching predicate
3. map transforms to array (not set)
4. Empty set iteration
5. Callback errors propagate
6. Large set iteration
7. Set operations + iteration combo
8. Chaining filter operations

### Integration Tests (15 tests)
1. Convert HashMap keys to HashSet
2. Convert HashSet to Array and back
3. Filter HashMap, convert values to Set
4. Queue→Array→HashSet pipeline
5. Stack→Array→HashMap pipeline
6. Mixed collection operations
7. Deep nesting (Map of Arrays, Array of Maps)
8. Error propagation across collections
9. Reference semantics verification (shared collections)
10. Multiple collection types in single function
11. Collection type guards work correctly
12. Empty collection edge cases
13. Large-scale integration (10K+ elements)
14. Memory behavior (no leaks with Rc<RefCell<>>)
15. Parity verification (interpreter vs VM results)

### Benchmark Tests (4 categories)
1. HashMap performance (insert, get, remove, iterate)
2. HashSet performance (add, has, operations, iterate)
3. Queue/Stack performance (push/pop throughput)
4. Comparison benchmarks (Atlas vs Rust std collections)

**Minimum test count:** 33 tests

**Parity requirement:** All tests run in both interpreter and VM with identical results.

---

## Acceptance Criteria

- ✅ 6 iteration functions implemented (3 HashMap + 3 HashSet)
- ✅ hashMapForEach, hashMapMap, hashMapFilter work correctly
- ✅ hashSetForEach, hashSetFilter, hashSetMap work correctly
- ✅ All intrinsics implemented in both interpreter and VM
- ✅ Callback pattern matches array intrinsics (consistent API)
- ✅ 33+ tests pass (10 HashMap + 8 HashSet + 15 integration)
- ✅ Collection benchmarks complete (4 categories)
- ✅ Benchmarks show competitive performance vs Rust std
- ✅ Integration tests verify cross-collection operations
- ✅ Error handling complete (callback errors, type errors)
- ✅ 100% interpreter/VM parity verified
- ✅ Documentation complete with real-world examples
- ✅ No clippy warnings
- ✅ cargo test -p atlas-runtime passes
- ✅ cargo bench (benchmarks run successfully)
- ✅ Decision log DR-005 (Collection API) referenced

---

## References

**Decision Logs:** DR-005 (Collection API Design and Iteration)
**Specifications:**
- docs/specification/runtime.md (Value representation, intrinsics)
- memory/patterns.md (Callback intrinsic pattern from arrays)
**Previous phases:**
- phase-07a-hash-infrastructure-hashmap.md (HashMap)
- phase-07b-hashset.md (HashSet)
- phase-07c-queue-stack.md (Queue/Stack)
**Next phase:** phase-08-regex.md (Regular expressions)
