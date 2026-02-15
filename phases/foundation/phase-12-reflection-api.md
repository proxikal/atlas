# Phase 12: Reflection and Introspection API

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Type system and runtime from v0.1 for reflection extension.

**Verification Steps:**
1. Check v0.1 completion: STATUS.md should confirm type system and runtime complete

2. Verify type system exists:
   ```bash
   grep -n "pub enum Type" crates/atlas-runtime/src/typechecker/types.rs | head -3
   cargo test typechecker 2>&1 | grep "test result"
   ```

3. Verify Value enum exists with type info:
   ```bash
   grep -n "pub enum Value" crates/atlas-runtime/src/value.rs | head -3
   cargo test value 2>&1 | grep "test result"
   ```

4. Verify runtime exists:
   ```bash
   ls crates/atlas-runtime/src/interpreter/mod.rs
   ls crates/atlas-runtime/src/vm/mod.rs
   cargo test interpreter vm 2>&1 | grep "test result"
   ```

**Expected from v0.1 (sufficient for reflection):**
- Type enum with all Atlas types (Number, String, Bool, Array, Object, Function, etc.)
- Value enum with runtime representations
- Type checker producing type information
- Interpreter and VM executing code

**Spec Check:**
- Check `docs/specification/types.md` for reflection requirements
- If spec defines reflection API: Implement per spec
- If spec doesn't define it: Reflection is NEW for v0.2

**Decision Tree:**

a) If v0.1 type system and runtime exist (Type and Value enums found):
   → Proceed with phase-12
   → Add reflection API on top of existing infrastructure
   → Extend Value to include type metadata if needed

b) If type system missing:
   → CRITICAL ERROR: v0.1 incomplete
   → Verify v0.1 completion in STATUS.md
   → STOP immediately

c) If spec defines reflection (check types.md):
   → Read spec section on reflection completely
   → Implement reflection API exactly per spec
   → Log: "Implemented reflection per specification"

d) If spec doesn't define reflection:
   → Reflection API is NEW for this phase
   → Design minimal type introspection API
   → Functions: typeof(), get_type(), has_field(), etc.
   → Document design decisions

**No user questions needed:** v0.1 infrastructure is verifiable via file checks and cargo test. If spec silent on reflection, implement minimal introspection API.

---

## Objective
Implement reflection and introspection API enabling runtime inspection of types, values, and program structure - supporting metaprogramming, serialization frameworks, and dynamic tooling for advanced Atlas applications.

## Files
**Create:** `crates/atlas-runtime/src/reflect/mod.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/reflect/type_info.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/reflect/value_info.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/stdlib/reflect.rs` (~600 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~150 lines type metadata)
**Create:** `docs/reflection.md` (~700 lines)
**Tests:** `crates/atlas-runtime/tests/reflection_tests.rs` (~600 lines)

## Dependencies
- Type system with runtime type info
- Value model with metadata
- Stdlib for reflection functions
- Module system for code inspection

## Implementation

### Runtime Type Information
Attach type metadata to all values at runtime. Store complete type information not just tags. Type info includes type name, kind enum struct function. For compound types store element types. For functions store parameter types and return type. For structs store field names and types. Make type info queryable from values. Minimal memory overhead for type metadata. Type info equality comparison. Serialize type info for debugging.

### Type Inspection Functions
Provide typeof function returning type name as string. is_type function checking value against type. get_type function returning TypeInfo object. has_field function checking struct field existence. get_field_type function returning field type. get_function_signature function returning signature. is_callable function checking if value is function. is_primitive function checking basic types.

### Value Inspection API
Enable inspection of value contents. get_fields function listing struct field names. get_field function accessing field by name. get_length function for arrays and strings. is_empty function for collections. get_keys function for future map types. get_values function for future map types. Deep inspection for nested structures. Iteration over collection contents.

### Type Metadata Objects
Create TypeInfo objects exposing type details. name property with type name. kind property with type category. fields property for struct types. parameters property for function types. return_type property for functions. element_type property for arrays. Display TypeInfo as readable string. Compare TypeInfo for equality. Pattern match on TypeInfo kind.

### Constructor and Type Creation
Provide functions for dynamic type operations. construct function creating values from type and data. cast function attempting type conversion. clone function deep copying values. equals function checking value equality. hash function computing value hash. to_string function converting any value to string. from_string function parsing strings to values.

### Module and Code Reflection
Inspect loaded modules and definitions. list_modules function returning loaded module names. get_module function returning module info. get_exports function listing exported symbols. get_imports function listing imported symbols. get_functions function listing defined functions. get_types function listing defined types. get_globals function listing global variables. Module metadata with source location.

### Practical Applications
Enable powerful use cases with reflection. Generic serialization to JSON without manual code. Validation frameworks using type constraints. Test frameworks discovering test functions. Dependency injection using type matching. Configuration binding to structs. Mock object creation for testing. Dynamic dispatch based on types. API documentation generation from types.

## Tests (TDD - Use rstest)

**Type inspection tests:**
1. typeof returns correct type name
2. is_type validates type correctly
3. get_type returns TypeInfo
4. TypeInfo for primitive types
5. TypeInfo for compound types
6. TypeInfo for functions
7. TypeInfo equality
8. TypeInfo display format

**Value inspection tests:**
1. get_fields lists struct fields
2. get_field accesses field value
3. has_field checks existence
4. get_length for arrays
5. get_length for strings
6. is_empty for collections
7. Deep nested inspection
8. Inspect function closures

**Type metadata tests:**
1. TypeInfo name property
2. TypeInfo kind property
3. Struct field types
4. Function parameter types
5. Function return type
6. Array element type
7. Nullable type handling

**Dynamic operations tests:**
1. construct value from type
2. cast between compatible types
3. clone deep copy
4. equals value comparison
5. hash computation
6. to_string conversion
7. from_string parsing

**Module reflection tests:**
1. list_modules loaded modules
2. get_exports from module
3. get_imports from module
4. get_functions in module
5. Module source location
6. Symbol lookup by name

**Use case tests:**
1. Serialize value to JSON using reflection
2. Deserialize JSON using types
3. Validate struct fields
4. Discover test functions by name pattern
5. Create mock object
6. Bind configuration to struct
7. Generate API documentation

**Integration tests:**
1. Reflection with interpreter
2. Reflection with VM
3. Performance overhead measurement
4. Type safety preservation
5. Memory usage with metadata

**Minimum test count:** 80 tests

## Integration Points
- Uses: Type system from typechecker
- Uses: Value model from value.rs
- Uses: Module system from phase-06
- Updates: Value with type metadata
- Creates: Reflection API
- Creates: Introspection stdlib module
- Output: Runtime metaprogramming capability

## Acceptance
- typeof and get_type work for all types
- TypeInfo objects expose complete type details
- Value inspection functions access contents
- Dynamic construction and casting work
- Module reflection lists exports and imports
- Reflection enables serialization frameworks
- Performance overhead acceptable under 10 percent
- Type safety maintained with reflection
- 80+ tests pass
- Documentation with use case examples
- Metaprogramming guide complete
- No clippy warnings
- cargo test passes
