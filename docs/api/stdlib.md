# Atlas Standard Library API Reference

**Purpose:** Complete API reference for Atlas standard library functions
**Audience:** AI agents and developers
**Status:** Growing - phases add functions as implemented

---

## Table of Contents
1. [Prelude (Built-in Functions)](#prelude-built-in-functions)
2. [String Functions](#string-functions)
3. [Array Functions](#array-functions)
4. [Math Functions](#math-functions)
5. [JSON Functions](#json-functions)
6. [File I/O Functions](#file-io-functions)
7. [Collection Functions](#collection-functions)
8. [Regex Functions](#regex-functions)
9. [DateTime Functions](#datetime-functions)
10. [Network Functions](#network-functions)

---

## Prelude (Built-in Functions)

**Note:** These functions are always in scope (no import needed).
**See also:** `docs/RUNTIME.md` for runtime behavior details.

### print
**Signature:** `print(value: string|number|bool|null) -> void`
**Behavior:** Writes value to stdout with newline
**Example:** `print("hello");` outputs `hello`
**Errors:** AT0102 if wrong type

### len
**Signature:** `len(value: string|T[]) -> number`
**Behavior:** Returns length (Unicode scalars for strings, element count for arrays)
**Example:** `len("🌟")` returns `1`
**Errors:** AT0102 if wrong type

### str
**Signature:** `str(value: number|bool|null) -> string`
**Behavior:** Converts value to string representation
**Example:** `str(42)` returns `"42"`
**Errors:** AT0102 if wrong type

---

## String Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/string.rs`
**Phase:** phases/stdlib/phase-01-complete-string-api.md

### Core Operations

#### split
**Signature:** `split(s: string, separator: string) -> string[]`
**Behavior:** Divides string by separator, returns array of parts
**Example:** `split("a,b,c", ",")` returns `["a", "b", "c"]`
**Errors:** AT0102 if wrong types

#### join
**Signature:** `join(parts: string[], separator: string) -> string`
**Behavior:** Combines array of strings with separator
**Example:** `join(["a", "b"], ",")` returns `"a,b"`
**Errors:** AT0102 if wrong types

#### trim
**Signature:** `trim(s: string) -> string`
**Behavior:** Removes leading and trailing whitespace (Unicode-aware)
**Example:** `trim("  hello  ")` returns `"hello"`
**Errors:** AT0102 if wrong type

#### trimStart
**Signature:** `trimStart(s: string) -> string`
**Behavior:** Removes leading whitespace (Unicode-aware)
**Example:** `trimStart("  hello")` returns `"hello"`
**Errors:** AT0102 if wrong type

#### trimEnd
**Signature:** `trimEnd(s: string) -> string`
**Behavior:** Removes trailing whitespace (Unicode-aware)
**Example:** `trimEnd("hello  ")` returns `"hello"`
**Errors:** AT0102 if wrong type

### Search Operations

#### indexOf
**Signature:** `indexOf(s: string, search: string) -> number`
**Behavior:** Returns index of first occurrence, -1 if not found
**Example:** `indexOf("hello", "ll")` returns `2`
**Errors:** AT0102 if wrong types

#### lastIndexOf
**Signature:** `lastIndexOf(s: string, search: string) -> number`
**Behavior:** Returns index of last occurrence, -1 if not found
**Example:** `lastIndexOf("hello", "l")` returns `3`
**Errors:** AT0102 if wrong types

#### includes
**Signature:** `includes(s: string, search: string) -> bool`
**Behavior:** Returns true if search string is found
**Example:** `includes("hello", "ll")` returns `true`
**Errors:** AT0102 if wrong types

### Transformation

#### toUpperCase
**Signature:** `toUpperCase(s: string) -> string`
**Behavior:** Converts to uppercase (Unicode-aware)
**Example:** `toUpperCase("hello")` returns `"HELLO"`
**Errors:** AT0102 if wrong type

#### toLowerCase
**Signature:** `toLowerCase(s: string) -> string`
**Behavior:** Converts to lowercase (Unicode-aware)
**Example:** `toLowerCase("HELLO")` returns `"hello"`
**Errors:** AT0102 if wrong type

#### substring
**Signature:** `substring(s: string, start: number, end: number) -> string`
**Behavior:** Extracts substring from start to end (UTF-8 boundary safe)
**Example:** `substring("hello", 1, 4)` returns `"ell"`
**Errors:** AT0102 if wrong types, AT0103 if invalid indices

#### charAt
**Signature:** `charAt(s: string, index: number) -> string`
**Behavior:** Returns grapheme cluster at index (not byte)
**Example:** `charAt("hello", 0)` returns `"h"`
**Errors:** AT0102 if wrong types, AT0103 if out of bounds

#### repeat
**Signature:** `repeat(s: string, count: number) -> string`
**Behavior:** Repeats string count times (max count to prevent abuse)
**Example:** `repeat("ha", 3)` returns `"hahaha"`
**Errors:** AT0102 if wrong types, AT0104 if count too large

#### replace
**Signature:** `replace(s: string, search: string, replacement: string) -> string`
**Behavior:** Replaces first occurrence of search with replacement
**Example:** `replace("hello", "l", "L")` returns `"heLlo"`
**Errors:** AT0102 if wrong types

### Formatting

#### padStart
**Signature:** `padStart(s: string, length: number, fill: string) -> string`
**Behavior:** Pads start with fill to reach target length
**Example:** `padStart("5", 3, "0")` returns `"005"`
**Errors:** AT0102 if wrong types

#### padEnd
**Signature:** `padEnd(s: string, length: number, fill: string) -> string`
**Behavior:** Pads end with fill to reach target length
**Example:** `padEnd("5", 3, "0")` returns `"500"`
**Errors:** AT0102 if wrong types

#### startsWith
**Signature:** `startsWith(s: string, prefix: string) -> bool`
**Behavior:** Returns true if string starts with prefix
**Example:** `startsWith("hello", "he")` returns `true`
**Errors:** AT0102 if wrong types

#### endsWith
**Signature:** `endsWith(s: string, suffix: string) -> bool`
**Behavior:** Returns true if string ends with suffix
**Example:** `endsWith("hello", "lo")` returns `true`
**Errors:** AT0102 if wrong types

---

## Array Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/array.rs`
**Phase:** phases/stdlib/phase-02-complete-array-api.md

**Note:** All array functions return NEW arrays - originals are never mutated (immutability).
**Callbacks:** v0.2 supports named functions only (no anonymous functions or closures).

### Core Operations

#### pop
**Signature:** `pop(arr: T[]) -> [T, T[]]`
**Behavior:** Removes and returns last element plus new array without that element
**Example:** `pop([1, 2, 3])` returns `[3, [1, 2]]`
**Errors:** AT0103 if array is empty

#### shift
**Signature:** `shift(arr: T[]) -> [T, T[]]`
**Behavior:** Removes and returns first element plus new array without that element
**Example:** `shift([1, 2, 3])` returns `[1, [2, 3]]`
**Errors:** AT0103 if array is empty

#### unshift
**Signature:** `unshift(arr: T[], element: T) -> T[]`
**Behavior:** Returns new array with element prepended
**Example:** `unshift([2, 3], 1)` returns `[1, 2, 3]`
**Errors:** AT0102 if wrong types

#### reverse
**Signature:** `reverse(arr: T[]) -> T[]`
**Behavior:** Returns new array with elements in reverse order
**Example:** `reverse([1, 2, 3])` returns `[3, 2, 1]`
**Errors:** AT0102 if wrong type

#### concat
**Signature:** `concat(arr1: T[], arr2: T[]) -> T[]`
**Behavior:** Returns new array combining both arrays
**Example:** `concat([1, 2], [3, 4])` returns `[1, 2, 3, 4]`
**Errors:** AT0102 if wrong types

#### flatten
**Signature:** `flatten(arr: T[][]) -> T[]`
**Behavior:** Flattens nested arrays by one level
**Example:** `flatten([[1], [2, 3]])` returns `[1, 2, 3]`
**Errors:** AT0102 if wrong type

### Search Operations

#### arrayIndexOf
**Signature:** `arrayIndexOf(arr: T[], element: T) -> number`
**Behavior:** Returns index of first occurrence, -1 if not found
**Example:** `arrayIndexOf([1, 2, 3, 2], 2)` returns `1`
**Errors:** AT0102 if wrong types

#### arrayLastIndexOf
**Signature:** `arrayLastIndexOf(arr: T[], element: T) -> number`
**Behavior:** Returns index of last occurrence, -1 if not found
**Example:** `arrayLastIndexOf([1, 2, 3, 2], 2)` returns `3`
**Errors:** AT0102 if wrong types

#### arrayIncludes
**Signature:** `arrayIncludes(arr: T[], element: T) -> bool`
**Behavior:** Returns true if element is in array
**Example:** `arrayIncludes([1, 2, 3], 2)` returns `true`
**Errors:** AT0102 if wrong types

#### find
**Signature:** `find(arr: T[], predicate: (T) -> bool) -> T | null`
**Behavior:** Returns first element matching predicate, null if none match
**Example:** `find([1, 5, 10], fn(x) { return x > 3; })` returns `5`
**Errors:** AT0102 if wrong types, AT0104 if predicate doesn't return bool

#### findIndex
**Signature:** `findIndex(arr: T[], predicate: (T) -> bool) -> number`
**Behavior:** Returns index of first element matching predicate, -1 if none match
**Example:** `findIndex([1, 5, 10], fn(x) { return x > 3; })` returns `1`
**Errors:** AT0102 if wrong types, AT0104 if predicate doesn't return bool

### Slicing

#### slice
**Signature:** `slice(arr: T[], start: number, end: number) -> T[]`
**Behavior:** Returns new array containing elements from start (inclusive) to end (exclusive)
**Example:** `slice([0, 1, 2, 3, 4], 1, 4)` returns `[1, 2, 3]`
**Errors:** AT0102 if wrong types, AT0105 if indices invalid

### Iteration & Transformation

#### map
**Signature:** `map(arr: T[], fn: (T) -> U) -> U[]`
**Behavior:** Returns new array with function applied to each element
**Example:** `map([1, 2, 3], double)` returns `[2, 4, 6]`
**Errors:** AT0102 if wrong types

#### filter
**Signature:** `filter(arr: T[], predicate: (T) -> bool) -> T[]`
**Behavior:** Returns new array with elements matching predicate
**Example:** `filter([1, 2, 3, 4], isEven)` returns `[2, 4]`
**Errors:** AT0102 if wrong types, AT0104 if predicate doesn't return bool

#### reduce
**Signature:** `reduce(arr: T[], fn: (Acc, T) -> Acc, initial: Acc) -> Acc`
**Behavior:** Reduces array to single value by applying function cumulatively
**Example:** `reduce([1, 2, 3], add, 0)` returns `6`
**Errors:** AT0102 if wrong types

#### forEach
**Signature:** `forEach(arr: T[], fn: (T) -> void) -> null`
**Behavior:** Executes function for each element (side effects only)
**Example:** `forEach([1, 2, 3], print)`
**Errors:** AT0102 if wrong types

#### flatMap
**Signature:** `flatMap(arr: T[], fn: (T) -> U[]) -> U[]`
**Behavior:** Maps each element to array, then flattens results by one level
**Example:** `flatMap([1, 2], duplicate)` returns `[1, 1, 2, 2]`
**Errors:** AT0102 if wrong types

### Testing Predicates

#### some
**Signature:** `some(arr: T[], predicate: (T) -> bool) -> bool`
**Behavior:** Returns true if any element matches predicate
**Example:** `some([1, 2, 3], fn(x) { return x > 2; })` returns `true`
**Errors:** AT0102 if wrong types, AT0104 if predicate doesn't return bool

#### every
**Signature:** `every(arr: T[], predicate: (T) -> bool) -> bool`
**Behavior:** Returns true if all elements match predicate
**Example:** `every([2, 4, 6], isEven)` returns `true`
**Errors:** AT0102 if wrong types, AT0104 if predicate doesn't return bool

### Sorting

#### sort
**Signature:** `sort(arr: T[], comparator: (T, T) -> number) -> T[]`
**Behavior:** Returns new sorted array using comparator (negative = first < second, 0 = equal, positive = first > second). Stable sort.
**Example:** `sort([3, 1, 2], fn(a, b) { return a - b; })` returns `[1, 2, 3]`
**Errors:** AT0102 if wrong types, AT0104 if comparator doesn't return number

#### sortBy
**Signature:** `sortBy(arr: T[], keyFn: (T) -> number|string) -> T[]`
**Behavior:** Returns new sorted array by comparing keys extracted by keyFn. Stable sort.
**Example:** `sortBy([3, -5, 2], abs)` returns `[2, 3, -5]`
**Errors:** AT0102 if wrong types

---

## Math Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/math.rs`
**Phase:** phases/stdlib/phase-03-complete-math-api.md

**Note:** All functions follow IEEE 754 semantics (NaN propagates, infinities handled, signed zero preserved).

### Constants

#### PI
**Value:** `3.141592653589793`
**Behavior:** Mathematical constant π

#### E
**Value:** `2.718281828459045`
**Behavior:** Mathematical constant e (Euler's number)

#### SQRT2
**Value:** `1.4142135623730951`
**Behavior:** Square root of 2

#### LN2
**Value:** `0.6931471805599453`
**Behavior:** Natural logarithm of 2

#### LN10
**Value:** `2.302585092994046`
**Behavior:** Natural logarithm of 10

### Basic Operations

#### abs
**Signature:** `abs(x: number) -> number`
**Behavior:** Returns absolute value (handles signed zero and infinities)
**Example:** `abs(-5)` returns `5`
**Errors:** AT0102 if wrong type

#### floor
**Signature:** `floor(x: number) -> number`
**Behavior:** Returns largest integer ≤ x (preserves special values)
**Example:** `floor(3.7)` returns `3`
**Errors:** AT0102 if wrong type

#### ceil
**Signature:** `ceil(x: number) -> number`
**Behavior:** Returns smallest integer ≥ x (preserves special values)
**Example:** `ceil(3.2)` returns `4`
**Errors:** AT0102 if wrong type

#### round
**Signature:** `round(x: number) -> number`
**Behavior:** Returns nearest integer using ties-to-even (banker's rounding)
**Example:** `round(2.5)` returns `2`, `round(3.5)` returns `4`
**Errors:** AT0102 if wrong type

#### min
**Signature:** `min(a: number, b: number) -> number`
**Behavior:** Returns smaller value (NaN propagates correctly)
**Example:** `min(5, 3)` returns `3`
**Errors:** AT0102 if wrong types

#### max
**Signature:** `max(a: number, b: number) -> number`
**Behavior:** Returns larger value (NaN propagates correctly)
**Example:** `max(5, 3)` returns `5`
**Errors:** AT0102 if wrong types

### Exponential & Power

#### sqrt
**Signature:** `sqrt(x: number) -> number`
**Behavior:** Returns square root (negative input returns NaN)
**Example:** `sqrt(16)` returns `4`, `sqrt(-1)` returns `NaN`
**Errors:** AT0102 if wrong type

#### pow
**Signature:** `pow(base: number, exponent: number) -> number`
**Behavior:** Returns base raised to exponent power
**Example:** `pow(2, 3)` returns `8`
**Errors:** AT0102 if wrong types

#### log
**Signature:** `log(x: number) -> number`
**Behavior:** Returns natural logarithm (negative/zero input returns NaN/-Infinity)
**Example:** `log(E)` returns `1`
**Errors:** AT0102 if wrong type

### Trigonometry

**Note:** All trigonometric functions use radians.

#### sin
**Signature:** `sin(x: number) -> number`
**Behavior:** Returns sine of x (radians)
**Example:** `sin(0)` returns `0`
**Errors:** AT0102 if wrong type

#### cos
**Signature:** `cos(x: number) -> number`
**Behavior:** Returns cosine of x (radians)
**Example:** `cos(0)` returns `1`
**Errors:** AT0102 if wrong type

#### tan
**Signature:** `tan(x: number) -> number`
**Behavior:** Returns tangent of x (radians)
**Example:** `tan(0)` returns `0`
**Errors:** AT0102 if wrong type

#### asin
**Signature:** `asin(x: number) -> number`
**Behavior:** Returns arcsine of x in radians (domain: [-1, 1], returns NaN outside)
**Example:** `asin(0.5)` returns approx `0.5236` (π/6)
**Errors:** AT0102 if wrong type

#### acos
**Signature:** `acos(x: number) -> number`
**Behavior:** Returns arccosine of x in radians (domain: [-1, 1], returns NaN outside)
**Example:** `acos(0.5)` returns approx `1.0472` (π/3)
**Errors:** AT0102 if wrong type

#### atan
**Signature:** `atan(x: number) -> number`
**Behavior:** Returns arctangent of x in radians
**Example:** `atan(1)` returns approx `0.7854` (π/4)
**Errors:** AT0102 if wrong type

### Utilities

#### clamp
**Signature:** `clamp(value: number, min: number, max: number) -> number`
**Behavior:** Restricts value to range [min, max] (validates min ≤ max)
**Example:** `clamp(15, 0, 10)` returns `10`
**Errors:** AT0102 if wrong types, AT0105 if min > max

#### sign
**Signature:** `sign(x: number) -> number`
**Behavior:** Returns -1 for negative, 0 for zero, 1 for positive (preserves signed zero)
**Example:** `sign(-5)` returns `-1`
**Errors:** AT0102 if wrong type

#### random
**Signature:** `random() -> number`
**Behavior:** Returns random number in [0, 1) with uniform distribution
**Example:** `random()` returns e.g. `0.723...`
**Errors:** None

---

## JSON Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/json.rs`
**Phase:** phases/stdlib/phase-04-json-type-utilities.md

_Phases will populate this section with JSON parsing and serialization functions_

---

## File I/O Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/io.rs`
**Phase:** phases/stdlib/phase-05-complete-file-io-api.md

_Phases will populate this section with file I/O functions_

---

## Collection Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/collections.rs`
**Phase:** phases/stdlib/phase-07-collections.md

_Phases will populate this section with HashMap, HashSet, Queue, Stack APIs_

---

## Regex Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/regex.rs`
**Phase:** phases/stdlib/phase-08-regex.md

_Phases will populate this section with regex functions_

---

## DateTime Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/datetime.rs`
**Phase:** phases/stdlib/phase-09-datetime.md

_Phases will populate this section with date/time functions_

---

## Network Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/network.rs`
**Phase:** phases/stdlib/phase-10-network-http.md

_Phases will populate this section with HTTP client functions_

---

## API Documentation Standards

**For AI agents adding new functions:**

1. **Format:** Follow the pattern above (Signature → Behavior → Example → Errors)
2. **Signatures:** Be explicit about types
3. **Behavior:** One sentence summary, key details below
4. **Examples:** Show common usage, not edge cases
5. **Errors:** List error codes, not full messages
6. **No code dumps:** This is API reference, not implementation
7. **Token efficiency:** Keep descriptions concise

**Example entry:**
```markdown
#### functionName
**Signature:** `functionName(arg: type) -> returnType`
**Behavior:** What it does in one sentence
**Example:** `functionName("input")` returns `"output"`
**Errors:** AT0102 if wrong type, AT0103 if invalid input
```
