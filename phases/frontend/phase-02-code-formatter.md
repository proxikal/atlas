# Phase 02: Code Formatter & Comment Preservation

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Parser must produce AST with full span information.

**Verification:**
```bash
grep -n "Span\|location" crates/atlas-runtime/src/ast.rs
cargo test parser
ls crates/atlas-runtime/src/parser/mod.rs
```

**What's needed:**
- Parser generating complete AST with Span information
- AST nodes with line and column information
- Lexer tokenizing source with location tracking
- Configuration system from foundation/phase-04

**If missing:** Parser from v0.1 should have spans - verify AST structure

---

## Objective
Implement automatic code formatter with comment preservation, configurable style settings, and idempotent formatting producing valid readable Atlas code with consistent indentation, spacing, and line breaking.

## Files
**Create:** `crates/atlas-formatter/` (new crate ~2000 lines total)
**Create:** `crates/atlas-formatter/src/lib.rs` (~200 lines)
**Create:** `crates/atlas-formatter/src/formatter.rs` (~800 lines)
**Create:** `crates/atlas-formatter/src/comments.rs` (~400 lines)
**Create:** `crates/atlas-formatter/src/visitor.rs` (~600 lines)
**Update:** `crates/atlas-runtime/src/lexer/mod.rs` (~100 lines emit comment tokens)
**Update:** `Cargo.toml` (add atlas-formatter to workspace)
**Create:** `crates/atlas-cli/src/commands/fmt.rs` (~300 lines)
**Tests:** `crates/atlas-formatter/tests/formatter_tests.rs` (~600 lines)
**Tests:** `crates/atlas-formatter/tests/comments_tests.rs` (~400 lines)

## Dependencies
- Parser from v0.1 with AST
- Lexer with token span information
- Configuration system from foundation/phase-04
- CLI structure for fmt command

## Implementation

### Formatter Core
Create Formatter struct managing formatting state. Maintain FormatConfig with indent_size max_width trailing_comma and semicolon_style settings. Track output buffer accumulating formatted text. Track current indent_level for proper nesting. Store comment collection for reinsertion. Implement format method taking AST and returning formatted string. Ensure formatting is idempotent running formatter twice produces same output. Handle edge cases empty files, single expressions, complex nesting.

### AST Visitor Pattern
Implement visitor pattern traversing AST nodes. Create visit_program method formatting top-level statements. Create visit_statement handling let assignments, functions, returns, if-else, loops. Create visit_expression formatting literals, binary ops, calls, arrays, objects. Create visit_function_declaration with parameter list and body formatting. Create visit_block with brace placement and indentation. Format each node type according to configuration. Insert appropriate whitespace and newlines. Break long lines respecting max_width. Add trailing commas based on config.

### Comment Preservation
Update lexer to emit comment tokens preserving text and location. Define Comment struct with kind text span and position. Define CommentKind enum Line Block Doc for different comment styles. Define CommentPosition enum Leading Trailing Interior for placement relative to code. Collect comments during parsing associating with nearby AST nodes. Implement comment attachment algorithm matching comments to statements and expressions. Preserve leading comments before statements. Preserve trailing comments on same line. Preserve interior comments within expressions. Reinsert comments in formatted output maintaining intent. Handle edge cases comments at file boundaries, between tokens, in complex expressions.

### Formatting Rules
Implement indentation using configured indent_size spaces per level. Implement line breaking when expressions exceed max_width. Format function calls with arguments wrapped across lines if needed. Format array literals with elements on separate lines for readability. Format object literals with key-value pairs aligned. Add or omit trailing commas based on configuration. Handle semicolon style always, never, or automatic semicolon insertion. Ensure consistent spacing around operators and keywords. Format chains of method calls with proper alignment.

### CLI Integration
Create fmt command in CLI accepting file paths. Read source file into string. Parse source into AST handling parse errors. Load formatter configuration from atlas.toml. Create formatter instance with config. Format AST to string. Write formatted output back to file or stdout. Support check mode verifying formatting without modifying. Support multiple files and directory recursion. Exit with error if parsing fails.

## Tests (TDD - Use rstest)

**Formatter tests:**
1. Basic statements formatting
2. Function declarations with parameters
3. If-else statements indentation
4. Loop formatting
5. Expression formatting operators
6. Array literal formatting
7. Object literal formatting
8. Method chain formatting
9. Long line breaking
10. Trailing comma insertion

**Comment preservation tests:**
1. Line comments before statements
2. Line comments after statements
3. Block comments multi-line
4. Doc comments on functions
5. Comments in expressions
6. Comments at file start and end
7. Mixed comment types
8. Comments with edge case placement

**Idempotency tests:**
1. Format twice produces same output
2. Already formatted code unchanged
3. Various code styles converge

**CLI tests:**
1. Format single file
2. Format multiple files
3. Check mode no modification
4. Invalid syntax error handling

**Minimum test count:** 100 tests (60 formatting, 40 comments)

## Integration Points
- Uses: Parser AST from runtime
- Uses: Lexer from runtime
- Uses: Configuration from atlas-config
- Updates: Lexer to emit comment tokens
- Creates: atlas-formatter crate
- Creates: CLI fmt command
- Output: Formatted Atlas source code

## Acceptance
- Formatter produces valid readable code
- Formatting is idempotent format twice same result
- All comment types preserved line block doc
- Comment positions maintained leading trailing interior
- Configuration respected indent size max width trailing commas semicolons
- Long lines broken at max_width
- CLI command works atlas fmt file.at
- Multiple files supported
- Check mode works without modification
- 100+ tests pass 60 formatting 40 comments
- Formatted output parses successfully
- No clippy warnings
- cargo test passes
