# Atlas JSON Stdlib Plan

**Version:** 1.0
**Status:** Design
**Target Release:** v0.5 (with JsonValue type) or v1.0 (with union types)
**Last Updated:** 2026-02-12

---

## Overview

This document defines the design for JSON parsing and serialization in Atlas. JSON support is critical for AI agent workflows, API integration, and data interchange, but presents unique challenges due to Atlas's strict type system.

**Core Challenge:** JSON is inherently dynamic (heterogeneous arrays, arbitrary object structures) while Atlas v0.1 is strictly typed without unions, generics beyond arrays, or an `any` type.

**Proposed Solution:** Introduce a `JsonValue` type as a restricted form of dynamic typing specifically for JSON data, isolated from the rest of the type system.

---

## Design Constraints

### Atlas Type System (v0.1)

**Available Types:**
- Primitives: `number`, `string`, `bool`, `null`
- Arrays: `T[]` (homogeneous, single type parameter)
- Functions: `(T1, T2) -> T3`

**Not Available (v0.1):**
- ❌ Union types (`string | number`)
- ❌ `any` type (no implicit or explicit dynamic typing)
- ❌ Object/map types (`{key: value}`)
- ❌ Generics beyond arrays
- ❌ Type aliases

### JSON Data Model

**JSON Value Types:**
- Primitives: number, string, boolean, null
- Array: `[value1, value2, ...]` (heterogeneous, any mix of types)
- Object: `{"key": value, ...}` (string keys, any value types)

**Mismatches:**
1. JSON arrays can mix types: `[1, "hello", true, null]`
2. JSON objects have no Atlas equivalent
3. JSON nesting is arbitrary: `{"users": [{"name": "Alice", "age": 30}]}`

---

## Proposed Type System Extension

### JsonValue Type

Add a new variant to the `Value` enum specifically for JSON data:

```rust
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Null,
    Array(Rc<RefCell<Vec<Value>>>),
    Function(FunctionRef),
    JsonValue(Rc<JsonData>),  // NEW: JSON-specific value
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonData {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Array(Vec<JsonData>),      // Heterogeneous array
    Object(BTreeMap<String, JsonData>),  // Key-value map
}
```

### Type System Integration

**JsonValue Properties:**
- **Isolated:** Cannot be assigned to regular Atlas variables
- **Opaque:** Cannot be used in expressions (`json + 1` is a type error)
- **Explicit Conversion:** Must explicitly extract values to Atlas types
- **Parse-Only:** Created via `json::parse()`, serialized via `json::stringify()`

**Rationale:**
- Avoids polluting Atlas's strict type system with dynamic types
- Makes JSON data explicitly distinct from regular values
- Forces developers to handle type uncertainty at JSON boundaries
- Prevents accidental mixing of JSON and Atlas values

---

## API Design

### Module: `json`

**Import (v1.0+):**
```atlas
import json;
```

**v0.5 (Pre-module system):**
Functions available in global namespace as `json_parse()` and `json_stringify()`.

### Function: `json::parse`

**Signature:**
```atlas
fn parse(input: string) -> JsonValue
```

**Behavior:**
- Parses JSON string into a `JsonValue`
- Throws runtime error on invalid JSON (AT0110)
- Supports full JSON spec (RFC 8259)

**Examples:**
```atlas
import json;

// Parse simple values
let num = json::parse("42");           // JsonValue(Number(42))
let str = json::parse("\"hello\"");    // JsonValue(String("hello"))
let bool_val = json::parse("true");    // JsonValue(Bool(true))
let null_val = json::parse("null");    // JsonValue(Null)

// Parse arrays
let arr = json::parse("[1, 2, 3]");                    // Homogeneous
let mixed = json::parse("[1, \"two\", true, null]");   // Heterogeneous

// Parse objects
let obj = json::parse("{\"name\": \"Alice\", \"age\": 30}");

// Parse nested structures
let data = json::parse("{
    \"users\": [
        {\"id\": 1, \"name\": \"Alice\"},
        {\"id\": 2, \"name\": \"Bob\"}
    ],
    \"count\": 2
}");
```

