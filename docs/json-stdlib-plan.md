# Atlas JSON Stdlib Plan

**Version:** 2.0 (Revised for AI-Ergonomics)
**Status:** Design
**Target Release:** v0.5 (with JsonValue type)
**Last Updated:** 2026-02-12

---

## Overview

This document defines the design for JSON parsing and serialization in Atlas. JSON support is critical for AI agent workflows, API integration, and data interchange.

**Design Philosophy:** Follow **Rust's `serde_json`** pattern—the gold standard for ergonomic JSON in strictly-typed languages. Provide natural indexing syntax that AI agents expect while maintaining type safety through explicit extraction.

**Core Pattern:**
```atlas
let data = json::parse("{\"user\":{\"name\":\"Alice\",\"age\":30}}");
let name = data["user"]["name"].as_string();  // Natural + Safe!
let age = data["user"]["age"].as_number();
```

---

## Design Rationale: AI-First Ergonomics

### What AI Agents Actually Generate

**Python (most common):**
```python
data = json.loads(text)
name = data["user"]["name"]
```

**TypeScript:**
```typescript
const data = JSON.parse(text);
const name = data.user.name;
```

**Rust (strictly-typed, ergonomic):**
```rust
let data: Value = serde_json::from_str(text)?;
let name = data["user"]["name"].as_str().unwrap();
```

### Atlas Approach: Match Rust

Atlas follows Rust's pattern because it's:
- ✅ **Natural for AI** (indexing like Python/JS)
- ✅ **Strictly typed** (explicit extraction required)
- ✅ **Safe** (errors on type mismatch)
- ✅ **Concise** (one line, not nested calls)

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
    Array(Vec<JsonData>),               // Heterogeneous array
    Object(BTreeMap<String, JsonData>),  // Key-value map
}
```

### Type System Integration

**JsonValue Properties:**
- **Indexable:** Supports `[]` operator for objects (string keys) and arrays (number indices)
- **Isolated:** Cannot be assigned to regular Atlas variables without extraction
- **Explicit Conversion:** Must use `.as_string()`, `.as_number()`, etc. to convert to Atlas types
- **Chain-able:** `data["user"]["name"]` returns `JsonValue` at each step

**Rationale:**
- Keeps JSON data explicitly distinct from regular values
- Forces type checking at extraction boundaries
- Prevents accidental mixing of JSON and Atlas values
- Maintains strict typing while providing ergonomic access

---

## API Design

### Module: `json`

**Import (v1.0+):**
```atlas
import json;
```

**v0.5 (Pre-module system):**
Functions available in global namespace as `json_parse()` and `json_stringify()`.

---

## Core Functions

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

// Parse primitives
let num = json::parse("42");
let str = json::parse("\"hello\"");
let bool_val = json::parse("true");
let null_val = json::parse("null");

// Parse arrays
let arr = json::parse("[1, 2, 3]");
let mixed = json::parse("[1, \"two\", true, null]");

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

---

### Function: `json::stringify`

**Signature:**
```atlas
fn stringify(value: JsonValue) -> string
```

**Behavior:**
- Serializes `JsonValue` to JSON string
- Output is compact (no whitespace)
- Escape sequences per JSON spec

**Examples:**
```atlas
import json;

let data = json::parse("{\"name\":\"Alice\",\"age\":30}");
let json_str = json::stringify(data);
print(json_str);  // {"name":"Alice","age":30}
```

---

## Indexing Operations

### Array Indexing: `jsonvalue[index]`

**Syntax:**
```atlas
let element = jsonvalue[0];  // Returns JsonValue
```

**Behavior:**
- `index` must be a `number` (compile-time type check)
- Returns `JsonValue` at that index
- If index out of bounds, returns `JsonValue::Null` (like Rust)
- If `jsonvalue` is not an array, returns `JsonValue::Null`

**Examples:**
```atlas
let arr = json::parse("[10, 20, 30]");
let first = arr[0].as_number();   // 10
let second = arr[1].as_number();  // 20

// Out of bounds → null
let missing = arr[99].is_null();  // true

// Not an array → null
let obj = json::parse("{\"a\":1}");
let invalid = obj[0].is_null();   // true
```

### Object Indexing: `jsonvalue["key"]`

**Syntax:**
```atlas
let value = jsonvalue["key"];  // Returns JsonValue
```

**Behavior:**
- `key` must be a `string` (compile-time type check)
- Returns `JsonValue` for that key
- If key doesn't exist, returns `JsonValue::Null` (like Rust)
- If `jsonvalue` is not an object, returns `JsonValue::Null`

**Examples:**
```atlas
let obj = json::parse("{\"name\":\"Alice\",\"age\":30}");
let name = obj["name"].as_string();  // "Alice"
let age = obj["age"].as_number();    // 30

// Missing key → null
let missing = obj["email"].is_null();  // true

