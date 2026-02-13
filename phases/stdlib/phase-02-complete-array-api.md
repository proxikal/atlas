# Phase 02: Complete Array API

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Value model must support array operations and first-class functions.

**Verification:**
```bash
grep -n "Array(Vec<Value>)" crates/atlas-runtime/src/value.rs
grep -n "Value::Function" crates/atlas-runtime/src/value.rs
grep -n "fn.*push\|fn.*slice" crates/atlas-runtime/src/stdlib/prelude.rs
```

**What's needed:**
- Value::Array variant exists ✓
- Value::Function works for callbacks (first-class functions) ✓
- Basic array ops from v0.1 ✓

**If missing:** Block - array core and first-class functions should exist

**Note:** This phase uses **named functions only** for callbacks. Anonymous functions and closures
with capture are deferred to v0.3. All callbacks must be pre-declared functions passed by name.

---

## Objective
Implement complete array manipulation API with 21 functions covering mutation, iteration, search, slicing, and sorting with functional programming support.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/array.rs` (~1200 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (add array module)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (register functions)
**Tests:** `crates/atlas-runtime/tests/stdlib_array_tests.rs` (~800 lines)
**VM Tests:** `crates/atlas-runtime/tests/vm_stdlib_array_tests.rs` (~800 lines)

## Dependencies
- v0.1 complete ✓
- First-class functions implemented (named functions as values) ✓
- Existing push and slice from v0.1 ✓
- Atlas-SPEC.md defines array semantics - immutable by default ✓

## Implementation

### Core Operations (6 functions)
Implement pop, shift, unshift, reverse, concat, flatten. Pop and shift remove elements returning new array plus removed value. Unshift prepends. Reverse returns new reversed array. Concat combines arrays. Flatten reduces nesting one level.

### Iteration/Transformation (4 functions)
Implement map, filter, reduce, forEach. These take callback functions needing interpreter access for execution. Map transforms elements. Filter keeps matching elements. Reduce accumulates to single value. forEach executes for side effects only.

### Search Operations (5 functions)
Implement indexOf, lastIndexOf, includes, find, findIndex. Index functions find first/last occurrence returning index or -1. Includes returns boolean. Find functions use predicates returning element or index.

### Slicing/Manipulation (4 functions)
Implement flatMap, some, every plus keep existing slice. FlatMap combines map and flatten. Some checks if any element matches. Every checks if all match. Handle empty array edge cases.

### Sorting (2 functions)
Implement sort and sortBy. Sort uses custom comparator function. SortBy uses key extraction function. Must be stable sort maintaining relative order for equal elements.

### Architecture Notes
All array functions return NEW arrays - never mutate originals. Atlas arrays are immutable. Functions with callbacks need interpreter parameter for execution. Implement Value equality checking for indexOf/includes. Use stable sorting algorithm.

### Callback Execution Pattern
Callbacks use **named functions only** (no anonymous functions or closures with capture in this phase).
Example pattern:
```atlas
fn double(x: number) -> number { return x * 2; }
let arr = [1, 2, 3];
let doubled = map(arr, double);  // Pass function by name
```

Prelude wrappers pass interpreter to array functions. Array functions call interpreter.call_function
for each callback invocation using the function value.

## Tests (TDD - Use rstest)

**Array tests cover:**
1. Each function with basic operations
2. Immutability - originals never changed
3. Callback functions - **named functions only** (no closures)
4. Empty arrays and edge cases
5. Sorting stability
6. VM parity for all operations

**Callback test examples:**
```atlas
// Valid: Named function
fn isEven(x: number) -> bool { return x % 2 == 0; }
filter([1,2,3,4], isEven);  // ✓

// Invalid: Anonymous function (not supported yet)
filter([1,2,3,4], fn(x) { return x % 2 == 0; });  // ✗ (v0.3+)

// Invalid: Closure with capture (not supported yet)
let threshold = 5;
filter(arr, fn(x) { return x > threshold; });  // ✗ (v0.3+)
```

**Minimum test count:** 150 tests (75 interpreter, 75 VM)

## Integration Points
- Uses: Value::Array, Value::Function, Value::Closure
- Uses: Interpreter::call_function for callbacks
- Uses: RuntimeError for error handling
- Updates: prelude.rs with 21 functions
- Updates: docs/api/stdlib.md
- Output: Complete array API in Atlas

## Acceptance
- All 21 functions implemented
- Immutability enforced throughout
- Named function callbacks work in map/filter/reduce ✓
- Stable sorting implemented
- All edge cases handled
- 150+ tests pass
- Interpreter/VM parity verified
- array.rs under 1300 lines
- Test files under 900 lines each
- Documentation updated
- No clippy warnings
- cargo test passes

**Known Limitations (deferred to v0.3+):**
- No anonymous function syntax: `fn(x) { ... }`
- No closure capture: Functions can only reference globals, not captured locals
- All callbacks must be pre-declared named functions