**Error Cases:**
```atlas
// Syntax error
let bad = json::parse("{invalid}");
// runtime error[AT0110]: JSON parse error: expected property name at line 1, column 2

// Unexpected end
let incomplete = json::parse("[1, 2");
// runtime error[AT0110]: JSON parse error: unexpected end of input

// Invalid escape
let bad_escape = json::parse("\"\\x\"");
// runtime error[AT0110]: JSON parse error: invalid escape sequence '\\x' at line 1, column 2
```

### Function: `json::stringify`

**Signature:**
```atlas
fn stringify(value: JsonValue) -> string
```

**Behavior:**
- Serializes `JsonValue` to JSON string
- Output is compact (no whitespace)
- Escape sequences per JSON spec
- Numbers formatted as shortest decimal representation
- NaN and Infinity not allowed (runtime error AT0111)

**Examples:**
```atlas
import json;

// Round-trip
let original = "{\"name\":\"Alice\",\"age\":30}";
let parsed = json::parse(original);
let serialized = json::stringify(parsed);
print(serialized);  // {"name":"Alice","age":30}

// Create from parse, then serialize
let data = json::parse("[1, 2, 3]");
let json_str = json::stringify(data);
print(json_str);  // [1,2,3]
```

### Helper Functions: Value Extraction

Since `JsonValue` is opaque, provide helper functions to extract Atlas values:

**`json::get_number`**
```atlas
fn get_number(value: JsonValue) -> number
```
- Extracts number from JsonValue
- Throws AT0112 if value is not a JSON number

**`json::get_string`**
```atlas
fn get_string(value: JsonValue) -> string
```
- Extracts string from JsonValue
- Throws AT0112 if value is not a JSON string

**`json::get_bool`**
```atlas
fn get_bool(value: JsonValue) -> bool
```
- Extracts boolean from JsonValue
- Throws AT0112 if value is not a JSON boolean

**`json::is_null`**
```atlas
fn is_null(value: JsonValue) -> bool
```
- Returns true if JsonValue is null, false otherwise

**`json::get_array`**
```atlas
fn get_array(value: JsonValue) -> JsonValue[]
```
- Extracts array from JsonValue
- Returns array of JsonValue (still opaque, need further extraction)
- Throws AT0112 if value is not a JSON array

**`json::get_object_value`**
```atlas
fn get_object_value(value: JsonValue, key: string) -> JsonValue
```
- Gets value for key from JSON object
- Throws AT0112 if value is not a JSON object
- Throws AT0113 if key does not exist

**Example Usage:**
```atlas
import json;

let json_str = "{\"name\":\"Alice\",\"age\":30,\"active\":true}";
let data = json::parse(json_str);

// Extract fields
let name = json::get_string(json::get_object_value(data, "name"));
let age = json::get_number(json::get_object_value(data, "age"));
let active = json::get_bool(json::get_object_value(data, "active"));

print(name);    // Alice
print(age);     // 30
print(active);  // true
```

**Nested Access:**
```atlas
import json;

let json_str = "{\"user\":{\"name\":\"Alice\",\"scores\":[95,87,92]}}";
let data = json::parse(json_str);

// Navigate nested structure
let user = json::get_object_value(data, "user");
let name = json::get_string(json::get_object_value(user, "name"));
let scores = json::get_array(json::get_object_value(user, "scores"));
let first_score = json::get_number(scores[0]);

print(name);         // Alice
print(first_score);  // 95
```

---

## Error Codes

### AT0110: JSON Parse Error

**Cause:** Invalid JSON syntax

**Examples:**
```atlas
json::parse("{invalid}")           // Missing quotes around key
json::parse("[1, 2,]")             // Trailing comma
json::parse("{\"a\": undefined}")  // JavaScript literal, not JSON
```

**Error Format:**
```
runtime error[AT0110]: JSON parse error: expected property name at line 1, column 2
  --> script.atl:5:14
   |
 5 | let data = json::parse("{invalid}");
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^ JSON syntax error
   |
   = note: expected property name or '}' at position 1
   = help: JSON object keys must be quoted strings: {"key": "value"}
```

### AT0111: JSON Invalid Value

**Cause:** Attempting to serialize NaN or Infinity