// Not an object → null
let arr = json::parse("[1,2,3]");
let invalid = arr["key"].is_null();    // true
```

### Chaining Indexing

**Nested Access:**
```atlas
let data = json::parse("{
    \"user\": {
        \"name\": \"Alice\",
        \"scores\": [95, 87, 92]
    }
}");

// Chain object and array indexing
let name = data["user"]["name"].as_string();        // "Alice"
let first_score = data["user"]["scores"][0].as_number();  // 95

// Safe navigation with null checks
let email = data["user"]["email"];
if (!email.is_null()) {
    print(email.as_string());
} else {
    print("No email");
}
```

---

## Type Extraction Methods

All extraction methods are **instance methods** on `JsonValue` (like Rust's approach).

### `.as_string() -> string`

**Behavior:**
- Extracts string from JsonValue
- Throws `AT0112` if value is not a JSON string

**Example:**
```atlas
let data = json::parse("\"hello\"");
let s = data.as_string();  // "hello"

let num = json::parse("42");
let bad = num.as_string();  // Error: AT0112
```

---

### `.as_number() -> number`

**Behavior:**
- Extracts number from JsonValue
- Throws `AT0112` if value is not a JSON number

**Example:**
```atlas
let data = json::parse("42.5");
let n = data.as_number();  // 42.5

let str = json::parse("\"hello\"");
let bad = str.as_number();  // Error: AT0112
```

---

### `.as_bool() -> bool`

**Behavior:**
- Extracts boolean from JsonValue
- Throws `AT0112` if value is not a JSON boolean

**Example:**
```atlas
let data = json::parse("true");
let b = data.as_bool();  // true

let num = json::parse("1");
let bad = num.as_bool();  // Error: AT0112
```

---

### `.is_null() -> bool`

**Behavior:**
- Returns `true` if JsonValue is null, `false` otherwise
- Never throws an error

**Example:**
```atlas
let null_val = json::parse("null");
let is_null = null_val.is_null();  // true

let num = json::parse("42");
let not_null = num.is_null();      // false

// Check for missing keys
let obj = json::parse("{\"a\":1}");
let missing = obj["b"].is_null();  // true
```

---

### `.as_array() -> JsonValue[]`

**Behavior:**
- Returns array of `JsonValue` elements
- Throws `AT0112` if value is not a JSON array
- Each element is still a `JsonValue` (needs further extraction)

**Example:**
```atlas
let data = json::parse("[1, \"two\", true]");
let arr = data.as_array();  // JsonValue[]

// Extract elements
let first = arr[0].as_number();   // 1
let second = arr[1].as_string();  // "two"
let third = arr[2].as_bool();     // true

// Iterate (requires for-each support, future feature)
// for (let item in arr) { ... }
```

---

### `.len() -> number`

**Behavior:**
- Returns length of JSON array or number of keys in JSON object
- Returns `0` for primitives (number, string, bool, null)

**Example:**
```atlas
let arr = json::parse("[1,2,3]");
let arr_len = arr.len();  // 3

let obj = json::parse("{\"a\":1,\"b\":2}");
let obj_len = obj.len();  // 2

let num = json::parse("42");
let num_len = num.len();  // 0
```

---

## Complete Usage Example

```atlas
import json;

// Parse API response
let response = json::parse("{
    \"status\": \"success\",
    \"data\": {
        \"users\": [
            {\"id\": 1, \"name\": \"Alice\", \"email\": \"alice@example.com\"},
            {\"id\": 2, \"name\": \"Bob\", \"email\": \"bob@example.com\"}
        ],
        \"total\": 2
    }
}");

// Extract fields naturally
let status = response["status"].as_string();
print(status);  // "success"

let total = response["data"]["total"].as_number();
print(total);  // 2

// Access array elements
let users = response["data"]["users"].as_array();
let first_user = users[0];
let first_name = first_user["name"].as_string();
let first_email = first_user["email"].as_string();

print(first_name);   // "Alice"
print(first_email);  // "alice@example.com"

// Safe access with null check
let metadata = response["metadata"];
if (metadata.is_null()) {
    print("No metadata");
} else {
    print(metadata.as_string());
}

// Round-trip
let serialized = json::stringify(response);
print(serialized);  // Compact JSON string
```

---

## Error Codes

### AT0110: JSON Parse Error

**Cause:** Invalid JSON syntax

**Examples:**
```atlas
json::parse("{invalid}")           // Missing quotes
json::parse("[1, 2,]")             // Trailing comma
json::parse("{\"a\": undefined}")  // Not valid JSON
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

---

### AT0112: JSON Type Mismatch

**Cause:** Attempting to extract wrong type from JsonValue

**Examples:**
```atlas
let data = json::parse("\"hello\"");
let num = data.as_number();  // Error: data is string, not number

let data = json::parse("42");
let arr = data.as_array();   // Error: data is number, not array
```

