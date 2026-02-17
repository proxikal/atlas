# Atlas Codebase Patterns

**Purpose:** Documented patterns from actual Atlas codebase for AI agents.

---

## Atlas Grammar Quick Reference (from parser)

**ALWAYS check this before writing Atlas code or tests.**

```
// Control flow — ALL require parentheses around condition
if (cond) { ... }
if (cond) { ... } else { ... }
while (cond) { ... }
for (init; cond; step) { ... }     // C-style: for (let i = 0; i < n; i++)
for item in iterable { ... }       // For-in: NO parens

// Functions
fn name(param: type, ...) -> ReturnType { ... }
fn name(param: type) { ... }       // No return type → parser stores "null" (not "void")

// Type syntax
number, string, bool, null         // Named types
Type<T1, T2>                       // Generics: Result<number, string>
type[]                              // Array type: number[]
(T1, T2) -> T3                     // Function type (parens, NOT fn keyword)

// Default return type
// When no `->` specified, parser sets: TypeRef::Named("null", Span::dummy())
// Formatter should OMIT return type when it equals "null"

// Variable declarations
let x = 5;                          // Immutable
var x = 5;                          // Mutable
let x: number = 5;                  // With type annotation

// Operators
x++;  x--;  x += 1;  x -= 1;      // Compound assignment
x = expr;                           // Assignment
arr[0] = expr;                      // Index assignment

// Match
match expr { pattern => body, ... }

// Modules
import { a, b } from "./path";
import * as ns from "./path";
export fn name() { ... }
export let x = 5;

// Comments (lexer skips by default, tokenize_with_comments() preserves them)
// line comment
/* block comment */
/// doc comment (3 slashes exactly, //// is NOT doc)
```

---

## Collection Types (Shared Mutable State)

**Pattern:** `Arc<Mutex<X>>` for all collection types (migrated from `Rc<RefCell<>>` in phase-18)

```rust
// In value.rs (actual current code)
pub enum Value {
    Array(Arc<Mutex<Vec<Value>>>),
    HashMap(Arc<Mutex<AtlasHashMap>>),
    HashSet(Arc<Mutex<AtlasHashSet>>),
    Queue(Arc<Mutex<AtlasQueue>>),
    Stack(Arc<Mutex<AtlasStack>>),
    String(Arc<String>),
    // ...
}
```

**Access pattern:** `.lock().unwrap()` — NOT `.borrow()` / `.borrow_mut()`

```rust
// Correct — read
let guard = arr.lock().unwrap();

// Correct — mutate
let mut guard = map.lock().unwrap();
guard.insert(key, value);
```

**Why Arc<Mutex<>>:** Thread safety required for async/tokio support (phase-18 migration). DR-009.

---

## Intrinsic Pattern (Callback-Based)

**Intrinsics** are runtime functions that need access to execution context (calling functions, managing stack). They're implemented directly in interpreter and VM, not as stdlib functions.

### Location Map

1. **Registration:** `crates/atlas-runtime/src/stdlib/mod.rs`
   - `is_array_intrinsic()` function - list all intrinsic names
2. **Interpreter:** `crates/atlas-runtime/src/interpreter/expr.rs`
   - Match in `eval_expr()` for `Expr::Call` with extern function
   - Implement as `intrinsic_name()` method
3. **VM:** `crates/atlas-runtime/src/vm/mod.rs`
   - Match in `execute_call_intrinsic()` method
   - Implement as `vm_intrinsic_name()` method

### Full Example: HashMap forEach Intrinsic

#### 1. Registration (stdlib/mod.rs)

```rust
/// Check if a function name is an array intrinsic (handled in interpreter/VM)
pub fn is_array_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "map" | "filter" | "forEach"
        // ... other intrinsics
        | "hashMapForEach"  // <-- Add here
        | "hashMapMap"
        | "hashMapFilter"
        // ...
    )
}
```

#### 2. Interpreter Implementation (interpreter/expr.rs)

