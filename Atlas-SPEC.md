# Atlas Language Spec (Draft v0.1)

## Goals
- Typed, strict (no implicit any).
- REPL-first experience with fast feedback.
- Compiled path for speed (bytecode VM).
- Cross-platform (macOS + Windows + Linux).
- Future-friendly for embedding (runtime as a library).
- Cohesive “gold features” from other languages without becoming a clone.

## Advanced Features Under Research
These features require careful design and research before implementation:
- Advanced type features (unions, intersections) - Complexity vs utility tradeoffs
- Async/await - What's the most explicit async model for AI?
- JIT/native codegen - Performance benefits vs maintenance complexity
- Concurrency primitives - Which concurrency model aligns with AI-first principles?

**These will be added when the design is right, not on a timeline.**

## File Format
- Source files use `.atl` extension.

## Lexical Structure
- Whitespace is insignificant except to separate tokens.
- Newlines are statement separators in REPL only; in files, semicolons terminate simple statements and braces delimit blocks.
- Single-line comments: `// ...`
- Multi-line comments: `/* ... */`

## Keywords
- `let`, `var`, `fn`, `if`, `else`, `while`, `for`, `return`, `break`, `continue`, `true`, `false`, `null`
- `match` - Pattern matching (v0.2+)
- `import`, `export` - Module system (v0.2+)
- `from`, `as` - Import keywords (v0.2+)

## Types
- Primitive: `number`, `string`, `bool`, `void`, `null`
- Arrays: `T[]` or `Array<T>`
- Function: `(T1, T2) -> T3`
- JSON: `json` (isolated dynamic type for JSON interop, v0.2+)
- Generic: `Type<T1, T2, ...>` (v0.2+)
- Built-in generics: `Option<T>`, `Result<T, E>` (v0.2+)

### Function Types
Functions are first-class values that can be stored in variables, passed as arguments, and returned from functions.

**Syntax:**
```atlas
// Function type with one parameter
(number) -> bool

// Function type with multiple parameters
(number, string) -> number

// Function type with no parameters
() -> void

// Nested function types
((number) -> bool) -> string

// Function type with array parameters/returns
(number[]) -> string[]
```

**Examples:**
```atlas
// Store function in variable
fn double(x: number) -> number { return x * 2; }
let f = double;
f(5);  // 10

// Pass function as argument
fn apply(fn_param: (number) -> number, x: number) -> number {
    return fn_param(x);
}
apply(double, 5);  // 10

// Return function from function
fn getDouble() -> (number) -> number {
    return double;
}
let g = getDouble();
g(5);  // 10
```

**Limitations (v0.2):**
- No anonymous function syntax: `fn(x) { ... }` (planned for v0.3+)
- No closure capture: Functions can only reference globals (planned for v0.3+)
- All function values must be named functions

### JSON Type (v0.2+)

The `json` type is an **isolated dynamic type** specifically for JSON interop. It is the **only exception** to Atlas's strict typing.

**Type declaration:**
```atlas
let data: json = /* json value from API or parser */;
```

**Key features:**
- **Natural indexing:** Supports both string keys (objects) and number indices (arrays)
- **Safe null semantics:** Missing keys/invalid indices return `null` instead of errors
- **Type isolation:** Cannot assign `json` to other types without explicit extraction
- **Structural equality:** JSON values compare by content, not reference

**Indexing:**
```atlas
// Object indexing with string keys
let name: json = data["user"]["name"];

// Array indexing with number indices
let first: json = items[0];

// Mixed indexing
let value: json = data["users"][0]["email"];
```

**Type safety:**
- `json` values can only be assigned to `json`-typed variables
- Cannot use `json` in expressions: `data + 1` is a type error
- Extraction methods required for type conversion (planned for Phase 4: JSON API)

**Limitations (v0.2):**
- No JSON literal syntax yet (planned for Phase 4: JSON API)
- No extraction methods yet (`.as_string()`, `.as_number()`, etc.)
- JSON values can only be created from Rust code or parsed from strings
- Type checking enforces isolation but extraction API not yet implemented

### Generic Types (v0.2+)

Generic types enable parameterized types for reusable, type-safe code.

**Syntax:**
```atlas
// Generic type application
Type<T1, T2, ...>

// Generic function declaration
fn functionName<T, E>(param: T) -> Result<T, E> {
    // body
}
```

