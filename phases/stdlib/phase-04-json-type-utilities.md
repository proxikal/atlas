# Phase 04: JSON & Type Utilities

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Value model must support all JSON types and type checking.

**Verification:**
```bash
grep -n "enum Value" crates/atlas-runtime/src/value.rs
grep -n "Object\|Map" crates/atlas-runtime/src/value.rs
```

**What's needed:**
- Value enum with Number, String, Bool, Null, Array
- Object/Map type for JSON objects

**If missing:** May need to add Value::Object variant

---

## Objective
Implement JSON parsing/serialization and runtime type checking utilities - 17 functions covering JSON operations, type guards, type conversion, and error handling.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/json.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/stdlib/types.rs` (~600 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (add modules)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (register functions)
**Update:** `crates/atlas-runtime/src/value.rs` (add Object variant if needed)
**Update:** `Cargo.toml` (add serde_json)
**Tests:** `crates/atlas-runtime/tests/stdlib_json_tests.rs` (~400 lines)
**Tests:** `crates/atlas-runtime/tests/stdlib_types_tests.rs` (~400 lines)
**VM Tests:** VM versions of both test files (~400 lines each)

## Dependencies
- v0.1 complete with Value model
- serde_json crate for JSON parsing
- Atlas-SPEC.md defines type system and JSON semantics

## Implementation

### JSON Functions (5 functions)
Implement parseJSON, toJSON, isValidJSON, prettifyJSON, minifyJSON. Parsing converts JSON to Atlas values with proper type mapping. Serialization handles all Atlas types detecting circular references. Validation checks without parsing. Pretty/minify format JSON strings with custom indentation.

### Type Checking Functions (7 functions)
Implement typeof, isString, isNumber, isBool, isNull, isArray, isFunction. Typeof returns type name as string. Guard functions return booleans. Note NaN is still a number.

### Type Conversion Functions (5 functions)
Implement toString, toNumber, toBool, parseInt, parseFloat. ToString converts any value to string representation. ToNumber handles truthy/falsy conversion. ToBool follows JavaScript-like rules. ParseInt supports radix 2-36. ParseFloat handles scientific notation.

### Architecture Notes
Use serde_json for JSON operations. Map JSON types bidirectionally with Atlas Value types. Detect circular references using pointer set before serialization. Functions cannot serialize - return error. Type conversions follow Atlas spec for truthy/falsy rules.

## Tests (TDD - Use rstest)

**JSON tests cover:**
1. Valid/invalid JSON parsing
2. All type mappings
3. Circular reference detection
4. Pretty/minify formatting
5. Edge cases - empty objects, nested arrays

**Type tests cover:**
1. All type guards with all value types
2. Typeof accuracy
3. Conversion correctness
4. ParseInt with various radixes
5. Truthy/falsy rules
6. VM parity

**Minimum test count:** 120 tests (60 interpreter, 60 VM)

## Integration Points
- Uses: Value enum (add Object if needed)
- Updates: Cargo.toml with serde_json = "1.0"
- Updates: value.rs for Object variant
- Updates: prelude.rs with 17 functions
- Updates: docs/stdlib.md
- Output: JSON and type utilities

## Acceptance
- All 17 functions implemented
- JSON parsing/serialization works
- Circular references detected
- Type checking accurate
- Type conversion follows spec
- 120+ tests pass
- Interpreter/VM parity verified
- json.rs under 600 lines, types.rs under 700 lines
- Test files under 500 lines each
- Documentation updated
- No clippy warnings
- cargo test passes