**Example:**
```atlas
// This would require creating JsonValue from Atlas values, which is phase 2
// For now, this error is reserved for future use
```

**Error Format:**
```
runtime error[AT0111]: Cannot serialize NaN or Infinity to JSON
  --> script.atl:8:14
   |
 8 | let str = json::stringify(invalid_value);
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^ value contains NaN or Infinity
   |
   = help: JSON does not support NaN or Infinity values
```

### AT0112: JSON Type Mismatch

**Cause:** Attempting to extract wrong type from JsonValue

**Examples:**
```atlas
let data = json::parse("\"hello\"");
let num = json::get_number(data);  // Error: data is a string, not a number

let data = json::parse("42");
let arr = json::get_array(data);  // Error: data is a number, not an array
```

**Error Format:**
```
runtime error[AT0112]: JSON type mismatch: expected number, found string
  --> script.atl:10:11
   |
10 | let num = json::get_number(data);
   |           ^^^^^^^^^^^^^^^^^^^^^^ expected JSON number
   |
   = note: value is a JSON string: "hello"
   = help: use json::get_string() to extract string values
```

### AT0113: JSON Key Not Found

**Cause:** Accessing non-existent object key

**Example:**
```atlas
let data = json::parse("{\"name\":\"Alice\"}");
let age = json::get_object_value(data, "age");  // Error: key "age" doesn't exist
```

**Error Format:**
```
runtime error[AT0113]: JSON object key not found: "age"
  --> script.atl:12:11
   |
12 | let age = json::get_object_value(data, "age");
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ key does not exist
   |
   = note: available keys: ["name"]
   = help: check for key existence or use a default value
```

---

## JSON Spec Compliance

### Supported (RFC 8259)

- ✅ Numbers: integers, decimals, scientific notation (`1`, `3.14`, `1e10`)
- ✅ Strings: Unicode, escape sequences (`\n`, `\t`, `\"`, `\\`, `\/`, `\uXXXX`)
- ✅ Booleans: `true`, `false`
- ✅ Null: `null`
- ✅ Arrays: `[...]` (heterogeneous, nested)
- ✅ Objects: `{...}` (string keys, any values, nested)
- ✅ Whitespace: space, tab, newline, carriage return (ignored)

### Not Supported (Extensions)

- ❌ Comments (`// ...` or `/* ... */`) - not part of JSON spec
- ❌ Trailing commas in arrays/objects - not part of JSON spec
- ❌ Unquoted keys - not part of JSON spec
- ❌ Single quotes for strings - not part of JSON spec
- ❌ `undefined`, `NaN`, `Infinity` - JavaScript literals, not JSON
- ❌ BigInt - not part of JSON spec

### Parsing Rules

**Numbers:**
- Leading zeros not allowed: `01` is invalid, `1` is valid
- Decimal point requires digits: `.5` is invalid, `0.5` is valid
- Exponent case-insensitive: `1e10` and `1E10` both valid
- No hex/octal/binary: `0xFF` is invalid

**Strings:**
- Must be double-quoted: `"hello"` valid, `'hello'` invalid
- Escape sequences: `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`
- Unicode escapes: `\uXXXX` (4 hex digits)
- Surrogate pairs for characters > U+FFFF: `\uD834\uDD1E` (musical G clef)

**Objects:**
- Keys must be strings: `{"key": ...}` valid, `{key: ...}` invalid
- Duplicate keys: last value wins (per spec)
- Order: preserved (implementation-defined, but we use BTreeMap for stability)

**Whitespace:**
- Space (U+0020), tab (U+0009), newline (U+000A), carriage return (U+000D)
- Whitespace allowed before/after values, not inside tokens

---

## Implementation Requirements

### Parser

**Library:** `serde_json` (de facto standard Rust JSON library)
- Well-maintained, security-audited
- Fully compliant with RFC 8259
- Excellent error messages
- Battle-tested (used by millions)

