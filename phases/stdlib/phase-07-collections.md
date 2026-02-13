# Phase 07: Collections - HashMap, HashSet, Queue

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Basic stdlib and generic types must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/stdlib/mod.rs
cargo test stdlib
grep -n "Generic" crates/atlas-runtime/src/typechecker/types.rs
```

**What's needed:**
- Stdlib infrastructure from v0.1
- Generic types from type system
- Hash function infrastructure
- Iterators or iteration support

**If missing:** Core systems should exist - enhancement needed

---

## Objective
Implement essential collection types HashMap, HashSet, Queue, and Stack providing efficient data structures for common programming patterns - completing stdlib with production-ready collections rivaling standard libraries of mature languages.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/collections/mod.rs` (~200 lines)
**Create:** `crates/atlas-runtime/src/stdlib/collections/hashmap.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/stdlib/collections/hashset.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/stdlib/collections/queue.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/stdlib/collections/stack.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~200 lines collection values)
**Update:** `docs/stdlib.md` (~400 lines collection docs)
**Tests:** `crates/atlas-runtime/tests/collections_tests.rs` (~1000 lines)

## Dependencies
- Stdlib infrastructure
- Generic type support
- Hash function (built-in or custom)
- Value equality comparison
- Iterator protocol (if exists)

## Implementation

### HashMap Implementation
Implement generic HashMap with key-value storage. Generic over key and value types. Hash table with separate chaining collision resolution. Automatic resizing with load factor threshold. Hash function for built-in types. Insertion with put method. Lookup with get method returning nullable. Removal with remove method. Contains check with has method. Keys and values accessors. Size and is_empty methods. Iteration over entries keys and values. Clear method removing all entries. Efficient O(1) average operations.

### HashSet Implementation
Implement HashSet backed by HashMap. Generic over element type. Store elements as keys with dummy values. Add method for insertion. Remove method for deletion. Contains method for membership test. Set operations union, intersection, difference, symmetric difference. Subset and superset tests. Size and is_empty methods. Iteration over elements. Clear method. Efficient set operations.

### Queue Implementation
Implement FIFO queue with circular buffer or linked list. Generic over element type. Enqueue method appending to back. Dequeue method removing from front. Peek method viewing front without removal. Size and is_empty methods. Clear method. Efficient O(1) enqueue and dequeue. Iteration preserving queue order. Optional capacity limit. Full queue handling.

### Stack Implementation
Implement LIFO stack with array or linked list. Generic over element type. Push method adding to top. Pop method removing from top. Peek method viewing top without removal. Size and is_empty methods. Clear method. Efficient O(1) push and pop operations. Iteration from top to bottom. Unbounded or bounded stack.

### Collection Iteration
Provide iteration over all collections. Iterator returning elements or entries. For-in loop support if language supports. Map method transforming elements. Filter method selecting elements. Reduce method aggregating. Lazy iteration where beneficial. Consistent iteration interface across collections.

### Memory Management
Manage memory efficiently for collections. Grow capacity dynamically as needed. Shrink capacity when appropriate. Avoid unnecessary allocations. Reuse backing storage. Memory-efficient representation. Consider cache locality. Benchmark memory usage.

### Thread Safety Considerations
Document thread safety properties. Collections not thread-safe by default. Exclusive access required for mutations. Provide guidance for concurrent use. Future: thread-safe variants if needed. Clear documentation of safety guarantees.

## Tests (TDD - Use rstest)

**HashMap tests:**
1. Create empty HashMap
2. Put and get key-value
3. Get nonexistent key returns null
4. Update existing key
5. Remove key
6. Has method membership
7. Keys accessor
8. Values accessor
9. Iteration over entries
10. Size and is_empty
11. Clear method
12. Collision handling
13. Automatic resizing
14. Hash function quality

**HashSet tests:**
1. Create empty HashSet
2. Add element
3. Contains membership test
4. Remove element
5. Union of sets
6. Intersection of sets
7. Difference of sets
8. Symmetric difference
9. Subset test
10. Superset test
11. Size and is_empty
12. Iteration over elements
13. Clear method

**Queue tests:**
1. Create empty queue
2. Enqueue element
3. Dequeue element FIFO order
4. Peek front element
5. Size and is_empty
6. Clear queue
7. Enqueue multiple dequeue multiple
8. Empty queue dequeue error
9. Iteration preserves order

**Stack tests:**
1. Create empty stack
2. Push element
3. Pop element LIFO order
4. Peek top element
5. Size and is_empty
6. Clear stack
7. Push multiple pop multiple
8. Empty stack pop error
9. Iteration top to bottom

**Collection iteration tests:**
1. Iterate HashMap entries
2. Iterate HashSet elements
3. Iterate Queue elements
4. Iterate Stack elements
5. Map over collection
6. Filter collection
7. Reduce collection
8. Lazy iteration

**Memory tests:**
1. HashMap dynamic growth
2. HashMap shrinking
3. HashSet memory usage
4. Queue circular buffer
5. Stack memory efficiency
6. Large collection handling

**Integration tests:**
1. Use collections in algorithms
2. Nested collections
3. Collections with complex types
4. Real-world use cases
5. Performance benchmarks

**Minimum test count:** 80 tests

## Integration Points
- Uses: Stdlib infrastructure
- Uses: Generic types
- Uses: Value equality and hashing
- Updates: Value with collection variants
- Creates: Essential collection types
- Output: Production-ready data structures

## Acceptance
- HashMap works with put, get, remove
- HashSet supports set operations
- Queue provides FIFO behavior
- Stack provides LIFO behavior
- Collections support iteration
- Automatic resizing in HashMap
- Efficient O(1) average operations
- Memory usage reasonable
- 80+ tests pass
- Documentation with examples
- Performance benchmarks included
- No clippy warnings
- cargo test passes
