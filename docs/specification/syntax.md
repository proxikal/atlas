# Atlas Syntax Specification

**Purpose:** Define Atlas grammar, keywords, and syntax rules.
**Status:** Living document — reflects current implementation.

---

## File Format

- Source files use `.atl` extension
- UTF-8 encoding required
- Newline-agnostic (LF or CRLF)

---

## Lexical Structure

### Whitespace
- Whitespace is insignificant except to separate tokens
- Newlines are statement separators in REPL only
- In files, semicolons terminate simple statements and braces delimit blocks

### Comments

```atlas
// Single-line comment

/*
 * Multi-line comment
 * Can span multiple lines
 */
```

---

## Keywords

### Keywords
`let`, `var`, `fn`, `if`, `else`, `while`, `for`, `return`, `break`, `continue`, `true`, `false`, `null`, `match`, `import`, `export`, `from`, `as`

**Note:** Keywords cannot be used as identifiers

---

## Literals

### Number Literals

All numbers are 64-bit floating-point (IEEE 754)

```atlas
// Integer form
42
0
-5

// Decimal form
3.14
0.5
-2.7

// Scientific notation
1e10        // 10 billion
1.5e-3      // 0.0015
6.022e23    // Avogadro's number
1e874       // Supports arbitrary exponents
```

**Syntax:** `digit { digit } [ "." digit { digit } ] [ ("e" | "E") ["+" | "-"] digit { digit } ]`

### String Literals

```atlas
"hello"
"world"
""  // Empty string
```

#### String Escapes
- `\"` - Double quote
- `\\` - Backslash
- `\n` - Newline
- `\r` - Carriage return
- `\t` - Tab

**Example:**
```atlas
"Line 1\nLine 2"
"She said \"Hello\""
"C:\\Users\\name"
```

### Boolean Literals

```atlas
true
false
```

### Null Literal

```atlas
null
```

### Array Literals

```atlas
[1, 2, 3]           // number[]
["a", "b", "c"]     // string[]
[true, false]       // bool[]
[]                  // Empty array (requires type context)
```

**Rules:**
- All elements must have the same type
- `[]` not allowed without type context (no implicit empty array)
- Trailing commas not allowed

---

## Expressions

### Operator Precedence (highest to lowest)

1. **Primary:** literals, identifiers, grouping `(expr)`
2. **Call/Index:** `fn(args)`, `arr[index]`
3. **Unary:** `-expr`, `!expr`
4. **Multiplicative:** `*`, `/`, `%`
5. **Additive:** `+`, `-`
6. **Comparison:** `<`, `<=`, `>`, `>=`
7. **Equality:** `==`, `!=`
8. **Logical AND:** `&&`
9. **Logical OR:** `||`

### Arithmetic Operators

```atlas
a + b   // Addition (number + number OR string + string)
a - b   // Subtraction (number only)
a * b   // Multiplication (number only)
a / b   // Division (number only)
a % b   // Modulo (number only)
```

**Type rules:**
- `+` allowed for `number + number` and `string + string` only
- `-`, `*`, `/`, `%` allowed for `number` only

### Comparison Operators

```atlas
a == b  // Equality (requires same type)
a != b  // Inequality (requires same type)
a < b   // Less than (number only)
a <= b  // Less than or equal (number only)
a > b   // Greater than (number only)
a >= b  // Greater than or equal (number only)
```

**Type rules:**
- `==`, `!=` require both operands have the same type
- `<`, `<=`, `>`, `>=` valid for `number` only

### Logical Operators

```atlas
a && b  // Logical AND (short-circuits)
a || b  // Logical OR (short-circuits)
!a      // Logical NOT
```

**Type rules:**
- All operands must be `bool`
- `&&` and `||` are short-circuiting

### Unary Operators

```atlas
-expr   // Negation (number only)
!expr   // Logical NOT (bool only)
```

### Increment/Decrement

**Note:** These are **statements only**, not expressions