**Integration:**
```rust
// crates/atlas-runtime/src/stdlib/json.rs

use serde_json;

pub fn json_parse(input: &str, span: Span) -> Result<Value, RuntimeError> {
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(json_val) => {
            let json_data = convert_serde_to_atlas(json_val);
            Ok(Value::JsonValue(Rc::new(json_data)))
        }
        Err(e) => Err(RuntimeError::JsonParseError {
            code: "AT0110",
            message: format!("JSON parse error: {}", e),
            span,
        }),
    }
}

fn convert_serde_to_atlas(val: serde_json::Value) -> JsonData {
    match val {
        serde_json::Value::Number(n) => JsonData::Number(n.as_f64().unwrap()),
        serde_json::Value::String(s) => JsonData::String(s),
        serde_json::Value::Bool(b) => JsonData::Bool(b),
        serde_json::Value::Null => JsonData::Null,
        serde_json::Value::Array(arr) => {
            JsonData::Array(arr.into_iter().map(convert_serde_to_atlas).collect())
        }
        serde_json::Value::Object(obj) => {
            JsonData::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, convert_serde_to_atlas(v)))
                    .collect()
            )
        }
    }
}
```

### Serializer

**Library:** `serde_json`

**Integration:**
```rust
pub fn json_stringify(value: &Value, span: Span) -> Result<String, RuntimeError> {
    if let Value::JsonValue(json_data) = value {
        let serde_val = convert_atlas_to_serde(json_data);
        Ok(serde_json::to_string(&serde_val).unwrap())
    } else {
        Err(RuntimeError::InvalidStdlibArgument {
            code: "AT0102",
            message: "json::stringify requires JsonValue argument".to_string(),
            span,
        })
    }
}

fn convert_atlas_to_serde(data: &JsonData) -> serde_json::Value {
    match data {
        JsonData::Number(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n).unwrap()
        ),
        JsonData::String(s) => serde_json::Value::String(s.clone()),
        JsonData::Bool(b) => serde_json::Value::Bool(*b),
        JsonData::Null => serde_json::Value::Null,
        JsonData::Array(arr) => serde_json::Value::Array(
            arr.iter().map(convert_atlas_to_serde).collect()
        ),
        JsonData::Object(obj) => serde_json::Value::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), convert_atlas_to_serde(v)))
                .collect()
        ),
    }
}
```

### Type Checker Integration

**JsonValue Type:**
- Add `JsonValue` as a new type in the type system
- Type checking rules:
  - `json::parse()` returns `JsonValue`
  - `json::stringify()` accepts only `JsonValue`
  - `JsonValue` cannot be assigned to other types
  - `JsonValue` cannot be used in expressions (no operators)
  - Extraction functions convert `JsonValue` to Atlas types

**Type Representation:**
```rust
// crates/atlas-frontend/src/types.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Null,
    Void,
    Array(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    JsonValue,  // NEW: JSON-specific type
}
```

### Runtime Value Integration

**Already shown above in "Proposed Type System Extension" section.**

---

## Testing Requirements

### Parse Tests

**Valid JSON:**
```rust
#[rstest]
#[case("42", JsonData::Number(42.0))]
#[case("\"hello\"", JsonData::String("hello".to_string()))]
#[case("true", JsonData::Bool(true))]
#[case("false", JsonData::Bool(false))]
#[case("null", JsonData::Null)]
#[case("[1,2,3]", JsonData::Array(vec![...]))]
#[case("{\"a\":1}", JsonData::Object(...))]
fn test_json_parse_valid(#[case] input: &str, #[case] expected: JsonData) {
    let result = json_parse(input, Span::dummy());
    assert!(result.is_ok());
    // Assert expected value
}
```

**Invalid JSON:**
```rust
#[rstest]
#[case("{invalid}", "AT0110")]
#[case("[1,2,]", "AT0110")]
#[case("undefined", "AT0110")]
#[case("'hello'", "AT0110")]  // Single quotes
#[case("{a:1}", "AT0110")]    // Unquoted key
fn test_json_parse_invalid(#[case] input: &str, #[case] error_code: &str) {
    let result = json_parse(input, Span::dummy());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), error_code);
}
```

### Stringify Tests

