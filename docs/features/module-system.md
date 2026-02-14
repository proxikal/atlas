# Module System

**Status:** ✅ Implemented (v0.2 BLOCKER 04-06)
**Specification:** docs/specification/modules.md
**Implementation:** crates/atlas-runtime/src/{module_loader.rs, module_executor.rs, resolver/}

---

## Overview

Atlas modules enable code organization through import/export with static type checking and dependency resolution.

**Key features:**
- Static imports (evaluated at load time)
- Named exports with type information
- Relative and absolute paths
- Circular dependency detection
- Type-safe cross-module references

---

## Syntax

### Export Declarations

```atlas
// Export variables
export let API_KEY = "secret";
export var counter = 0;

// Export functions
export fn add(a: number, b: number) -> number {
    return a + b;
}

// Export type-annotated values
export let config: json = parseConfig();
```

**Rules:**
- `export` keyword before declaration
- Can export: variables (`let`/`var`), functions
- Exports are immutable from importing module's perspective
- Each name can only be exported once per module

### Import Declarations

```atlas
// Named imports
import { add, subtract } from "./math.atl";
import { config } from "../config.atl";

// Multiple imports
import { fn1, fn2, fn3 } from "./utils.atl";

// Absolute paths (from project root)
import { helper } from "/src/lib/helpers.atl";
```

**Rules:**
- Imports must be at top of file (before any statements)
- Named imports only (no default/namespace imports in v0.2)
- Source path must be string literal
- Imported names are immutable (cannot reassign)

---

## Module Resolution

### Path Resolution Rules

**Relative paths** (start with `./` or `../`):
- Resolved relative to importing file
- `./file.atl` → same directory
- `../file.atl` → parent directory
- `../../lib/util.atl` → grandparent/lib directory

**Absolute paths** (start with `/`):
- Resolved from project root
- `/src/main.atl` → project_root/src/main.atl

**File extension:**
- `.atl` extension required
- No implicit extensions added

### Example Directory Structure

```
project/
├── src/
│   ├── main.atl
│   ├── utils.atl
│   └── lib/
│       └── math.atl
└── tests/
    └── test.atl
```

```atlas
// In src/main.atl:
import { helper } from "./utils.atl";      // ✅ Same dir
import { add } from "./lib/math.atl";      // ✅ Subdirectory
import { add } from "/src/lib/math.atl";   // ✅ Absolute

// In tests/test.atl:
import { helper } from "../src/utils.atl"; // ✅ Parent dir
```

---

## Type System Integration

### Export Type Information

When a module exports a symbol, its type is preserved:

```atlas
// math.atl
export fn add(a: number, b: number) -> number {
    return a + b;
}

// main.atl
import { add } from "./math.atl";
let result: number = add(1, 2);  // ✅ Type-checked
let bad: string = add(1, 2);     // ❌ AT3001: Type mismatch
```

### Cross-Module Type Checking

The type checker validates:
1. Exported symbol exists in target module
2. Imported symbol's type matches usage
3. Function signatures are preserved
4. Generic types are maintained

---

## Dependency Resolution

### Loading Order

Modules are loaded in **topological order** (dependencies first):

```atlas
// a.atl
import { utilB } from "./b.atl";
export let utilA = utilB + 1;

// b.atl
import { utilC } from "./c.atl";
export let utilB = utilC * 2;

// c.atl
export let utilC = 10;
```

**Load order:** c.atl → b.atl → a.atl (dependencies before dependents)

### Circular Dependency Detection

Circular imports are detected and rejected:

```atlas
// a.atl
import { b } from "./b.atl";  // ❌ AT5003: Circular dependency
export let a = 1;

// b.atl
import { a } from "./a.atl";  // ❌ AT5003: Circular dependency
export let b = 2;
```

**Error:** `AT5003: Circular dependency detected between modules`

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| AT5001 | Invalid module path | Path doesn't start with `./`, `../`, or `/` |
| AT5002 | Module not found | File doesn't exist at resolved path |
| AT5003 | Circular dependency | Module dependency cycle detected |
| AT5004 | Export not found | Attempting to export undeclared symbol |
| AT5005 | Import resolution failed | Cannot resolve import path |
| AT5006 | Module not exported | Imported symbol not exported from module |
| AT5007 | Namespace import unsupported | `import *` not supported in v0.2 |
| AT5008 | Duplicate export | Symbol exported multiple times |

---

## Limitations (v0.2)

**Not supported:**
- ❌ Default exports (`export default`)
- ❌ Namespace imports (`import * as ns from`)
- ❌ Re-exports (`export { x } from "./other.atl"`)
- ❌ Dynamic imports (`import(path)`)
- ❌ Export aliasing (`export { x as y }`)
- ❌ Import aliasing (`import { x as y } from`)

**Planned for future versions** (v0.3+)

---

## Best Practices

1. **One module per file** - Keep modules focused and single-purpose
2. **Explicit exports** - Only export what's needed for external use
3. **Avoid deep nesting** - Prefer flat module structures
4. **Use absolute paths for library code** - Makes refactoring easier
5. **Group related imports** - Organize imports by source/purpose

---

## Examples

### Basic Module Usage

```atlas
// math.atl - Utility module
export fn add(a: number, b: number) -> number {
    return a + b;
}

export fn multiply(a: number, b: number) -> number {
    return a * b;
}

let internal = 42;  // Not exported, private to module

// main.atl - Application entry
import { add, multiply } from "./math.atl";

let sum = add(5, 3);      // ✅ 8
let product = multiply(4, 2);  // ✅ 8
// let x = internal;      // ❌ AT0002: Undefined symbol
```

### Configuration Module

```atlas
// config.atl
export let DEBUG = true;
export let API_URL = "https://api.example.com";
export let MAX_RETRIES = 3;

// app.atl
import { DEBUG, API_URL, MAX_RETRIES } from "./config.atl";

if (DEBUG) {
    print("Debug mode enabled");
}
```

### Library Structure

```
lib/
├── index.atl          # Main entry point
├── utils/
│   ├── string.atl     # String utilities
│   └── array.atl      # Array utilities
└── api/
    └── client.atl     # API client

```

```atlas
// lib/index.atl
import { formatString } from "./utils/string.atl";
import { sortArray } from "./utils/array.atl";

export fn processData(data: string[]) -> string[] {
    let formatted = map(data, formatString);
    return sortArray(formatted);
}
```

---

## Implementation Notes

**Architecture:**
- `ModuleLoader`: Loads and caches modules, builds dependency graph
- `ModuleResolver`: Resolves relative/absolute paths
- `ModuleExecutor`: Executes modules in topological order
- `Binder`: Links imports to exports with type information

**Performance:**
- Modules are cached after first load
- Dependency graph computed once at load time
- Topological sort ensures correct evaluation order

---

## See Also

- **Specification:** `docs/specification/modules.md`
- **Implementation Phases:**
  - `phases/foundation/BLOCKER-04-A-module-resolution.md`
  - `phases/foundation/BLOCKER-04-B-module-loading.md`
  - `phases/foundation/BLOCKER-04-C-cross-module-type-checking.md`
- **Related Features:**
  - Type system: `docs/specification/types.md`
  - Import/export in REPL: `docs/specification/repl.md`