**Examples:**
```atlas
// Generic function with inference
fn identity<T>(x: T) -> T {
    return x;
}

let num = identity(42);        // T inferred as number
let str = identity("hello");   // T inferred as string

// Explicit type arguments
let result = identity<number>(42);

// Multiple type parameters
fn pair<A, B>(first: A, second: B) -> Result<A, B> {
    return Ok([first, second]);
}
```

**Built-in Generic Types:**

`Option<T>` - Represents optional values:
```atlas
let some: Option<number> = Some(42);
let none: Option<number> = None;
```

`Result<T, E>` - Represents success or failure:
```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

**Array syntax sugar:**
```atlas
let arr1: number[] = [1, 2, 3];        // Sugar for Array<number>
let arr2: Array<number> = [1, 2, 3];  // Explicit generic form
```

**Semantics:**
- Monomorphization: Generates specialized code per type instantiation
- Type inference: Arguments infer type parameters when possible
- Type parameters: Lexically scoped to function declaration
- No constraints in v0.2 (all type parameters unbounded)

**Limitations (v0.2):**
- No user-defined generic types (only built-in: Option, Result, Array)
- No type parameter constraints/bounds
- No variance (all type parameters invariant)
- No higher-kinded types

**See:** `docs/design/generics.md` for complete design

### Pattern Matching (v0.2+)

Pattern matching enables destructuring and conditional logic based on value structure. Essential for ergonomic `Result<T,E>` and `Option<T>` handling.

**Syntax:**
```atlas
match expression {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression
}
```

**Examples:**
```atlas
// Result handling
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) { return Err("division by zero"); }
    return Ok(a / b);
}

match divide(10, 2) {
    Ok(value) => print("Result: " + str(value)),
    Err(error) => print("Error: " + error)
}

// Option handling
match find([1, 2, 3], 2) {
    Some(index) => print("Found at " + str(index)),
    None => print("Not found")
}

// Literal patterns
match x {
    0 => "zero",
    1 => "one",
    _ => "many"
}

// Nested patterns
match result {
    Ok(Some(value)) => process(value),
    Ok(None) => use_default(),
    Err(error) => handle_error(error)
}
```

**Pattern Types:**
- Literal: `42`, `"hello"`, `true`, `false`, `null`
- Wildcard: `_` (matches anything, binds nothing)
- Variable: `value` (matches anything, binds to name)
- Constructor: `Ok(value)`, `Err(error)`, `Some(x)`, `None`
- Array: `[]`, `[x]`, `[x, y]` (fixed-length only in v0.2)

**Semantics:**
- Match is an expression (has a type/value)
- Exhaustiveness checking required (must handle all cases)
- All arms must return compatible types
- First-match-wins evaluation order
- Variable bindings scoped to arm expression

**Exhaustiveness:**
```atlas
// ✅ Exhaustive - all constructors covered
match result {
    Ok(value) => value,
    Err(error) => 0
}

// ❌ Non-exhaustive - compiler error
match result {
    Ok(value) => value
    // Missing: Err case
}

// ✅ Wildcard makes it exhaustive
match x {
    0 => "zero",
    _ => "non-zero"
}
```

**Limitations (v0.2):**
- No guard clauses (`pattern if condition`)
- No OR patterns (`0 | 1 | 2`)
- No rest patterns in arrays (`[first, ...rest]`)
- No struct patterns (no user-defined structs yet)

**See:** `docs/design/pattern-matching.md` for complete design

### Module System (v0.2+)

Module system enables multi-file programs with explicit imports and exports.

**File extensions:** `.atl`

**Export syntax:**
```atlas
// Export functions
export fn add(a: number, b: number) -> number {
    return a + b;
}

// Export variables
export let PI = 3.14159;
export var counter = 0;
```

**Import syntax:**
```atlas
// Named imports
import { add, subtract } from "./math";

// Namespace import
import * as math from "./math";
// Usage: math.add(2, 3)
```

**Module paths:**
```atlas
import { x } from "./sibling";        // Relative path
import { y } from "../parent";        // Parent directory
import { z } from "/src/utils";      // Absolute from project root
```

**Semantics:**
- Single evaluation: Each module executed exactly once
- Caching: Module exports cached by absolute path
- Circular dependencies: Detected and rejected (compile error)
- Exports only: Non-exported items are module-private
- Initialization order: Topological sort by dependencies

**Examples:**
```atlas
// math.atl
export fn add(a: number, b: number) -> number {
    return a + b;
}
export let PI = 3.14159;