**Round-trip:**
```rust
#[rstest]
#[case("{\"name\":\"Alice\",\"age\":30}")]
#[case("[1,2,3,4,5]")]
#[case("[1,\"two\",true,null]")]
#[case("{\"nested\":{\"deep\":{\"value\":42}}}")]
fn test_json_roundtrip(#[case] input: &str) {
    let parsed = json_parse(input, Span::dummy()).unwrap();
    let serialized = json_stringify(&parsed, Span::dummy()).unwrap();
    let reparsed = json_parse(&serialized, Span::dummy()).unwrap();

    // Assert parsed == reparsed (structure matches)
}
```

### Extraction Tests

```rust
#[test]
fn test_json_get_number() {
    let json = json_parse("42", Span::dummy()).unwrap();
    let num = json_get_number(&json, Span::dummy()).unwrap();
    assert_eq!(num, 42.0);
}

#[test]
fn test_json_get_number_wrong_type() {
    let json = json_parse("\"hello\"", Span::dummy()).unwrap();
    let result = json_get_number(&json, Span::dummy());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "AT0112");
}

#[test]
fn test_json_get_object_value() {
    let json = json_parse("{\"name\":\"Alice\"}", Span::dummy()).unwrap();
    let name_val = json_get_object_value(&json, "name", Span::dummy()).unwrap();
    let name = json_get_string(&name_val, Span::dummy()).unwrap();
    assert_eq!(name, "Alice");
}

#[test]
fn test_json_get_object_value_missing_key() {
    let json = json_parse("{\"name\":\"Alice\"}", Span::dummy()).unwrap();
    let result = json_get_object_value(&json, "age", Span::dummy());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "AT0113");
}
```

### Integration Tests

**REPL:**
```atlas
> import json;
> let data = json::parse("{\"x\":42}");
> let val = json::get_object_value(data, "x");
> let num = json::get_number(val);
> print(num);
42
```

**VM:**
- Ensure JsonValue works correctly in bytecode execution
- Verify no memory leaks with reference counting
- Test large JSON documents (100KB+)

---

## Performance Considerations

### Parsing Performance

**Target:** Parse 1MB JSON in <100ms on modern hardware

**Optimizations:**
- Use `serde_json` (highly optimized, SIMD in some cases)
- Lazy parsing not needed for v0.5 (future optimization)
- Consider streaming parser for very large files (v1.1+)

### Memory Usage

**Target:** Linear memory usage (O(n) where n = input size)

**Considerations:**
- `serde_json` creates intermediate AST (necessary for validation)
- Atlas `JsonData` duplicates structure (could share with `serde_json` in future)
- Reference counting for objects/arrays minimizes copies

### Serialization Performance

**Target:** Serialize 1MB structure in <50ms

**Optimizations:**
- `serde_json::to_string` is highly optimized
- Compact output (no pretty-printing by default)
- Pretty-printing can be added later: `json::stringify_pretty(value: JsonValue, indent: number) -> string`

---

## Security Considerations

### Denial of Service

**Attack:** Deeply nested JSON exhausts stack
```json
{{{{{{{{{... 10000 levels deep ...}}}}}}}}}
```

**Mitigation:**
- Set max nesting depth (128 levels, same as most browsers)
- Reject deeply nested input with AT0110 error
- `serde_json` has built-in depth limits

**Attack:** Large arrays/objects exhaust memory
```json
[1,1,1,1,... 1 billion elements ...]
```

**Mitigation:**
- Set max input size (10MB default, configurable via CLI flag)
- Reject oversized input before parsing
- Document memory requirements (10MB input ≈ 50MB RAM)

### Malicious Escape Sequences

**Attack:** Invalid Unicode escapes, overlong sequences
```json
"\uD800"  // Unpaired surrogate
"\uFFFF"  // Invalid character
```

**Mitigation:**
- `serde_json` validates all escape sequences per spec
- Invalid sequences rejected with AT0110 error
- No custom escape handling (avoid bugs)

### JSON Injection

**Not Applicable:**
- Atlas does not support string interpolation in JSON (no templating)
- All JSON input is parsed and validated
- Output is always escaped by `serde_json`

---

## Migration Path

### v0.5: Initial Implementation

