# Atlas Module System Specification

**Purpose:** Define module imports, exports, and resolution.
**Status:** Living document — reflects current implementation.

---

## Overview

Atlas module system enables multi-file programs with explicit imports and exports. Designed for clarity, safety, and AI-friendliness.

**Key principles:**
- Explicit exports (no implicit exports)
- Static imports (no dynamic loading)
- Cyclic dependencies rejected
- Single evaluation guarantee

---

## File Structure

### File Extension

`.atl` - All Atlas source files use this extension

### Module Identification

Modules identified by file path:
- Relative: `./sibling.atl`, `../parent.atl`
- Absolute: `/src/utils.atl` (from project root)

**Note:** `.atl` extension can be omitted in import paths

---

## Export Syntax

### Function Exports

```atlas
// Export function definition
export fn add(a: number, b: number) -> number {
    return a + b;
}

// Generic function export
export fn identity<T>(x: T) -> T {
    return x;
}
```

### Variable Exports

```atlas
// Export constant
export let PI = 3.14159;

// Export mutable variable
export var counter = 0;
```

### Current Limitations

- No default exports (`export default`)
- No export renaming (`export { x as y }`)
- No re-exports (`export { x } from "./mod"`)
- No type-only exports

---

## Import Syntax

### Named Imports

```atlas
// Import specific exports
import { add, subtract } from "./math";

// Import multiple from same module
import { PI, E, sqrt } from "./constants";

// Use imported names
let result = add(2, 3);
print(str(PI));
```

### Namespace Import

```atlas
// Import all exports under namespace
import * as math from "./math";

// Access via namespace
let sum = math.add(10, 20);
print(str(math.PI));
```

### Current Limitations

- No default imports (`import x from "./mod"`)
- No import renaming (`import { x as y }`)
- No dynamic imports (`import()`)
- No type-only imports
- REPL mode does not support imports

---

## Module Paths

### Relative Paths

```atlas
import { x } from "./sibling";       // Same directory
import { y } from "../parent";       // Parent directory
import { z } from "../../utils";    // Two levels up
import { w } from "./sub/module";   // Subdirectory
```

### Absolute Paths

```atlas
import { config } from "/src/config";    // From project root
import { utils } from "/lib/utils";      // Library path
```

**Note:** Absolute paths resolve from project root (defined by nearest `atlas.toml` or current directory)

### Path Resolution

1. Append `.atl` if no extension
2. Check if file exists
3. If relative: resolve from importing file's directory
4. If absolute: resolve from project root
5. Error if file not found

---

## Module Semantics

### Single Evaluation

Each module executed exactly once, regardless of import count:

```atlas
// counter.atl
export var count = 0;
count = count + 1;
print("Module loaded");

// main.atl
import { count } from "./counter";  // Prints "Module loaded"
print(str(count));  // 1

import { count } from "./counter";  // No print (already loaded)
print(str(count));  // Still 1 (same instance)
```

### Module Caching

- Modules cached by absolute path
- First import evaluates module
- Subsequent imports use cached exports
- Cache persists for program lifetime

### Initialization Order

Modules initialized in topological order by dependencies:

```atlas
// a.atl
import { b_value } from "./b";
export let a_value = b_value + 1;

// b.atl
export let b_value = 42;

// main.atl
import { a_value } from "./a";
// Initialization order: b → a → main
```

---

## Circular Dependencies

### Rejected

Cyclic imports are **compile errors:**

```atlas
// a.atl
import { b } from "./b";  // Error: circular dependency
export let a = 1;

// b.atl
import { a } from "./a";  // Error: circular dependency
export let b = 2;
```

**Error:** `AT1013: Circular dependency detected: a.atl → b.atl → a.atl`

**Rationale:** Circular dependencies create initialization order problems. Atlas rejects them at compile time.

### Workaround

Refactor into three modules:

```atlas
// shared.atl
export let shared = 0;

// a.atl
import { shared } from "./shared";
export let a = shared + 1;

// b.atl
import { shared } from "./shared";
export let b = shared + 2;
```

---

## Export Visibility

### Module-Private

Non-exported declarations are module-private:

```atlas
// utils.atl
fn helper(x: number) -> number {  // Private
    return x * 2;
}

export fn publicFn(x: number) -> number {  // Public
    return helper(x);
}

// main.atl
import { publicFn } from "./utils";
publicFn(5);  // OK
helper(5);    // Error: 'helper' not exported
```

### No Internal Access

Importing module cannot access private declarations:

```atlas
// math.atl
let PRECISION = 0.0001;  // Private
export fn round(x: number) -> number {
    // ...
}

// main.atl
import { round } from "./math";
print(str(PRECISION));  // Error: not exported
```

---

## Examples

### Math Module

```atlas
// math.atl
export fn add(a: number, b: number) -> number {
    return a + b;
}

export fn subtract(a: number, b: number) -> number {
    return a - b;
}

export let PI = 3.14159;
export let E = 2.71828;

// Helper (private)
fn validate(x: number) -> bool {
    return !is_nan(x);
}
```

### Using Math Module

```atlas
// main.atl
import { add, PI } from "./math";

let result = add(10, 20);
print(str(result));  // 30

let circumference = 2 * PI * 5;
print(str(circumference));  // 31.4159
```

### Namespace Import

```atlas
// main.atl
import * as math from "./math";

let sum = math.add(1, 2);
let diff = math.subtract(5, 3);
print(str(sum + diff));  // 5

print(str(math.PI));  // 3.14159
```

---

## Type Checking

### Cross-Module Types

Types checked across module boundaries:

```atlas
// types.atl
export fn process(x: number) -> string {
    return str(x * 2);
}

// main.atl
import { process } from "./types";
let result: string = process(21);  // OK
let wrong: number = process(21);   // Error: type mismatch
```

### Function Signatures

Imported function signatures must match:

```atlas
// lib.atl
export fn greet(name: string) -> void {
    print("Hello, " + name);
}

// main.atl
import { greet } from "./lib";
greet("Alice");  // OK
greet(42);       // Error: expected string, found number
```

---

## Build System Integration

### Module Resolution

Build system must:
1. Start from entry point (main.atl)
2. Parse imports
3. Resolve module paths
4. Build dependency graph
5. Detect cycles
6. Compile in topological order

### Entry Point

Default entry: `main.atl` or file specified with `atlas run <file>`

---

## Current Limitations

- File paths only (no package imports like `"std/math"`)
- No conditional imports
- No lazy loading
- REPL doesn't support modules

**Note:** Package manager infrastructure exists (`atlas-package/` crate) but CLI integration is pending. See `ROADMAP.md`.

---

## Error Codes

- `AT1013` - Circular dependency
- `AT1014` - Module not found
- `AT1015` - Import from non-existent export
- `AT1016` - Duplicate export name
- `AT1017` - Import/export in REPL mode

---

## Design Rationale

### Explicit Exports

- **Pro:** Clear API surface
- **Pro:** Prevents accidental exposure
- **Con:** More verbose than wildcard exports

**Decision:** Clarity over brevity (AI-friendly)

### Static Imports

- **Pro:** Analyzable dependency graph
- **Pro:** Fast resolution (compile-time)
- **Con:** No conditional loading

**Decision:** Simplicity and speed over flexibility

### Reject Cycles

- **Pro:** Deterministic initialization
- **Pro:** Easier to reason about
- **Con:** Requires refactoring circular code

**Decision:** Reliability over convenience

---

## See Also

- **Implementation:** `docs/implementation/06-module-system.md`
- **Design:** `docs/design/modules.md`
- **Blocker:** `phases/blockers/blocker-04-*.md`