// main.atl
import { add, PI } from "./math";
let result = add(2, 3);
print(str(PI));
```

**Namespace imports:**
```atlas
import * as math from "./math";
let sum = math.add(10, 20);
print(str(math.PI));
```

**Limitations (v0.2):**
- No default exports (`export default`)
- No re-exports (`export { x } from "./mod"`)
- No dynamic imports (all imports top-level)
- No package imports (file paths only)
- No type-only imports

**See:** `docs/design/modules.md` for complete design

### Typing Rules
- `let` is immutable, `var` is mutable.
- No implicit `any`.
- Function params and return types must be explicit.
- Local variables can be inferred from initializer.
- `null` is only assignable to `null` (no implicit nullable).
- Conditionals require `bool` (no truthy/falsey coercion).
- `number` is a 64-bit floating-point value (IEEE 754).
- `NaN` and `Infinity` results are runtime errors (`AT0007`).
- `==` and `!=` require both operands have the same type; otherwise it's a type error.
- `+` is allowed for `number + number` and `string + string` only.
- `<`, `<=`, `>`, `>=` are only valid for `number`.
- Array indexing requires a `number` index; non-integer indices are runtime errors.
- `&&` and `||` are short-circuiting.

## Semantics
- Lexical scoping with block scope for `let` and `var`.
- Shadowing is allowed in nested scopes.
- Redeclaring a name in the same scope is a compile-time error.
- `var` allows reassignment; `let` does not.
- Function parameters are immutable within the function body.
- `break` and `continue` are only valid within loops.
- `return` is only valid inside functions.
- Top-level statements execute in order.
- Functions are declared at top-level only (no nested function declarations in v0.1).
- Top-level function declarations are hoisted (can be called before definition).
- Variables must be declared before use (no forward reference).
- `for` initializer variables are scoped to the loop body.
- Compound assignment (`+=`, `-=`, etc.) is only valid on mutable variables (`var`).
- Increment/decrement (`++`, `--`) is only valid on mutable variables (`var`).
- Pre-increment/decrement returns the new value; post-increment/decrement returns the old value.
- Increment/decrement operators are statements, not expressions (cannot be used in larger expressions).

## Runtime Model
- Value representation and memory model are defined in `docs/runtime.md`.
- v0.1 uses reference counting (no GC).
- Strings are immutable, arrays are mutable and reference-counted.
- Strings are UTF-8.
## Value Model Reference
- Detailed value model: `docs/value-model.md`.

## Literals
- Number: `123`, `3.14`, `1e10`, `1.5e-3`, `1e874` (supports scientific notation with arbitrary exponents)
  - Integer: `42`, `0`, `-5`
  - Decimal: `3.14`, `0.5`, `-2.7`
  - Scientific: `1e10` (10 billion), `1.5e-3` (0.0015), `6.022e23` (Avogadro's number)
  - All numbers are 64-bit floating-point (IEEE 754)
- String: `"hello"`
- Boolean: `true`, `false`
- Null: `null`
- Array: `[expr1, expr2, ...]` (all elements must have the same type)

### String Escapes
- `\"`, `\\`, `\n`, `\r`, `\t`

## Expressions
- Arithmetic: `+ - * / %`
- Comparison: `== != < <= > >=`
- Logical: `&& || !`
- Unary: `-expr` (negation), `!expr` (logical not)
- Increment/Decrement (statements, not expressions):
  - Pre-increment: `++var` (increments, returns new value)
  - Pre-decrement: `--var` (decrements, returns new value)
  - Post-increment: `var++` (increments, returns old value)
  - Post-decrement: `var--` (decrements, returns old value)
  - Note: Only valid as standalone statements, not within expressions
- Grouping: `(expr)`
- Call: `fnName(arg1, arg2)`
- Index: `arr[i]`

### Array Semantics
- Array element types are invariant and homogeneous.
- `[]` is not allowed without a type context (no implicit empty array).
- Arrays are mutable; element assignment is supported.
- Array equality is reference identity (no deep equality in v0.1).
- Array indices must be whole numbers (non-integers are runtime error `AT0103`).
  - `1.0` is valid; fractional values are not.
  - Negative indices are out-of-bounds (`AT0006`).