```rust
// In eval_expr() for Expr::Call with extern function:
match &func_ref.name.as_str() {
    // Array intrinsics
    "map" => return self.intrinsic_map(&args, call.span),
    // ... other intrinsics
    // HashMap intrinsics (callback-based)
    "hashMapForEach" => return self.intrinsic_hashmap_for_each(&args, call.span),
    "hashMapMap" => return self.intrinsic_hashmap_map(&args, call.span),
    "hashMapFilter" => return self.intrinsic_hashmap_filter(&args, call.span),
    _ => {}
}

// Implementation method:
fn intrinsic_hashmap_for_each(
    &mut self,
    args: &[Value],
    span: crate::span::Span,
) -> Result<Value, RuntimeError> {
    // 1. Validate argument count
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "hashMapForEach() expects 2 arguments (map, callback)".to_string(),
            span,
        });
    }

    // 2. Extract and validate map argument
    let map = match &args[0] {
        Value::HashMap(m) => m.lock().unwrap().entries(),  // Get entries snapshot
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "hashMapForEach() first argument must be HashMap".to_string(),
                span,
            })
        }
    };

    // 3. Extract and validate callback argument
    let callback = match &args[1] {
        Value::Function(_) => &args[1],
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "hashMapForEach() second argument must be function".to_string(),
                span,
            })
        }
    };

    // 4. Iterate and call callback
    for (key, value) in map {
        // Call callback with (value, key) arguments
        self.call_value(callback, vec![value, key.to_value()], span)?;
    }

    // 5. Return appropriate value
    Ok(Value::Null)
}
```

#### 3. VM Implementation (vm/mod.rs)

```rust
// In execute_call_intrinsic():
fn execute_call_intrinsic(
    &mut self,
    name: &str,
    args: &[Value],
    span: crate::span::Span,
) -> Result<Value, RuntimeError> {
    match name {
        "map" => self.vm_intrinsic_map(args, span),
        // ... other intrinsics
        // HashMap intrinsics (callback-based)
        "hashMapForEach" => self.vm_intrinsic_hashmap_for_each(args, span),
        "hashMapMap" => self.vm_intrinsic_hashmap_map(args, span),
        "hashMapFilter" => self.vm_intrinsic_hashmap_filter(args, span),
        _ => Err(RuntimeError::UnknownFunction {
            name: name.to_string(),
            span,
        }),
    }
}

// Implementation method (IDENTICAL to interpreter):
fn vm_intrinsic_hashmap_for_each(
    &mut self,
    args: &[Value],
    span: crate::span::Span,
) -> Result<Value, RuntimeError> {
    // 1. Validate argument count
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "hashMapForEach() expects 2 arguments (map, callback)".to_string(),
            span,
        });
    }

    // 2. Extract and validate map argument
    let map = match &args[0] {
        Value::HashMap(m) => m.lock().unwrap().entries(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "hashMapForEach() first argument must be HashMap".to_string(),
                span,
            })
        }
    };

    // 3. Extract callback (VM doesn't need type check - will fail on call)
    let callback = &args[1];

    // 4. Iterate and call callback
    for (key, value) in map {
        // VM uses vm_call_function_value instead of call_value
        self.vm_call_function_value(callback, vec![value, key.to_value()], span)?;
    }

    // 5. Return appropriate value
    Ok(Value::Null)
}
```

### Key Differences: Interpreter vs VM

| Aspect | Interpreter | VM |
|--------|-------------|-----|
| **Callback invoke** | `self.call_value()` | `self.vm_call_function_value()` |
| **Callback validation** | Explicit `match Value::Function` | Implicit (call will error) |
| **Everything else** | IDENTICAL | IDENTICAL |

### Intrinsic Patterns by Return Type

**forEach pattern (side effects only):**
- Iterates collection
- Calls callback for each element
- Returns `Value::Null`
- Example: `hashMapForEach`, `hashSetForEach`, array `forEach`

**map pattern (transform elements):**
- Iterates collection
- Calls callback for each element
- Collects results into new collection
- Returns new collection (same type or array)
- Example: `hashMapMap` (map→map), `hashSetMap` (set→array), array `map`