**Error Format:**
```
runtime error[AT0112]: JSON type mismatch: expected number, found string
  --> script.atl:10:11
   |
10 | let num = data.as_number();
   |           ^^^^^^^^^^^^^^^^ expected JSON number
   |
   = note: value is a JSON string: "hello"
   = help: use .as_string() to extract string values
```

---

## Implementation Requirements

### JsonValue Indexing

**Add index operators to JsonValue:**

```rust
// crates/atlas-runtime/src/stdlib/json.rs

impl JsonData {
    /// Index with string key (object access)
    pub fn index_string(&self, key: &str) -> JsonData {
        match self {
            JsonData::Object(map) => map.get(key).cloned().unwrap_or(JsonData::Null),
            _ => JsonData::Null,
        }
    }

    /// Index with number (array access)
    pub fn index_number(&self, index: f64) -> JsonData {
        if index.fract() != 0.0 || index < 0.0 {
            return JsonData::Null;  // Non-integer or negative
        }

        match self {
            JsonData::Array(arr) => {
                let idx = index as usize;
                arr.get(idx).cloned().unwrap_or(JsonData::Null)
            }
            _ => JsonData::Null,
        }
    }
}
```

### Type Extraction Methods

```rust
impl JsonData {
    pub fn as_string(&self, span: Span) -> Result<String, RuntimeError> {
        match self {
            JsonData::String(s) => Ok(s.clone()),
            _ => Err(RuntimeError::JsonTypeMismatch {
                code: "AT0112",
                expected: "string",
                found: self.type_name(),
                span,
            }),
        }
    }

    pub fn as_number(&self, span: Span) -> Result<f64, RuntimeError> {
        match self {
            JsonData::Number(n) => Ok(*n),
            _ => Err(RuntimeError::JsonTypeMismatch {
                code: "AT0112",
                expected: "number",
                found: self.type_name(),
                span,
            }),
        }
    }

    pub fn as_bool(&self, span: Span) -> Result<bool, RuntimeError> {
        match self {
            JsonData::Bool(b) => Ok(*b),
            _ => Err(RuntimeError::JsonTypeMismatch {
                code: "AT0112",
                expected: "bool",
                found: self.type_name(),
                span,
            }),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, JsonData::Null)
    }

    pub fn as_array(&self, span: Span) -> Result<Vec<JsonData>, RuntimeError> {
        match self {
            JsonData::Array(arr) => Ok(arr.clone()),
            _ => Err(RuntimeError::JsonTypeMismatch {
                code: "AT0112",
                expected: "array",
                found: self.type_name(),
                span,
            }),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            JsonData::Array(arr) => arr.len(),
            JsonData::Object(map) => map.len(),
            _ => 0,
        }
    }

    fn type_name(&self) -> &str {
        match self {
            JsonData::Number(_) => "number",
            JsonData::String(_) => "string",
            JsonData::Bool(_) => "bool",
            JsonData::Null => "null",
            JsonData::Array(_) => "array",
            JsonData::Object(_) => "object",
        }
    }
}
```

### Parser (Using serde_json)

```rust
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

### Type Checker Integration

**Add JsonValue indexing to type checker:**

```rust
// When type-checking index expression: jsonvalue[key]

match (base_type, index_type) {
    (Type::JsonValue, Type::String) => {
        // Object indexing: returns JsonValue
        Ok(Type::JsonValue)
    }
    (Type::JsonValue, Type::Number) => {
        // Array indexing: returns JsonValue
        Ok(Type::JsonValue)
    }
    (Type::JsonValue, other) => {
        Err(Diagnostic::error(
            "AT0001",
            &format!("JSON index must be string or number, found {}", other),
            index_span
        ))
    }
    // ... existing array indexing logic
}
```

**Add method call support for `.as_string()`, `.as_number()`, etc.**

---

## Testing Requirements

### Parse Tests

```rust
#[rstest]
#[case("42", JsonData::Number(42.0))]
#[case("\"hello\"", JsonData::String("hello".to_string()))]
#[case("true", JsonData::Bool(true))]
#[case("null", JsonData::Null)]
fn test_json_parse_primitives(#[case] input: &str, #[case] expected: JsonData) {
    let result = json_parse(input, Span::dummy()).unwrap();
    if let Value::JsonValue(data) = result {
        assert_eq!(*data, expected);
    } else {
        panic!("Expected JsonValue");
    }
}
```

### Indexing Tests

```rust
#[test]
fn test_json_object_indexing() {
    let json = json_parse("{\"name\":\"Alice\",\"age\":30}", Span::dummy()).unwrap();
    let Value::JsonValue(data) = json else { panic!() };

    let name = data.index_string("name");
    assert_eq!(name, JsonData::String("Alice".to_string()));

    let age = data.index_number(30.0);
    assert_eq!(age, JsonData::Null);  // Not an array
}