## Statements
- Variable declaration:
  - `let name: type = expr;`
  - `var name: type = expr;`
  - `let name = expr;` (type inferred)
- Assignment:
  - Simple: `name = expr;`
  - Array element: `arr[i] = expr;`
  - Compound assignment (mutable variables only):
    - `var += expr;` (addition)
    - `var -= expr;` (subtraction)
    - `var *= expr;` (multiplication)
    - `var /= expr;` (division)
    - `var %= expr;` (modulo)
- Increment/Decrement (mutable variables only):
  - Pre-increment: `++var;` (increments by 1, returns new value)
  - Pre-decrement: `--var;` (decrements by 1, returns new value)
  - Post-increment: `var++;` (increments by 1, returns old value)
  - Post-decrement: `var--;` (decrements by 1, returns old value)
- Function declaration:
  - `fn add(a: number, b: number) -> number { return a + b; }`
- If:
  - `if (cond) { ... } else { ... }`
- While:
  - `while (cond) { ... }`
- For (simple):
  - `for (let i = 0; i < 10; i = i + 1) { ... }`
  - `for (var i = 0; i < 10; i++) { ... }` (with increment operator)
- Return:
  - `return expr;`

## Grammar (EBNF)
```ebnf
program        = { module_item } ;
module_item    = export_decl | import_decl | decl_or_stmt ;           (* v0.2+ *)
decl_or_stmt   = fn_decl | stmt ;

(* Module system - v0.2+ *)
export_decl    = "export" ( fn_decl | var_decl ) ;
import_decl    = "import" import_clause "from" string ";" ;
import_clause  = named_imports | namespace_import ;
named_imports  = "{" import_specifiers "}" ;
import_specifiers = import_specifier { "," import_specifier } ;
import_specifier  = ident ;
namespace_import  = "*" "as" ident ;

fn_decl        = "fn" ident [ type_params ] "(" [ params ] ")" "->" type block ;
type_params    = "<" type_param_list ">" ;                           (* v0.2+ *)
type_param_list = ident { "," ident } ;                              (* v0.2+ *)
params         = param { "," param } ;
param          = ident ":" type ;

stmt           = var_decl | assign_stmt | compound_assign_stmt | increment_stmt
               | decrement_stmt | if_stmt | while_stmt | for_stmt
               | return_stmt | break_stmt | continue_stmt | expr_stmt ;

var_decl       = ("let" | "var") ident [ ":" type ] "=" expr ";" ;
assign_stmt    = assign_target "=" expr ";" ;
assign_expr    = assign_target "=" expr ;
assign_target  = ident { "[" expr "]" } ;
compound_assign_stmt = ident compound_op expr ";" ;
compound_op    = "+=" | "-=" | "*=" | "/=" | "%=" ;
increment_stmt = ( "++" ident | ident "++" ) ";" ;
decrement_stmt = ( "--" ident | ident "--" ) ";" ;
if_stmt        = "if" "(" expr ")" block [ "else" block ] ;
while_stmt     = "while" "(" expr ")" block ;
for_stmt       = "for" "(" [ for_init ] ";" [ expr ] ";" [ for_step ] ")" block ;
for_init       = var_decl_no_semi | assign_expr ;
for_step       = assign_expr | compound_assign_expr | increment_expr | decrement_expr ;
compound_assign_expr = ident compound_op expr ;
increment_expr = "++" ident | ident "++" ;
decrement_expr = "--" ident | ident "--" ;
var_decl_no_semi = ("let" | "var") ident [ ":" type ] "=" expr ;
return_stmt    = "return" [ expr ] ";" ;
break_stmt     = "break" ";" ;
continue_stmt  = "continue" ";" ;
expr_stmt      = expr ";" ;

block          = "{" { stmt } "}" ;

expr           = logic_or ;
logic_or       = logic_and { "||" logic_and } ;
logic_and      = equality { "&&" equality } ;
equality       = comparison { ("==" | "!=") comparison } ;
comparison     = term { ("<" | "<=" | ">" | ">=") term } ;
term           = factor { ("+" | "-") factor } ;
factor         = unary { ("*" | "/" | "%") unary } ;
unary          = ("!" | "-") unary | call ;
call           = primary { [ type_args ] "(" [ args ] ")" | "[" expr "]" } ;  (* v0.2+: type_args *)
type_args      = "<" type_arg_list ">" ;                             (* v0.2+ *)
type_arg_list  = type { "," type } ;                                 (* v0.2+ *)
args           = expr { "," expr } ;
array_literal  = "[" [ args ] "]" ;
primary        = number | string | "true" | "false" | "null" | ident
               | array_literal | "(" expr ")" | match_expr ;           (* v0.2+: match_expr *)

(* Pattern matching - v0.2+ *)
match_expr     = "match" expr "{" match_arms "}" ;
match_arms     = match_arm { "," match_arm } [ "," ] ;
match_arm      = pattern "=>" expr ;
pattern        = literal_pattern | wildcard_pattern | variable_pattern
               | constructor_pattern | array_pattern ;
literal_pattern = number | string | "true" | "false" | "null" ;
wildcard_pattern = "_" ;
variable_pattern = ident ;
constructor_pattern = ident "(" [ pattern_list ] ")" ;
array_pattern  = "[" [ pattern_list ] "]" ;
pattern_list   = pattern { "," pattern } ;

type           = primary_type [ "[]" ] | generic_type | function_type ;  (* v0.2+ *)
primary_type   = "number" | "string" | "bool" | "void" | "null" ;
generic_type   = ident "<" type_arg_list ">" ;                       (* v0.2+ *)
function_type  = "(" [ type_list ] ")" "->" type ;
type_list      = type { "," type } ;
ident          = letter { letter | digit | "_" } ;
number         = digit { digit } [ "." digit { digit } ] [ ("e" | "E") ["+" | "-"] digit { digit } ] ;
string         = "\"" { char } "\"" ;
```

