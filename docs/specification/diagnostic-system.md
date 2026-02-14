# Atlas Diagnostic System

**Purpose:** Comprehensive reference for Atlas diagnostic system including errors, warnings, formatting, and stability rules

**For AI Agents:** This is your single source of truth for diagnostic behavior. All errors and warnings follow these rules consistently across compiler, interpreter, and VM.

---

## Table of Contents
1. [Overview](#overview)
2. [Diagnostic Schema](#diagnostic-schema)
3. [Help Text Standards](#help-text-standards)
4. [Output Formats](#output-formats)
5. [Error Codes](#error-codes)
6. [Warning Codes](#warning-codes)
7. [Emission Policy](#emission-policy)
8. [Ordering Rules](#ordering-rules)
9. [Normalization Rules](#normalization-rules)

---

## Overview

All errors and warnings in Atlas are represented as `Diagnostic` objects. This ensures consistency across:
- Compiler (lexer, parser, type checker)
- Interpreter
- Bytecode VM
- REPL
- CLI tools

**Core Principles:**
- Every diagnostic includes precise location information (file, line, column, length)
- Every diagnostic has a unique error code
- Diagnostics are deterministic and reproducible
- Both human-readable and machine-readable formats are supported

---

## Diagnostic Schema

All diagnostics follow this schema:

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `diag_version` | integer | Diagnostic schema version (currently 1) |
| `level` | string | Either `"error"` or `"warning"` |
| `code` | string | Unique diagnostic code (e.g., `"AT0001"`) |
| `message` | string | Short summary of the diagnostic |
| `file` | string | File path (normalized) |
| `line` | integer | 1-based line number |
| `column` | integer | 1-based column number |
| `length` | integer | Length of the error span in characters |
| `snippet` | string | Source line containing the error |
| `label` | string | Short label for the caret range |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `notes` | string[] | Additional explanatory notes |
| `related` | object[] | Secondary locations providing context |
| `help` | string | **Actionable suggestion for fixing the error** |

**Note:** While `help` is technically optional in the schema, **all diagnostics should provide help text** whenever possible. See [Help Text Standards](#help-text-standards) for guidelines.

### Related Location Schema

Each entry in `related` array:
```json
{
  "file": "path/to/file.atl",
  "line": 10,
  "column": 5,
  "length": 6,
  "message": "context message"
}
```

---

## Help Text Standards

**Philosophy:** Every diagnostic should guide the user toward a solution. Help text transforms errors from "what's wrong" to "how to fix it."

### Requirements

**All diagnostics SHOULD include help text that:**
- ✅ Provides specific, actionable guidance (not generic advice)
- ✅ Uses imperative voice ("change X to Y", "add Z", "remove W")
- ✅ References actual symbols/types from the error context
- ✅ Offers concrete next steps the user can take immediately

**Help text is REQUIRED for:**
- Type mismatches (suggest type conversion or annotation change)
- Undefined symbols (suggest declaration or check for typos)
- Syntax errors (show correct syntax)
- Semantic errors (explain what needs to change)

**Help text may be OMITTED only when:**
- The error message is completely self-explanatory (rare)
- There is genuinely no actionable fix (extremely rare)

### Quality Guidelines

#### ✅ Good Help Text (Specific and Actionable)

**Type Mismatch:**
```
❌ Bad:  "fix the type error"
✅ Good: "ensure both operands are numbers (for addition) or both are strings (for concatenation)"
```

**Undefined Symbol:**
```
❌ Bad:  "declare the variable first"
✅ Good: "declare 'foo' before using it, or check for typos"
```

**Function Arguments:**
```
❌ Bad:  "wrong number of arguments"
✅ Good: "provide exactly 2 arguments (expected 2, found 3)"
```

**Mutability:**
```
❌ Bad:  "variable is immutable"
✅ Good: "declare 'x' as mutable: var x = ... (or use let mut x = ...)"
```

**Import Errors:**
```
❌ Bad:  "module not found"
✅ Good: "ensure the module exists and has been loaded before importing from it"
```

**Pattern Matching:**
```
❌ Bad:  "invalid pattern"
✅ Good: "use 'Some(value)' to match and extract the inner value from Option"
```

#### ❌ Anti-Patterns to Avoid

**Vague Advice:**
```
❌ "check your code"
❌ "fix the syntax"
❌ "see documentation"
❌ "invalid input"
```

**Generic Statements:**
```
❌ "types must match"
❌ "variable must be declared"
❌ "invalid syntax"
```

**Passive Voice:**
```
❌ "the type should be changed"
✅ "change the type to number"
```

**Missing Context:**
```
❌ "provide the correct type"
✅ "the value must be of type number (found string)"
```

### Help Text Patterns by Error Category

#### Type Errors

**Pattern:** `"<operation> requires <type>, found <actual>"`
- Include what operation triggered the error
- State expected type explicitly
- Show actual type found
- Suggest conversion or type annotation change

**Examples:**
```
"arithmetic operators (-, *, /, %) only work with numbers"
"both operands must have the same type for equality comparison"
"argument 2 must be of type string (found number)"
```

#### Symbol Resolution Errors

**Pattern:** Reference the symbol name and suggest fixes
- Always mention the symbol name
- Suggest declaration syntax
- Offer typo check when relevant

**Examples:**
```
"declare 'count' with 'let' or 'const' before assigning to it"
"define 'helper' before exporting it"
"rename this parameter to avoid conflict with 'index'"
```

#### Import/Module Errors

**Pattern:** Explain resolution and suggest path corrections
- Reference module paths explicitly
- Suggest path format when applicable
- Explain dependency requirements

**Examples:**
```
"check the module's exports or import a different symbol"
"refactor your modules to remove circular imports - modules cannot import each other in a cycle"
"Use './file' for same directory, '../file' for parent, or '/src/file' for absolute paths"
```

#### Pattern Matching Errors

**Pattern:** Show correct pattern syntax
- Display the exact syntax needed
- Explain what the pattern does
- Reference the type being matched

**Examples:**
```
"use 'None' without arguments to match empty Option values"
"Add arm: Some(_) => ... or use wildcard _"
"valid constructor patterns are: Some, None (for Option) and Ok, Err (for Result)"
```

#### Syntax Errors

**Pattern:** Show correct syntax directly
- Don't just say "invalid syntax"
- Show the expected token or structure
- Give a concrete example when possible

**Examples:**
```
"check your syntax for typos or missing tokens"
"add '*/' to close the multi-line comment"
"use named imports instead: import { name } from \"...\""
```

### Context-Aware Help Text

**Use variable/type names from the error:**
```rust
// Don't:
.with_help("rename the variable")

// Do:
.with_help(format!("rename '{}' or remove the previous declaration", var_name))
```

**Reference specific types:**
```rust
// Don't:
.with_help("fix the type")

// Do:
.with_help(format!(
    "change the variable type to {} or use a {} value",
    init_type.display_name(),
    declared_type.display_name()
))
```

**Provide exact counts:**
```rust
// Don't:
.with_help("provide correct number of arguments")

// Do:
.with_help(format!(
    "provide exactly {} argument{}",
    expected,
    if expected == 1 { "" } else { "s" }
))
```

### Help Text Style Guide

**Voice:** Imperative, direct
- "change X to Y" ✅
- "X should be changed to Y" ❌

**Tone:** Helpful, not condescending
- "check for typos" ✅
- "you made a mistake" ❌

**Length:** Concise but complete
- One sentence preferred
- Two sentences maximum
- Avoid paragraphs

**Terminology:** Match user's code
- Use actual symbol names from error
- Use Atlas terminology (not Rust/other languages)
- Be consistent with language documentation

### Examples from Atlas Codebase

**Excellent Help Text Examples:**
```rust
// Binder - undefined symbol
.with_help(format!("declare '{}' before using it, or check for typos", id.name))

// Typechecker - type mismatch
.with_help("ensure both operands are numbers (for addition) or both are strings (for concatenation)")

// Typechecker - unused variable
.with_help(format!("remove the variable or prefix with underscore: _{}", name))

// Module loader - circular dependency
.with_help("refactor your modules to remove circular imports - modules cannot import each other in a cycle")

// Compiler - pattern matching
.with_help("use 'Some(value)' to match and extract the inner value from Option")

// Typechecker - mutability
.with_help(format!("declare '{}' as mutable: var {} = ...", id.name, id.name))
```

### Implementation Checklist

When adding a new diagnostic:

- [ ] Error has clear, specific message
- [ ] Error has appropriate error code
- [ ] Error includes precise span information
- [ ] Error has descriptive label for caret
- [ ] **Error includes actionable help text**
- [ ] Help text is specific to the error context
- [ ] Help text uses imperative voice
- [ ] Help text references actual symbols/types when applicable
- [ ] Related locations added if context helps understanding
- [ ] Error tested with snapshot tests

### Coverage Goal

**Target:** 100% of diagnostics should have help text

**Current Coverage:** 59+ diagnostics with comprehensive help text across:
- Typechecker (38 errors)
- Binder (9 errors)
- Compiler (5 errors)
- Module loader (2 errors)
- Lexer (1 error)
- Parser (1 error)
- Module executor (1 error)
- Resolver (3 errors)

**Verification:** Review diagnostic emissions during development to ensure help text is present and high-quality.

---

## Output Formats

### Human-Readable Format

**Error Example:**
```
error[AT0001]: Type mismatch
  --> path/to/file.atl:12:9
   |
12 | let x: number = "hello";
   |         ^^^^^ expected number, found string
   |
help: convert the value to number or change the variable type
```

**Warning Example:**
```
warning[AT2001]: Unused variable
  --> path/to/file.atl:5:9
   |
 5 | let unused = 42;
   |     ^^^^^^ variable declared but never used
   |
help: remove the variable or prefix with underscore: _unused
```

**Format Rules:**
- Level and code on first line: `error[CODE]: Message`
- Location on second line: `--> file:line:column`
- Source snippet with line number
- Caret line showing span with label
- Optional help text at end

### Machine-Readable (JSON) Format

**Full Example:**
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

**JSON Output:**
- One JSON object per diagnostic
- Newline-delimited for multiple diagnostics
- Suitable for parsing by tools and AI agents
- All fields guaranteed to be present (except optional ones)

---

## Error Codes

### Error Code Format
- Prefix: `AT` (Atlas)
- Category digit: Error type (0-9)
- Sequential number: 001-999 within category
- Example: `AT0001`, `AT0005`, `AT3003`

### Error Code Categories

**AT0xxx - Type Errors:**
- `AT0001`: Type mismatch
- `AT0002`: Undefined symbol
- `AT0005`: Divide by zero (runtime)
- `AT0006`: Out-of-bounds array access (runtime)
- `AT0007`: NaN or Infinity result (runtime)

**AT1xxx - Syntax Errors:**
- Parse errors, malformed expressions, invalid syntax

**AT2xxx - Warnings:**
- `AT2001`: Unused variable
- `AT2002`: Unreachable code

**AT3xxx - Semantic Errors:**
- `AT3003`: Immutability violation (assigning to `let`)

**AT4xxx - Runtime Errors:**
- Stack overflow, invalid stdlib arguments, etc.

**AT5xxx - Module Errors:**
- `AT5003`: Circular dependency detected
- `AT5004`: Cannot export (symbol not found)
- `AT5005`: Module not found
- `AT5006`: Module does not export symbol
- `AT5007`: Namespace imports not yet supported
- `AT5008`: Duplicate export

### Error Code Policy

**Rules:**
- Every diagnostic MUST use a defined error code
- New error codes MUST be added to `Atlas-SPEC.md` first
- Error codes are stable (never reused for different errors)
- Same error condition always produces same error code

**For complete error code listing, see:** `Atlas-SPEC.md`

---

## Warning Codes

### Warning Categories

Warnings are non-fatal diagnostics that indicate potential issues without blocking execution.

### Current Warning Codes

**AT2001: Unused Variable**
- **Triggered when:** Variable declared but never read
- **Example:**
  ```atlas
  let unused = 42;  // AT2001 warning
  let x = 5;
  print(x);
  ```
- **Suppression:** Prefix variable name with `_`
  ```atlas
  let _unused = 42;  // No warning
  ```

**AT2002: Unreachable Code**
- **Triggered when:** Code after `return`, `break`, or `continue` that cannot execute
- **Example:**
  ```atlas
  fn foo() -> number {
      return 42;
      print("never executed");  // AT2002 warning
  }
  ```

### Warning Emission Rules

**Behavior:**
- Warnings do NOT block execution
- Programs with warnings can still run
- Warnings are emitted even if errors exist

**Unused Variable Rules:**
- Variables starting with `_` do NOT trigger warnings
- Unused function parameters ARE warned unless prefixed with `_`
- Unused function return values do NOT trigger warnings

---

## Emission Policy

### Error Emission

**Compile-Time Errors:**
- Stop after **25 errors** (prevent overwhelming output)
- Errors are emitted as soon as detected
- First 25 errors are guaranteed to be reported

**Runtime Errors:**
- First error stops execution
- Runtime error produces diagnostic with stack trace
- REPL reports runtime error but continues accepting input

**REPL-Specific:**
- REPL reports **first error** per input line
- Multiple errors in one input only show first
- Avoids overwhelming user during interactive development

### Warning Emission

**Compile-Time Warnings:**
- ALL warnings are emitted (no limit)
- Warnings emitted even if errors exist
- Warnings do not block compilation or execution

**Warning vs Error Priority:**
- Errors are shown before warnings
- Warnings are supplementary information

---

## Ordering Rules

### Purpose
Ensure deterministic, predictable diagnostic output. Same input always produces same output order.

### Ordering Algorithm

**Primary Sort:** Level
1. All errors first
2. Then all warnings

**Secondary Sort:** Location (within same level)
1. Sort by file path (lexicographic)
2. Then by line number (ascending)
3. Then by column number (ascending)

**Example:**
```
Input with diagnostics at:
- error at file.atl:15:5
- warning at file.atl:10:3
- error at file.atl:10:5

Output order:
1. error at file.atl:10:5  (error, line 10, col 5)
2. error at file.atl:15:5  (error, line 15, col 5)
3. warning at file.atl:10:3 (warning after all errors)
```

### Multiple Files

When diagnostics span multiple files:
```
file_a.atl:10:5 error
file_b.atl:5:3 error
file_a.atl:20:1 error
file_b.atl:15:7 warning

Output order:
1. file_a.atl:10:5 error
2. file_a.atl:20:1 error
3. file_b.atl:5:3 error
4. file_b.atl:15:7 warning
```

---

## Normalization Rules

### Purpose
Ensure diagnostics are stable and reproducible across different machines, operating systems, and development environments.

### Path Normalization

**Absolute to Relative:**
- Strip absolute path prefixes
- Use relative paths from project root
- Example: `/Users/me/projects/atlas/src/main.atl` → `src/main.atl`

**Fallback to Filename:**
- If relative path unavailable, use filename only
- Example: `main.atl`

**Consistency:**
- Same source file always produces same path in diagnostics
- No machine-specific path information leaks into diagnostics

### Line Ending Normalization

**Rule:**
- All line endings normalized to `\n` (Unix style)
- Applies to both source input and diagnostic output
- `\r\n` (Windows) converted to `\n`
- `\r` (old Mac) converted to `\n`

**Rationale:**
- Consistent diagnostics regardless of source file line endings
- Identical JSON output across platforms

### Volatile Field Removal

**Remove:**
- Timestamps
- OS-specific paths
- Machine-specific information
- Temporary directory paths
- User-specific information

**Keep:**
- Error codes
- Messages
- Relative file paths
- Line/column numbers
- Source snippets

### Cross-Machine Stability

**Test:**
- Same input on two different machines yields **identical** JSON output
- Bit-for-bit identical diagnostic objects
- Enables reproducible testing and CI/CD

**Applications:**
- Snapshot testing with `insta`
- Cross-platform CI verification
- Deterministic test suites

---

## Integration with Testing

### Snapshot Testing

**Using `insta` crate:**
```rust
let diagnostics = compile("invalid.atl");
insta::assert_json_snapshot!(diagnostics);
```

**Benefits:**
- Catch unintended diagnostic changes
- Verify diagnostic message quality
- Ensure consistent error codes

### Property Testing

**Invariants to test:**
- All diagnostics have valid error codes
- All diagnostics have non-empty messages
- Line/column numbers are positive
- Length is non-negative
- Ordering rules are followed

---

## Best Practices

### For Compiler Authors

**DO:**
- ✅ Always include precise span information
- ✅ Provide clear, specific error messages
- ✅ **Include actionable `help` text for every diagnostic** (see [Help Text Standards](#help-text-standards))
- ✅ Use imperative voice in help text ("change X", "add Y", "remove Z")
- ✅ Reference actual symbols/types from error context in help text
- ✅ Use `related` locations to show context
- ✅ Follow ordering rules consistently
- ✅ Test diagnostics with snapshot tests

**DON'T:**
- ❌ Create new error codes without spec update
- ❌ Skip span information
- ❌ Write vague error messages ("fix your code", "invalid syntax")
- ❌ Write generic help text ("see documentation", "check the manual")
- ❌ Omit help text unless absolutely no fix is possible
- ❌ Emit diagnostics with invalid fields
- ❌ Use passive voice in help text ("should be changed" → "change")

### For AI Agents

**When generating code:**
- Parse JSON diagnostics programmatically
- Extract error code and location
- Use `help` text for fix suggestions
- Follow `related` locations for context

**When reading diagnostics:**
- Check `diag_version` for compatibility
- Look for `help` field for guidance
- Use `related` to understand full error context
- Error codes are stable - safe to pattern match

---

## Diagnostic Examples

### Type Mismatch with Help

```json
{
  "diag_version": 1,
  "level": "error",
  "code": "AT0001",
  "message": "Type mismatch",
  "file": "example.atl",
  "line": 5,
  "column": 14,
  "length": 7,
  "snippet": "let x: number = \"hello\";",
  "label": "expected number, found string",
  "help": "convert the value using str() or change the variable type"
}
```

### Undefined Symbol with Related Location

```json
{
  "diag_version": 1,
  "level": "error",
  "code": "AT0002",
  "message": "Undefined symbol",
  "file": "example.atl",
  "line": 10,
  "column": 9,
  "length": 3,
  "snippet": "let y = foo;",
  "label": "symbol 'foo' not found",
  "related": [
    {
      "file": "example.atl",
      "line": 15,
      "column": 4,
      "length": 3,
      "message": "similar function 'bar' defined here"
    }
  ],
  "help": "did you mean 'bar'?"
}
```

### Unused Variable Warning

```json
{
  "diag_version": 1,
  "level": "warning",
  "code": "AT2001",
  "message": "Unused variable",
  "file": "example.atl",
  "line": 3,
  "column": 5,
  "length": 6,
  "snippet": "let unused = 42;",
  "label": "variable declared but never used",
  "help": "remove the variable or prefix with underscore: _unused"
}
```

---

## Implementation References

**For implementation details, see:**
- `docs/implementation/08-diagnostics.md` - Diagnostic implementation guide
- `Atlas-SPEC.md` - Complete error code listing
- `docs/implementation/07-typechecker.md` - Type error generation
- `docs/implementation/03-lexer.md` - Lexer error generation
- `docs/implementation/04-parser.md` - Parser error generation

---

**Summary:** Atlas diagnostics are precise, consistent, deterministic, and designed for both human and AI consumption. Every diagnostic follows strict rules for schema, ordering, and normalization to ensure reproducibility across all environments.
