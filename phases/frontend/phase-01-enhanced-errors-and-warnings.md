# Phase 01: Enhanced Error Messages & Warning System

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Parser and type checker must exist from v0.1 with basic diagnostics.

**Verification:**
```bash
grep -n "Diagnostic\|Error\|diagnostic" crates/atlas-runtime/src/error.rs
grep -n "error\|Error" crates/atlas-runtime/src/parser/mod.rs
grep -n "TypeError\|type.*error" crates/atlas-runtime/src/typechecker/mod.rs
grep -n "pub struct Span" crates/atlas-runtime/src/ast.rs
cargo test diagnostics parser typechecker
```

**What's needed:**
- v0.1 error types ParseError TypeError RuntimeError
- AST nodes with Span information line column
- Parser produces errors with locations
- Type checker produces errors with locations

**If missing:** v0.1 should have basic diagnostics - verify error infrastructure exists

---

## Objective
Transform basic error messages into production-quality diagnostics with source snippets, color-coded output, systematic error codes, contextual help suggestions, and comprehensive warning system for non-fatal issues like unused variables and dead code.

## Files
**Create:** `crates/atlas-runtime/src/diagnostics/mod.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/diagnostics/formatter.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/diagnostics/warnings.rs` (~700 lines)
**Create:** `crates/atlas-runtime/src/diagnostics/error_codes.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/error.rs` (~100 lines add error codes)
**Update:** `crates/atlas-runtime/src/parser/mod.rs` (~150 lines emit enhanced errors)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~150 lines emit enhanced errors)
**Update:** `crates/atlas-runtime/src/lib.rs` (~20 lines export diagnostics)
**Tests:** `crates/atlas-runtime/tests/enhanced_errors_tests.rs` (~500 lines)
**Tests:** `crates/atlas-runtime/tests/warnings_tests.rs` (~500 lines)

## Dependencies
- v0.1 complete parser type checker AST with Spans
- Source text available for snippet extraction
- Terminal color support add termcolor crate
- Configuration system from foundation/phase-04

## Implementation

### Error Code System
Define systematic error code enum. Assign parse errors E0001 through E0099 covering unexpected token, expected token, unterminated string, invalid literals, EOF. Assign type errors E0100 through E0199 covering type mismatch, undefined variable, undefined function, arity mismatch, invalid operations. Assign runtime errors E0200 through E0299 covering division by zero, index out of bounds, null reference, stack overflow. Assign warning codes W0001 through W0099 covering unused variable, unused function, dead code, shadowed variable, unnecessary annotations, constant comparisons. Implement as_str method returning code string. Implement description method returning human-readable description. Implement help method returning optional contextual help text for common errors.

### Diagnostic Formatter
Create DiagnosticFormatter managing error output. Create Diagnostic struct with code message span severity and notes fields. Define Severity enum Error Warning Note. Implement format method generating structured error output. Format header line with severity error code and message. Format location line with file line column indicator. Extract source snippet showing relevant line from source text. Generate caret line with carets pointing to error location accounting for column offset and error length. Include help text when available for error code. Append additional notes for context. Support colored output using termcolor library with red for errors yellow for warnings cyan for notes. Auto-detect terminal capabilities and respect NO_COLOR environment variable. Handle Unicode characters correctly in column counting and alignment.

### Warning System
Create WarningEmitter managing warning collection and filtering. Define Warning struct with code message span and kind fields. Define WarningKind enum for unused variable, unused function, dead code, shadowed variable, unnecessary type annotation, constant comparison results. Create WarningConfig with level deny and allow sets. Define WarningLevel enum Allow Warn Deny. Implement emit method checking configuration before adding warning. Check allow set to suppress specific warnings. Check deny set to promote warnings to errors. Respect global warning level. Collect all emitted warnings. Provide warnings accessor and has_warnings predicate. Implement should_error checking if warning should be treated as error based on configuration.

### Parser Integration
Update parser to emit enhanced diagnostics. Create unexpected_token_error method generating E0001 diagnostic. Create expected_token_error method generating E0002 diagnostic with note about expected token. Create unterminated_string_error for E0003. Create invalid_number_error for E0004. Replace simple error strings with Diagnostic structs. Include Span information from tokens in all errors. Add contextual notes where helpful.

### Type Checker Integration
Update type checker to emit enhanced diagnostics. Create type_mismatch_error generating E0101 with expected and found types. Create undefined_variable_error generating E0102 with help suggestion. Create undefined_function_error generating E0103. Create arity_mismatch_error generating E0104 with function signature details. Replace simple TypeError with Diagnostic structs. Include Span information from AST nodes. Add helpful notes about common mistakes.

### Warning Detection
Implement unused variable detection tracking declarations and usages. Implement unused function detection for non-public functions. Implement dead code detection after returns and in unreachable branches. Implement shadowing detection tracking variable scopes. Implement unnecessary annotation detection comparing explicit types with inferred types. Implement constant comparison detection for always true or false comparisons. Emit appropriate WarningKind for each detection with accurate Span.

## Tests (TDD - Use rstest)

**Error formatting tests:**
1. Error codes assigned no duplicates
2. Source snippet extraction correct line
3. Caret alignment correct position
4. Help text for common errors
5. Color output versus plain text
6. Multi-line error support
7. Parse error formatting
8. Type error formatting
9. Unicode character handling
10. Edge cases first line last line empty file

**Warning system tests:**
1. Unused variable detection
2. Unused function detection
3. Dead code detection
4. Variable shadowing detection
5. Unnecessary annotation detection
6. Constant comparison detection
7. Warning configuration allow deny level
8. Warning promotion to errors
9. Warning suppression
10. Multiple warnings collection

**Minimum test count:** 120 tests (60 errors, 60 warnings)

## Integration Points
- Uses: Span from AST
- Uses: Parser error emission
- Uses: Type checker error emission
- Uses: Configuration from foundation/phase-04
- Creates: Enhanced diagnostic system
- Creates: Warning infrastructure
- Output: Production-quality error messages

## Acceptance
- All error codes systematically assigned E0001-E0299 W0001-W0099
- Source snippets show correct line and column
- Carets align properly Unicode-aware
- Color output works on supported terminals
- Help text provided for 10+ common errors
- 6+ warning types implemented and working
- Warnings configurable allow warn deny
- Parser emits enhanced errors with codes
- Type checker emits enhanced errors with codes
- 120+ tests pass 60 errors 60 warnings
- Documentation includes all error codes
- termcolor dependency added to Cargo.toml
- NO_COLOR environment variable respected
- Multi-line error support works
- Warning config integrates with atlas.toml
- No clippy warnings
- cargo test passes