## REPL Rules
- Accepts single expressions without semicolons and prints the result.
- Accepts multi-line blocks; input ends when braces are balanced.
- Keeps global scope and declarations across inputs.
- Type-checks before evaluation; type errors are reported without executing.
- Implementation guidance is in `docs/repl.md` (core/UI split).

## Diagnostics & Testing References
- Diagnostics schema and formats: `docs/diagnostics.md`
- Testing conventions: `docs/testing.md`
- Versioning policy: `docs/versioning.md`
- AI-first principles: `docs/ai-principles.md`
- Decision log: `docs/decision-log.md`
- Coverage matrix: `docs/coverage-matrix.md`
- Phase gates: `docs/phase-gates.md`

## Standard Library (v0.1)
- `print(value: string|number|bool|null)` -> void
- `len(value: string|T[])` -> number
- `str(value: number|bool|null)` -> string
  - `len(string)` returns Unicode scalar count.

## Prelude (v0.1)
- Prelude built-ins are defined in `docs/prelude.md`.

## Error Model
- Compile-time errors: syntax errors, type errors, invalid control flow.
- Runtime errors: divide by zero, invalid numeric result, out-of-bounds array access, invalid index, null usage.
- Errors include: file name, line, column, length, and a short error code.
- Errors are emitted in both human-readable and machine-readable JSON formats.
- REPL: runtime errors do not terminate the session.

### Diagnostic Format (Human)
```
error[AT0001]: Type mismatch
  --> path/to/file.atl:12:9
   |
12 | let x: number = "hello";
   |         ^^^^^ expected number, found string
   |
help: convert the value to number or change the variable type
```

### Diagnostic Format (JSON)
```json
{
  "diag_version": 1,
  "level": "error",
  "code": "AT0001",
  "message": "Type mismatch",
  "file": "path/to/file.atl",
  "line": 12,
  "column": 9,
  "length": 5,
  "snippet": "let x: number = \"hello\";",
  "label": "expected number, found string",
  "notes": [
    "inferred type of \"hello\" is string"
  ],
  "related": [
    {
      "file": "path/to/file.atl",
      "line": 10,
      "column": 5,
      "length": 6,
      "message": "variable declared here"
    }
  ],
  "help": "convert the value to number or change the variable type"
}
```

### Error Codes
- `AT0001` Type mismatch
- `AT0002` Unknown symbol
- `AT0003` Invalid assignment
- `AT0004` Missing return
- `AT0005` Divide by zero
- `AT0006` Out-of-bounds access
- `AT0007` Invalid numeric result (NaN/Infinity)
- `AT0102` Invalid stdlib argument
- `AT0103` Invalid index (non-integer)
- `AT1000` Syntax error
- `AT1001` Invalid token
- `AT1002` Unterminated string
- `AT1003` Invalid escape sequence
- `AT1004` Unterminated multi-line comment
- `AT1010` Illegal break/continue
- `AT1011` Illegal return
- `AT1012` Illegal prelude shadowing
- `AT2001` Unused variable (warning)
- `AT2002` Unreachable code (warning)
- `AT2003` Redeclaration

