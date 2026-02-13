# BLOCKER 01: JSON Value Type

**Category:** Foundation - Value Model Extension
**Blocks:** Stdlib Phase 4 (JSON), Phase 10 (HTTP), and 10+ other phases
**Estimated Effort:** 1-2 weeks
**Complexity:** Medium

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Core value model and type system from v0.1 must be stable.

**Verification:**
```bash
grep -n "enum Value" crates/atlas-runtime/src/value.rs
grep -n "enum Type" crates/atlas-runtime/src/types.rs
cargo test --lib value
cargo test --lib types
```

**What's needed:**
- Value enum with basic types (Number, String, Bool, Null, Array, Function)
- Type system with type checking
- Interpreter and VM value handling
- All v0.1 tests passing

**If missing:** Should exist from v0.1

---

## Objective

Add `JsonValue` type to Atlas value model following the design documented in `docs/reference/decision-log.md`. This is the **only exception** to Atlas's strict typing - a controlled dynamic type specifically for JSON interop.

**Design principle:** Isolated dynamic typing for JSON only. Does not pollute the rest of the type system.

---

## Background

From `decision-log.md`:
- JsonValue is isolated from regular type system
- Cannot be assigned to non-JsonValue variables without explicit extraction
- Natural indexing: `data["user"]["name"]`
- Explicit extraction: `.as_string()`, `.as_number()`, etc.
- Returns `JsonValue::Null` for missing keys (safe, no crashes)

**Rationale:** JSON is critical for AI agent workflows (APIs, config, data interchange). Rust's `serde_json` pattern is proven and ergonomic.

---

## Files

### Create
- `crates/atlas-runtime/src/json_value.rs` (~400 lines)
  - JsonValue enum
  - Indexing operations
  - Type extraction methods
  - Equality/display implementations

### Modify
- `crates/atlas-runtime/src/value.rs` (~10 lines)
  - Add `JsonValue(Rc<JsonValue>)` variant
  - Update type_name(), Display, PartialEq

- `crates/atlas-runtime/src/types.rs` (~10 lines)
  - Add `Type::JsonValue`
  - Update display_name(), is_assignable_to()

- `crates/atlas-runtime/src/ast.rs` (~5 lines)
  - Add `TypeRef::Named("json", _)` support

- `crates/atlas-runtime/src/binder.rs` (~5 lines)
  - Resolve "json" type to Type::JsonValue

- `crates/atlas-runtime/src/typechecker/expr.rs` (~50 lines)
  - Type check json indexing
  - Type check extraction methods
  - Enforce isolation (cannot assign JsonValue to non-json typed vars)

- `crates/atlas-runtime/src/interpreter/expr.rs` (~30 lines)
  - Evaluate json indexing
  - Handle method calls on JsonValue

- `crates/atlas-runtime/src/compiler/expr.rs` (~30 lines)
  - Compile json indexing
  - Compile method calls

- `crates/atlas-runtime/src/vm/mod.rs` (~20 lines)
  - Execute json operations
  - Handle json value on stack

### Tests
- `crates/atlas-runtime/tests/json_value_tests.rs` (~600 lines)
  - JsonValue construction
  - Indexing (objects, arrays)
  - Type extraction methods
  - Error cases (wrong type extractions)
  - Null safety
  - Equality and display

- `crates/atlas-runtime/tests/vm_json_value_tests.rs` (~600 lines)
  - Identical tests for VM parity

**Minimum test count:** 100+ tests (50 interpreter, 50 VM)

---

## Implementation

### Step 1: JsonValue Core Type
Create `json_value.rs` with JsonValue enum containing 6 variants: Null, Bool, Number, String, Array(Vec<JsonValue>), Object(HashMap<String, JsonValue>). Implement constructor helpers, type checking methods (is_null, is_bool, etc.), extraction methods (as_number, as_string returning Option), and indexing operations that return Null for missing keys/invalid indices.

**Key principle:** Safe indexing - return JsonValue::Null for missing keys or out-of-bounds indices, never panic.

### Step 2: Value Integration
Add JsonValue(Rc<JsonValue>) variant to Value enum. Update type_name(), PartialEq, and Display implementations to handle json values. JsonValue should display in JSON format.

### Step 3: Type System Integration
Add Type::JsonValue variant. Update display_name() and is_assignable_to(). **Critical:** JsonValue is NOT assignable to any other type - enforce strict isolation. Only JsonValue can assign to JsonValue.