```atlas
++var   // Pre-increment (increments, returns new value)
--var   // Pre-decrement (decrements, returns new value)
var++   // Post-increment (increments, returns old value)
var--   // Post-decrement (decrements, returns old value)
```

**Rules:**
- Only valid as standalone statements
- Cannot be used within expressions
- Variable must be mutable (`var`, not `let`)

### Grouping

```atlas
(expr)  // Explicit precedence control
```

### Function Calls

```atlas
fnName(arg1, arg2, arg3)
fnName()  // No arguments

// With type arguments identity<number>(42)
```

### Array Indexing

```atlas
arr[0]      // Access first element
arr[i + 1]  // Index can be any number expression
```

**Rules:**
- Index must be a `number`
- Non-integer indices are runtime errors (`AT0103`)
- Negative indices are out-of-bounds (`AT0006`)
- `1.0` is valid; fractional values (e.g., `1.5`) are not

### JSON Indexing 
```atlas
data["user"]        // String key (object)
data[0]             // Number index (array)
data["user"]["name"] // Chained indexing
```

**Rules:**
- Accepts `string` or `number` index
- Returns `json` type
- Missing keys/invalid indices return `null` (safe)

### Array Semantics

- Array element types are invariant and homogeneous
- Arrays are mutable; element assignment supported
- Array equality is reference identity (not deep equality)

---

## Statements

### Variable Declaration

```atlas
// Explicit type
let x: number = 42;
var y: string = "hello";

// Type inference
let z = 3.14;  // Inferred as number
```

**Rules:**
- `let` is immutable
- `var` is mutable
- Type can be inferred from initializer
- Initializer required

### Assignment

```atlas
// Simple assignment
name = value;

// Array element assignment
arr[i] = value;

// Compound assignment (mutable variables only)
var += expr;   // Addition
var -= expr;   // Subtraction
var *= expr;   // Multiplication
var /= expr;   // Division
var %= expr;   // Modulo
```

### Increment/Decrement Statements

```atlas
// Mutable variables only
++var;   // Pre-increment
--var;   // Pre-decrement
var++;   // Post-increment
var--;   // Post-decrement
```

### Function Declaration

```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}

// Generic function fn identity<T>(x: T) -> T {
    return x;
}

// No return value
fn greet(name: string) -> void {
    print("Hello " + name);
}

// Nested function fn outer() -> number {
    fn helper(x: number) -> number {
        return x * 2;
    }
    return helper(21);  // Returns 42
}
```

**Rules:**
- Parameter types must be explicit
- Return type must be explicit
- Can be declared at top-level or nested within functions/blocks - Nested functions are hoisted within their scope (forward references allowed)
- Nested functions can shadow outer functions and globals
- Nested functions can call sibling functions at the same scope level

**Current Limitations:**
- Nested functions cannot capture outer scope variables (no closure)
- Anonymous/lambda functions not supported

See `ROADMAP.md` for planned enhancements.

### If Statement

```atlas
if (condition) {
    // true branch
}

if (condition) {
    // true branch
} else {
    // false branch
}
```

**Rules:**
- Condition must be `bool`
- Braces required (no single-statement if)

### While Loop

```atlas
while (condition) {
    // loop body
}
```

**Rules:**
- Condition must be `bool`
- Braces required

### For Loop

```atlas
// Classic for loop
for (let i = 0; i < 10; i = i + 1) {
    // loop body
}

// With increment operator
for (var i = 0; i < 10; i++) {
    // loop body
}

// All parts optional
for (;;) {  // Infinite loop
    break;
}
```

**Syntax:** `for (init; condition; step) { body }`

### For-In Loop

```atlas
// Iterate over array elements
for item in array {
    print(item);
}

// With explicit type annotation
for x in [1, 2, 3] {
    print(x);
}

// Nested iteration
for row in matrix {
    for item in row {
        process(item);
    }
}

// With break and continue
for item in items {
    if (item == target) {
        break;
    }
    if (item < 0) {
        continue;
    }
    process(item);
}
```