### Runtime Stack Trace Format
```
runtime error[AT0101]: Divide by zero
  --> path/to/file.atl:27:13
   |
27 | let x = 10 / 0;
   |             ^
stack trace:
  at divide(a: number, b: number) path/to/file.atl:27:13
  at main() path/to/file.atl:4:1
```

### Warnings
- Warnings are non-fatal and use `warning[CODE]` format.
- Example warnings: unused variable, unreachable code.
- Warnings are also emitted as JSON diagnostics with `"level": "warning"`.

### Diagnostic Policy
- Stop after 25 compile-time errors to avoid flooding.
- Continue emitting warnings even if errors exist.
- In REPL, report the first error for the current input and continue.
- Errors are emitted before warnings for the same input.

## CLI Behavior (v0.1)
- `atlas repl` starts a REPL session.
- `atlas run path/to/file.atl` parses, type-checks, and runs the file.
- `atlas build path/to/file.atl` emits bytecode (`.atb`, format in `docs/bytecode-format.md`).
- `atlas ast path/to/file.atl --json` emits AST JSON.
- `atlas typecheck path/to/file.atl --json` emits typecheck JSON.

## Modules (not in v0.1)
- No `import` or `module` support.

## REPL Behavior
- Keeps state between inputs.
- Allows single expressions without semicolons.
- Evaluates and prints expression results automatically.

## Errors
- Type errors are compile-time errors.
- Runtime errors: divide by zero, out-of-bounds, null usage.

## Example
```atlas
fn add(a: number, b: number) -> number {
  return a + b;
}

let x = add(1, 2);
print(x);
```

## Bytecode VM (v0.1 outline)
- Stack-based VM.
- Instructions: `PUSH_CONST`, `LOAD_LOCAL`, `STORE_LOCAL`, `ADD`, `SUB`, `MUL`, `DIV`, `JMP`, `JMP_IF_FALSE`, `CALL`, `RET`.
- Debug info mapping is defined in `docs/debug-info.md`.

## Module System (Design Sketch)
- Not implemented in v0.1.
- Design defined in `docs/modules.md`.

## Compiler IR
- v0.1 compiles AST directly to bytecode.
- IR design notes in `docs/ir.md`.

## Milestones
1. Lexer + parser
2. AST + binder
3. Type checker
4. Interpreter (REPL)
5. Bytecode compiler
6. VM
7. Stdlib
8. CLI tooling

## Gold Features Roadmap
### v0.1 (Core)
- TypeScript-style strict typing with explicit annotations for function params/returns.
- Python-style readability and minimal ceremony.
- REPL-first workflow with clear error reporting.
- Small, coherent standard library (strings, files, JSON, time).

### v1.0 (Stability)
- Module system with explicit `import`.
- Packaging layout and deterministic builds.
- Standard library expansion (collections, path, io).

### v1.1+ (Differentiators)
- Go-style lightweight concurrency: `spawn`, `chan<T>`, `select`.
- Optional `option<T>` type for explicit null handling.
- Embedding API so host apps can run Atlas scripts safely.

## Test Plan (v0.1)
- Lexer
  - Tokenize keywords, identifiers, numbers, strings, operators.
  - Handle comments and whitespace.
- Parser
  - Parse declarations, statements, expressions, and precedence.
- Type Checker
  - Reject assigning wrong types, missing returns, invalid operations.
- Interpreter
  - Evaluate arithmetic, conditionals, loops, and function calls.
- VM
  - Bytecode roundtrip for simple programs matches interpreter output.

### Example Programs
1. Hello
```atlas
print(\"Hello, Atlas\");
```

2. Arithmetic
```atlas
let x = 2 + 3 * 4;
print(x);
```

3. Functions
```atlas
fn add(a: number, b: number) -> number {
  return a + b;
}
print(add(4, 5));
```

4. Control Flow
```atlas
let sum = 0;
for (let i = 0; i < 5; i = i + 1) {
  sum = sum + i;
}
print(sum);
```