### Step 4: Parser/Binder Support
Add "json" to recognized type names in binder's resolve_type_ref(). The "json" keyword resolves to Type::JsonValue. No new syntax needed - uses existing TypeRef::Named path.

### Step 5: Type Checker Enforcement
Extend check_index to allow both string and number indices for JsonValue (returns JsonValue). Add isolation checks in check_var_decl: json values cannot be assigned to non-json typed variables without explicit extraction. Enforce this bidirectionally.

### Step 6: Interpreter Support
Extend eval_index to handle Value::JsonValue with string or number indices. Call JsonValue's index_str() or index_num() methods. Return Value::JsonValue wrapping the result (which may be Null).

### Step 7: VM Support
Compile json indexing operations in compiler/expr.rs. Emit appropriate opcodes for index operations. VM execution handles JsonValue on stack like other values.

### Step 8: Comprehensive Testing
Test JsonValue construction, object indexing (found/missing keys), array indexing (valid/invalid indices), nested indexing, type extraction, equality, display. Test isolation enforcement (compiler errors for improper assignments). Full interpreter/VM parity.

---

## Acceptance Criteria

**Functionality:**
- ✅ JsonValue enum with all 6 types (null, bool, number, string, array, object)
- ✅ Object indexing with string keys (returns null for missing)
- ✅ Array indexing with number indices (returns null for out of bounds)
- ✅ Type extraction methods (as_bool, as_number, etc.)
- ✅ Type checking methods (is_bool, is_number, etc.)
- ✅ Value::JsonValue variant integrated
- ✅ Type::JsonValue integrated
- ✅ Type checker enforces isolation (no json to non-json assignment without extraction)

**Quality:**
- ✅ 100+ tests pass (50+ interpreter, 50+ VM)
- ✅ 100% interpreter/VM parity
- ✅ Zero clippy warnings
- ✅ All code formatted (cargo fmt)
- ✅ json_value.rs under 500 lines
- ✅ Test files under 700 lines each

**Documentation:**
- ✅ Update decision-log.md with implementation notes
- ✅ Add examples to Atlas-SPEC.md
- ✅ Document json type in type system docs

**Integration:**
- ✅ Type system properly isolates JsonValue
- ✅ Runtime handles JsonValue in both engines
- ✅ No crashes on invalid operations (returns null safely)

---

## Architecture Notes

**Isolation is critical:** JsonValue must not leak into the rest of the type system. Cannot assign json to string, number, etc. without explicit extraction.

**Null safety:** Missing keys/indices return `JsonValue::Null`, not runtime errors. This matches JSON semantics and is safe.

**Performance:** Use Rc for sharing. JsonValue operations are not hot path (mostly used for config/API responses).

**Future:** When method call syntax is added, extraction will be:
```atlas
let name: string = data["user"]["name"].as_string();
```

For now, extraction needs builtin functions or special syntax (define in Phase 4: JSON API).

---

## Dependencies

**Requires:**
- v0.1 complete (value model, type system)
- First-class functions (for method calls later)

**Blocks:**
- Stdlib Phase 4: JSON Type Utilities
- Stdlib Phase 10: Network HTTP
- Any phase needing JSON data handling

---

## Testing Strategy

**Unit tests (in json_value.rs):**
- JsonValue enum operations
- Indexing logic
- Type extraction
- Edge cases

**Integration tests:**
- Atlas code using json type
- Type checker isolation
- Runtime behavior
- Error cases

**Parity tests:**
- Every test in interpreter version
- Identical test in VM version
- Verify exact same output

---

## Known Limitations

**No JSON literals yet:** This phase adds the type infrastructure. JSON parsing from strings comes in Stdlib Phase 4.

**No method calls yet:** Extraction via methods (`.as_string()`) requires method call syntax. May need builtin functions as intermediate:
```atlas
let s: string = json_as_string(data["name"]);
```

Or add to Phase 4: JSON API.

**No JSON serialization yet:** Converting Atlas values TO json comes in Phase 4.

---

## Rollout Plan

1. Implement JsonValue enum with tests
2. Integrate into Value enum
3. Integrate into Type system
4. Add type checking with isolation
5. Interpreter support
6. VM support
7. Comprehensive testing
8. Documentation

**Each step must be complete and tested before proceeding.**

No shortcuts. This is foundational.
