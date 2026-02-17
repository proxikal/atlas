# Atlas JSON Dump Formats

**Purpose:** Comprehensive reference for JSON output formats including AST dumps, typecheck dumps, debug info, and stability guarantees

**For AI Agents:** Use these JSON formats to analyze Atlas programs programmatically. All formats are stable, deterministic, and versioned.

---

## Table of Contents
1. [Overview](#overview)
2. [AST JSON Dump](#ast-json-dump)
3. [Typecheck JSON Dump](#typecheck-json-dump)
4. [Debug Information Format](#debug-information-format)
5. [Stability Guarantees](#stability-guarantees)

---

## Overview

Atlas provides machine-readable JSON dumps of:
- **AST (Abstract Syntax Tree)** - Complete parse tree with spans
- **Typecheck Results** - Inferred types, symbol bindings, type information
- **Debug Info** - Source location mapping for bytecode instructions

**Design Principles:**
- Deterministic output (same input always produces same output)
- Stable across environments (same output on different machines/OSes)
- Versioned formats (breaking changes increment version)
- Complete span information (every node has source location)

**Use Cases:**
- AI agents analyzing code structure
- IDEs and editors for syntax highlighting
- Linters and code analysis tools
- Test assertions and snapshot testing
- Debugging and error reporting

---

## AST JSON Dump

### Purpose
Provide stable JSON representation of Abstract Syntax Tree for AI agents and tooling.

### CLI Usage
```bash
# Dump AST as JSON
atlas ast path/to/file.atl --json

# Dump AST as human-readable format
atlas ast path/to/file.atl
```

### Schema

**Root Structure:**
```json
{
  "ast_version": 1,
  "kind": "Program",
  "items": [...]
}
```

**Node Structure:**
Every AST node includes:
- `kind`: Node type (e.g., "LetDecl", "FunctionDecl", "BinaryExpr")
- `span`: Source location information
- Additional fields specific to node type

**Span Structure:**
```json
{
  "file": "path/to/file.atl",
  "start": 0,
  "end": 15,
  "line": 1,
  "column": 1
}
```

### Example AST Dump

**Input:**
```atlas
let x: number = 42;
```

**Output:**
```json
{
  "ast_version": 1,
  "kind": "Program",
  "items": [
    {
      "kind": "LetDecl",
      "span": {
        "file": "example.atl",
        "start": 0,
        "end": 19,
        "line": 1,
        "column": 1
      },
      "name": "x",
      "type_annotation": {
        "kind": "NumberType",
        "span": {...}
      },
      "initializer": {
        "kind": "NumberLiteral",
        "span": {...},
        "value": 42.0
      }
    }
  ]
}
```

### AST Node Types

**Declarations:**
- `LetDecl` - Immutable variable declaration
- `VarDecl` - Mutable variable declaration
- `FunctionDecl` - Function declaration

**Statements:**
- `ExprStmt` - Expression statement
- `IfStmt` - If/else statement
- `WhileStmt` - While loop
- `ForStmt` - For loop
- `ReturnStmt` - Return statement
- `BreakStmt` - Break statement
- `ContinueStmt` - Continue statement

**Expressions:**
- `NumberLiteral` - Number literal
- `StringLiteral` - String literal
- `BoolLiteral` - Boolean literal
- `NullLiteral` - Null literal
- `ArrayLiteral` - Array literal
- `Identifier` - Variable reference
- `BinaryExpr` - Binary operation
- `UnaryExpr` - Unary operation
- `CallExpr` - Function call
- `IndexExpr` - Array indexing

**Types:**
- `NumberType` - number type
- `StringType` - string type
- `BoolType` - bool type
- `VoidType` - void type
- `NullType` - null type
- `ArrayType` - T[] type
- `FunctionType` - (T1, T2) -> T3 type

### Versioning

**Current Version:** `ast_version: 2`

**Breaking Changes:**
- Node kind renamed or removed → Version increment
- Required field added/removed → Version increment
- Span format changed → Version increment

**Non-Breaking Changes:**
- Optional field added → No version increment
- Field order changed (order is deterministic but not semantic)

---

## Typecheck JSON Dump

### Purpose
Provide stable JSON representation of type checker results including inferred types and symbol bindings.

### CLI Usage
```bash
# Dump typecheck results as JSON
atlas typecheck path/to/file.atl --json

# Run type checker (human-readable output)
atlas typecheck path/to/file.atl
```

### Schema

**Root Structure:**
```json
{
  "typecheck_version": 1,
  "symbols": [...],
  "types": [...]
}
```

**Symbol Structure:**
```json
{
  "name": "variable_name",
  "kind": "variable" | "function" | "parameter",
  "span": {...},
  "type": {...},
  "declared_type": {...} | null,
  "inferred_type": {...}
}
```

**Type Structure:**
```json
{
  "kind": "number" | "string" | "bool" | "void" | "null" | "array" | "function",
  "element_type": {...} | null,  // For arrays
  "param_types": [...] | null,   // For functions
  "return_type": {...} | null    // For functions
}
```

### Example Typecheck Dump

**Input:**
```atlas
let x = 42;
fn add(a: number, b: number) -> number {
    return a + b;
}
```

**Output:**
```json
{
  "typecheck_version": 1,
  "symbols": [
    {
      "name": "x",
      "kind": "variable",
      "span": {
        "file": "example.atl",
        "start": 4,
        "end": 5,
        "line": 1,
        "column": 5
      },
      "declared_type": null,
      "inferred_type": {
        "kind": "number"
      }
    },
    {
      "name": "add",
      "kind": "function",
      "span": {
        "file": "example.atl",
        "start": 15,
        "end": 18,
        "line": 2,
        "column": 4
      },
      "declared_type": {
        "kind": "function",
        "param_types": [
          {"kind": "number"},
          {"kind": "number"}
        ],
        "return_type": {"kind": "number"}
      },
      "inferred_type": {
        "kind": "function",
        "param_types": [
          {"kind": "number"},
          {"kind": "number"}
        ],
        "return_type": {"kind": "number"}
      }
    }
  ],
  "types": []
}
```

### Versioning

**Current Version:** `typecheck_version: 1`

**Breaking Changes:**
- Symbol kind added/removed → Version increment
- Type representation changed → Version increment
- Required field added/removed → Version increment

---

## Debug Information Format

### Purpose
Enable precise error reporting by mapping bytecode instructions back to source locations.

### Debug Info in Bytecode

**Structure:**
- Each bytecode instruction has associated debug info
- Debug info includes `Span` (file, line, column, length)
- Stored in `.atb` bytecode files

**Benefits:**
- Runtime errors show exact source location
- Stack traces include line numbers
- Debugging tools can map bytecode to source

### Debug Info Storage

**Instruction Stream:**
```rust
struct Chunk {
    instructions: Vec<Instruction>,
    debug_info: Vec<SpanId>,  // Parallel array
    span_table: Vec<Span>,    // Unique spans
}
```

**Span Table:**
- Deduplicated span storage
- Each unique span gets an ID
- Instructions reference spans by ID

**Example Mapping:**
```
Instruction 0: LOAD_CONST 0    -> Span ID 5 -> (file.atl, line 3, col 9)
Instruction 1: LOAD_CONST 1    -> Span ID 5 -> (file.atl, line 3, col 9)
Instruction 2: ADD             -> Span ID 6 -> (file.atl, line 3, col 11)
```

### Debug Info in .atb Files

**Section in Bytecode Format:**
```
Debug Info Section:
  - Span table count: u32
  - Each span:
    - File index: u32
    - Line: u32
    - Column: u32
    - Length: u32
  - Instruction span mapping:
    - u32 per instruction (index into span table)

File Table:
  - File count: u32
  - Each file:
    - u32 length + UTF-8 bytes
```

**See:** `docs/RUNTIME.md` for complete bytecode format specification

### Debug Info Policy

**Current Behavior:**
- Debug info is **enabled by default**
- All bytecode includes complete source mapping
- No runtime performance impact (info not loaded unless needed)

**Future Considerations:**
- Optional stripping for size optimization
- Separate debug symbol files
- Source map format for minified code

---

## Stability Guarantees

### Purpose
Ensure JSON dumps are deterministic, reproducible, and stable across environments.

### Deterministic Output

**Field Ordering:**
- JSON object fields always appear in same order
- Array elements maintain source order
- Consistent serialization across runs

**Example:**
```json
// Always this order:
{
  "kind": "...",
  "span": {...},
  "name": "...",
  "type": {...}
}

// Never:
{
  "span": {...},
  "kind": "...",
  // random order
}
```

### Formatting Consistency

**Rules:**
- No trailing whitespace
- Consistent indentation (2 spaces)
- Stable line breaks
- UTF-8 encoding

**Benefits:**
- Snapshot testing works reliably
- Git diffs are clean and meaningful
- Cross-platform consistency

### Version Fields

**All dumps include version:**
- `ast_version: 2` in AST dumps
- `typecheck_version: 1` in typecheck dumps
- `diag_version: 1` in diagnostics (see DIAGNOSTIC_SYSTEM.md)

**Version Policy:**
- Breaking format changes increment version
- Non-breaking additions preserve version
- Version mismatch produces warning

### Cross-Environment Stability

**Guaranteed Identical Output:**
- Same source code on different machines → Same JSON
- Different OSes (Linux, macOS, Windows) → Same JSON
- Different times → Same JSON (no timestamps)

**Test Verification:**
```bash
# On machine A:
atlas ast example.atl --json > dump_a.json

# On machine B:
atlas ast example.atl --json > dump_b.json

# Should be identical:
diff dump_a.json dump_b.json  # No differences
```

### Normalization Rules

**Paths:**
- Use relative paths from project root
- No absolute paths
- Consistent path separators (/)

**Line Endings:**
- All normalized to `\n`
- Windows `\r\n` converted to `\n`

**Timestamps:**
- No timestamps in output
- No volatile fields

**Benefits:**
- Reliable snapshot testing with `insta`
- Reproducible CI/CD builds
- Cross-platform test suites

---

## Testing with JSON Dumps

### Snapshot Testing

**Using Rust `insta` crate:**
```rust
#[test]
fn test_ast_dump() {
    let ast = parse("let x = 42;");
    let json = serde_json::to_string_pretty(&ast).unwrap();
    insta::assert_snapshot!(json);
}
```

**Benefits:**
- Catch unintended AST changes
- Verify format stability
- Detect regressions

### Property Testing

**Invariants to verify:**
- Every node has valid span
- All spans have positive line/column
- Version fields always present
- No duplicate field names

---

## Integration with Tools

### IDEs and Editors

**Syntax Highlighting:**
- Parse AST dump to identify tokens
- Apply colors based on node kinds
- Span information for precise highlighting

**Code Navigation:**
- Jump to definition using symbol spans
- Find references using typecheck dump
- Outline view from AST structure

### Linters and Analyzers

**Custom Rules:**
```python
# Python example: Find unused variables
import json

dump = json.loads(typecheck_output)
for symbol in dump["symbols"]:
    if symbol["kind"] == "variable" and not symbol.get("used"):
        print(f"Unused: {symbol['name']} at {symbol['span']}")
```

### AI Agents

**Code Understanding:**
- Parse AST to understand program structure
- Use typecheck dump to verify type correctness
- Extract spans for precise code modifications

---

## Implementation References

**For implementation details, see:**
- `docs/implementation/05-ast.md` - AST structure and serialization
- `docs/implementation/07-typechecker.md` - Type checking and symbol resolution
- `docs/implementation/11-bytecode.md` - Bytecode format and debug info
- `docs/RUNTIME.md` - Complete bytecode format specification

---

**Summary:** Atlas JSON dumps provide stable, deterministic, versioned machine-readable representations of programs. Use them for tooling, testing, and AI-driven code analysis.