#[test]
fn test_json_array_indexing() {
    let json = json_parse("[10,20,30]", Span::dummy()).unwrap();
    let Value::JsonValue(data) = json else { panic!() };

    let first = data.index_number(0.0);
    assert_eq!(first, JsonData::Number(10.0));

    let missing = data.index_number(99.0);
    assert_eq!(missing, JsonData::Null);
}

#[test]
fn test_json_chained_indexing() {
    let json = json_parse(
        "{\"user\":{\"name\":\"Alice\",\"scores\":[95,87,92]}}",
        Span::dummy()
    ).unwrap();
    let Value::JsonValue(data) = json else { panic!() };

    let user = data.index_string("user");
    let scores = user.index_string("scores");
    let first_score = scores.index_number(0.0);

    assert_eq!(first_score, JsonData::Number(95.0));
}
```

### Extraction Tests

```rust
#[test]
fn test_as_string_success() {
    let data = JsonData::String("hello".to_string());
    let result = data.as_string(Span::dummy()).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_as_string_type_mismatch() {
    let data = JsonData::Number(42.0);
    let result = data.as_string(Span::dummy());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "AT0112");
}

#[test]
fn test_is_null() {
    assert!(JsonData::Null.is_null());
    assert!(!JsonData::Number(42.0).is_null());
    assert!(!JsonData::String("".to_string()).is_null());
}
```

### Integration Tests (REPL)

```atlas
> import json;
> let data = json::parse("{\"x\":42}");
> let val = data["x"];
> let num = val.as_number();
> print(num);
42

> let obj = json::parse("{\"user\":{\"name\":\"Alice\"}}");
> let name = obj["user"]["name"].as_string();
> print(name);
Alice

> let missing = obj["user"]["email"].is_null();
> print(missing);
true
```

---

## JSON Spec Compliance

Same as before (RFC 8259 compliance via `serde_json`).

---

## Security Considerations

Same as before (depth limits, size limits via `serde_json`).

---

## Comparison with Other Languages

### Rust (Our Model)

```rust
let data: Value = serde_json::from_str(json_str)?;
let name = data["user"]["name"].as_str().unwrap();
```

**Atlas Equivalent:**
```atlas
let data = json::parse(json_str);
let name = data["user"]["name"].as_string();
```

✅ **Nearly identical!** This is exactly what AI agents expect.

### Python

```python
data = json.loads(json_str)
name = data["user"]["name"]
```

**Atlas is almost as concise**, just adds explicit `.as_string()` for type safety.

### TypeScript

```typescript
const data = JSON.parse(json_str);
const name = data.user.name;  // Property access
```

**Atlas uses `[]` for all access** (consistent with arrays), slightly different from TypeScript's `.` notation.

---

## Migration Path

### v0.5: Initial Implementation

**Deliverables:**
- `JsonValue` type with indexing support (`[]` operator)
- `json::parse()` and `json::stringify()` functions
- Extraction methods (`.as_string()`, `.as_number()`, `.as_bool()`, `.is_null()`, `.as_array()`)
- `.len()` method for arrays/objects
- Error codes AT0110, AT0112
- Comprehensive tests

### v1.0: Module System Integration

- Functions moved to `json::` module namespace
- `import json;` required

### v1.1+: Advanced Features

- Pretty printing: `json::stringify_pretty(value, indent)`
- Object key iteration (requires language support)
- Array iteration helpers

---

## Summary

**Atlas JSON design follows Rust's `serde_json` pattern:**
1. ✅ **Natural Indexing** - `data["user"]["name"]` (AI-friendly)
2. ✅ **Explicit Extraction** - `.as_string()` (type-safe)
3. ✅ **Null Safety** - `.is_null()` check (no crashes)
4. ✅ **Strict Typing** - Cannot mix JsonValue with Atlas types
5. ✅ **Ergonomic** - Concise, readable, natural for AI agents

**Implementation Timeline:**
- **v0.5:** Core JSON support (this plan)
- **v1.0:** Module system integration
- **v1.1+:** Advanced features

**Next Steps:**
- ✅ JSON stdlib plan documented (this file)
- ⬜ Implement `JsonValue` type with indexing
- ⬜ Add extraction methods (`.as_*()`)
- ⬜ Update type checker for JsonValue indexing
- ⬜ Write comprehensive tests
- ⬜ Update `docs/stdlib.md`
- ⬜ Add `serde_json = "1.0"` to `Cargo.toml`

---

**References:**
- RFC 8259: The JSON Data Interchange Format
- Rust `serde_json` docs: https://docs.rs/serde_json/
- `docs/stdlib-expansion-plan.md` - Stdlib roadmap
- `docs/io-security-model.md` - Security model