**Deliverables:**
- `JsonValue` type added to runtime
- `json::parse()` and `json::stringify()` functions (global namespace)
- Extraction helpers (`json::get_number`, etc.)
- Error codes AT0110-AT0113
- Comprehensive tests

**Limitations:**
- No module system (functions in global namespace as `json_parse`, `json_stringify`)
- Cannot create `JsonValue` from Atlas values (parse-only)
- No pretty-printing

### v1.0: Module System Integration

**Enhancements:**
- Functions moved to `json::` module namespace
- `import json;` required
- Prelude does NOT auto-import JSON functions (explicit import required)

### v1.1+: Advanced Features

**Potential Additions:**
- `json::from_atlas(value: T) -> JsonValue` - convert Atlas values to JSON
- `json::stringify_pretty(value: JsonValue, indent: number) -> string`
- `json::merge(a: JsonValue, b: JsonValue) -> JsonValue` - merge objects
- `json::query(value: JsonValue, path: string) -> JsonValue` - JSONPath queries
- Streaming parser for very large files: `json::parse_stream(path: string) -> JsonValue[]`

---

## Alternative Designs Considered

### Alternative 1: Typed JSON Schemas

**Approach:** Define schemas and generate type-safe accessors

```atlas
schema UserSchema {
    name: string;
    age: number;
}

let data = json::parse<UserSchema>("{\"name\":\"Alice\",\"age\":30}");
print(data.name);  // Type-safe access
```

**Pros:**
- Type-safe at compile time
- Better IDE support
- Catches schema mismatches early

**Cons:**
- Requires schema definition language (significant complexity)
- Doesn't handle dynamic/unknown JSON structures
- Not AI-friendly (agents can't predict schemas)

**Decision:** Rejected for v0.5, possible for v2.0+

### Alternative 2: Defer Until Union Types

**Approach:** Wait for union types in language roadmap

```atlas
// Hypothetical v2.0 with unions
type Json = number | string | bool | null | Json[] | Map<string, Json>;

fn parse(input: string) -> Json { ... }
```

**Pros:**
- More elegant type system integration
- No special-casing for JSON
- Union types useful for other features

**Cons:**
- Delays JSON support (critical for AI workflows)
- Union types are complex (v2.0+ feature)
- Still need object/map type

**Decision:** Rejected. JSON is too important to defer.

### Alternative 3: String-Based Access Only

**Approach:** Keep JSON as strings, provide query helpers

```atlas
let json_str = "{\"name\":\"Alice\"}";
let name = json::query(json_str, "$.name");  // Returns "\"Alice\"" (still JSON)
```

**Pros:**
- No type system changes
- Simple implementation

**Cons:**
- Error-prone (no validation)
- Poor performance (re-parse on every access)
- No type safety at all

**Decision:** Rejected. Too limiting.

---

## Summary

**JSON stdlib design:**
1. ✅ **JsonValue Type** - Isolated dynamic type for JSON data only
2. ✅ **Parse/Stringify API** - Simple, familiar functions
3. ✅ **Extraction Helpers** - Convert JSON to Atlas types safely
4. ✅ **Error Handling** - Clear error codes (AT0110-AT0113)
5. ✅ **serde_json Library** - Battle-tested, secure, compliant
6. ✅ **Security** - Depth limits, size limits, validation

**Implementation Timeline:**
- **v0.5:** Core JSON support (this plan)
- **v1.0:** Module system integration
- **v1.1+:** Advanced features (pretty-print, merge, query)

**Next Steps:**
- ✅ JSON stdlib plan documented (this file)
- ⬜ Implement `JsonValue` type in runtime
- ⬜ Add `json::parse()` and `json::stringify()` functions
- ⬜ Add extraction helper functions
- ⬜ Write comprehensive tests
- ⬜ Update `docs/stdlib.md` with JSON functions
- ⬜ Add to `Cargo.toml`: `serde_json = "1.0"`

---

**References:**
- RFC 8259: The JSON Data Interchange Format
- `docs/stdlib-expansion-plan.md` - Stdlib roadmap
- `docs/io-security-model.md` - Security model for I/O
- `docs/stdlib.md` - Current stdlib specification
- `serde_json` documentation: https://docs.rs/serde_json/