**Syntax:** `for IDENTIFIER in expression block`

**Type Requirements:**
- Iterable expression must be of type `array`
- Loop variable has type of array elements
- Type is inferred from array element type

**Scope:**
- Loop variable is scoped to the loop body
- Not accessible outside the loop
- Can shadow outer variables

**Control Flow:**
- `break` exits the for-in loop
- `continue` skips to next iteration
- Early `return` from enclosing function works as expected

**Implementation:**
For-in loops iterate directly over array elements without explicit indexing.

### Return Statement

```atlas
return;           // void return
return expr;      // return value
```

**Rules:**
- Must be inside function body
- Type must match function return type

### Break/Continue

```atlas
break;      // Exit loop
continue;   // Skip to next iteration
```

**Rules:**
- Must be inside loop body

### Expression Statement

```atlas
fn();       // Function call
expr;       // Any expression (value discarded)
```

---

## Grammar (EBNF)

```ebnf
program        = { module_item } ;
module_item    = export_decl | import_decl | decl_or_stmt ;           decl_or_stmt   = fn_decl | stmt ;

(* Module system *)
export_decl    = "export" ( fn_decl | var_decl ) ;
import_decl    = "import" import_clause "from" string ";" ;
import_clause  = named_imports | namespace_import ;
named_imports  = "{" import_specifiers "}" ;
import_specifiers = import_specifier { "," import_specifier } ;
import_specifier  = ident ;
namespace_import  = "*" "as" ident ;

fn_decl        = "fn" ident [ type_params ] "(" [ params ] ")" "->" type block ;
type_params    = "<" type_param_list ">" ;                           type_param_list = ident { "," ident } ;                              params         = param { "," param } ;
param          = ident ":" type ;

stmt           = fn_decl | var_decl | assign_stmt | compound_assign_stmt | increment_stmt
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
call           = primary { [ type_args ] "(" [ args ] ")" | "[" expr "]" } ;  (*  type_args *)
type_args      = "<" type_arg_list ">" ;                             type_arg_list  = type { "," type } ;                                 args           = expr { "," expr } ;
array_literal  = "[" [ args ] "]" ;
primary        = number | string | "true" | "false" | "null" | ident
               | array_literal | "(" expr ")" | match_expr ;           (*  match_expr *)

(* Pattern matching *)
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

type           = primary_type [ "[]" ] | generic_type | function_type ;  primary_type   = "number" | "string" | "bool" | "void" | "null" | "json" ; (*  json *)
generic_type   = ident "<" type_arg_list ">" ;                       function_type  = "(" [ type_list ] ")" "->" type ;
type_list      = type { "," type } ;
ident          = letter { letter | digit | "_" } ;
number         = digit { digit } [ "." digit { digit } ] [ ("e" | "E") ["+" | "-"] digit { digit } ] ;
string         = "\"" { char } "\"" ;
```

---

## Scoping Rules

### Lexical Scoping
- Block scope for `let` and `var`
- Function parameters scoped to function body
- Shadowing allowed in nested scopes

### Redeclaration
- Redeclaring a name in the same scope is a compile-time error
- Shadowing in nested scope is allowed

### Examples

```atlas
let x = 1;
{
    let x = 2;  // OK: shadows outer x
    print(str(x));  // 2
}
print(str(x));  // 1

// Error: redeclaration in same scope
let y = 1;
let y = 2;  // Compile error
```

---

## Identifier Rules

**Syntax:** `letter { letter | digit | "_" }`

**Valid:**
```atlas
x
myVar
_private
user_id
count2
```

**Invalid:**
```atlas
2fast     // Cannot start with digit
my-var    // Hyphens not allowed
fn        // Keywords reserved
```

---

## Notes

- All syntax is case-sensitive
- Semicolons required for statements in file mode
- REPL mode allows semicolon omission for single expressions
- Unicode identifiers not supported (ASCII only)