**filter pattern (selective copy):**
- Iterates collection
- Calls predicate for each element
- Keeps elements where predicate returns truthy
- Returns new collection (same type)
- Example: `hashMapFilter`, `hashSetFilter`, array `filter`

---

## Stdlib Function Pattern (Non-Intrinsic)

**Stdlib functions** don't need execution context. They're called directly via match statements in interpreter/VM.

### Location Map

1. **Registration:** `crates/atlas-runtime/src/stdlib/mod.rs`
   - `is_builtin()` function - list all builtin names
2. **Implementation:**
   - **Module-specific:** `crates/atlas-runtime/src/stdlib/{module}.rs`
   - **Interpreter:** Calls module function from `eval_call()`
   - **VM:** Calls module function from `execute_call_builtin()`

### Example: HashMap.put() (Non-Intrinsic)

#### Registration (stdlib/mod.rs)

```rust
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "print" | "len"
        // ...
        | "hashMapNew" | "hashMapPut" | "hashMapGet"  // <-- Add here
        // ...
    )
}
```

#### Implementation (stdlib/collections/hashmap.rs)

```rust
/// HashMap.put(map, key, value) - Insert/update entry
pub fn hashmap_put(
    map: &Value,
    key: &Value,
    value: Value,
    span: crate::span::Span,
) -> Result<Value, RuntimeError> {
    let hashmap = expect_hashmap(map, span)?;
    let hash_key = HashKey::try_from_value(key.clone(), span)?;
    hashmap.lock().unwrap().insert(hash_key, value.clone());
    Ok(value)
}
```

#### Interpreter Call (interpreter/expr.rs)

```rust
// In eval_call():
"hashMapPut" => {
    if args.len() != 3 {
        return Err(/* ... */);
    }
    crate::stdlib::collections::hashmap::hashmap_put(&args[0], &args[1], args[2].clone(), span)
}
```

#### VM Call (vm/mod.rs)

```rust
// In execute_call_builtin() - IDENTICAL:
"hashMapPut" => {
    if args.len() != 3 {
        return Err(/* ... */);
    }
    crate::stdlib::collections::hashmap::hashmap_put(&args[0], &args[1], args[2].clone(), span)
}
```

---

## Error Pattern

```rust
use crate::value::RuntimeError;

// Type errors
return Err(RuntimeError::TypeError {
    msg: "descriptive message".to_string(),
    span,
});

// Unknown function
return Err(RuntimeError::UnknownFunction {
    name: name.to_string(),
    span,
});

// Index out of bounds
return Err(RuntimeError::IndexOutOfBounds {
    index: idx,
    span,
});
```

**Always include:**
- Descriptive error message
- `span` for error location

---

## Helper Pattern (Type Extraction)

```rust
/// Extract HashMap from Value or error
fn expect_hashmap(
    value: &Value,
    span: crate::span::Span,
) -> Result<Arc<Mutex<AtlasHashMap>>, RuntimeError> {
    match value {
        Value::HashMap(m) => Ok(Arc::clone(m)),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected HashMap, got {}", value.type_name()),
            span,
        }),
    }
}
```

**Pattern:** Use `expect_X` helpers for type checking with good error messages.

---

## Test Harness Pattern

```rust
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;

fn run(code: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut interpreter = Interpreter::new();
    let security = SecurityContext::allow_all();
    match interpreter.eval(&ast, &security) {
        Ok(val) => Ok(format!("{:?}", val)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[test]
fn test_something() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "key", 42);
        hashMapGet(map, "key")
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("42.0"));
}
```

---

## Summary

**When to use intrinsic:**
- Need to call callbacks (map, filter, forEach)
- Need execution context
- Implement in both interpreter AND VM

**When to use stdlib function:**
- Simple operations (put, get, add, remove)
- No callback execution
- Implement in module, call from both engines

**Parity requirement:**
- Both engines MUST produce identical results
- Test both paths using integration tests
