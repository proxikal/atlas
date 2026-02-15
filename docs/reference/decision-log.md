# Atlas Decision Log

**⚠️  DEPRECATED:** This file has been migrated to `docs/decision-logs/` for better organization and AI-friendliness.

**New location:** See `docs/decision-logs/README.md` for the new structure and search guide.

**Migration complete:** All decisions below have been migrated to categorized decision files.

---

## Legacy Content (Reference Only)

This log captures irreversible or high-impact design decisions. Update when a new foundational decision is made.

## Language Semantics
- Strict typing, no implicit any or nullable. (TypeScript-style)
- No truthy/falsey coercion; conditionals require `bool`.
- `number` is IEEE 754 f64, but invalid numeric results are runtime errors.
- `+` supports `number + number` and `string + string` only.
- Comparisons `< <= > >=` only for `number`.
- Equality requires same-type operands; strings compare by value, arrays by reference identity.
- Top-level statements execute in order.
- Function declarations are top-level only (v0.1) and hoisted.
- Variables must be declared before use (no forward reference).
- `break`/`continue` only inside loops; `return` only inside functions.

## Number Literals
- Format: `digit { digit } [ "." digit { digit } ] [ ("e"|"E") ["+" | "-"] digit { digit } ]`
- Supports: Integer (`123`), decimal (`3.14`), and scientific notation (`1e10`, `1.5e-3`, `2.5E+10`)
- Rationale: AI-friendliness. Scientific notation is far more readable and token-efficient than 300+ digit literals. As an AI-first language, this improves both human and AI code generation/understanding.
- Lexer validates: Exponent must have at least one digit (e.g., `1e` or `1e+` are errors)

## Runtime Model
- Single shared `Value` enum across interpreter and VM.
- Reference counting (`Rc/Arc`), no GC in v0.1.
- Strings immutable; arrays mutable and shared by reference.
- Function arguments are passed by value with shared references for heap types.

## Security Architecture
- `SecurityContext` threads through runtime via raw pointer in Interpreter/VM.
- Rationale: Avoids lifetime complexity while maintaining security checks for I/O operations.
- `call_builtin()` accepts `&SecurityContext` parameter for permission enforcement.
- Interpreter/VM store `current_security: Option<*const SecurityContext>` set during `eval()`/`run()`.
- Pattern: Pass SecurityContext to `eval()`/`run()`, access via unsafe deref in stdlib functions.
- Safe because: SecurityContext lifetime guaranteed valid for duration of eval/run execution.

## Diagnostics
- Unified `Diagnostic` type with human + JSON formats.
- Errors emitted before warnings.
- Diagnostics include `diag_version` and optional related spans.

## Warning Implementation (Unused Variables/Code)
- TypeChecker maintains internal `declared_symbols` and `used_symbols` tracking per function.
- Rationale: Binder creates/destroys scopes during binding phase, before type checking runs. Scopes are gone by the time TypeChecker needs to check for unused symbols.
- Symbol table remains immutable during type checking; usage tracking is TypeChecker's responsibility.
- Symbol struct has no `used` field - keeps Symbol focused on type information, TypeChecker focused on analysis.
- Warnings emitted at end of each function scope, not globally.
- AI-friendly: Clear separation of concerns, no unused fields to cause confusion.

## Prelude
- Built-ins `print`, `len`, `str` always in scope.
- Global shadowing of prelude names is illegal (`AT1012`).

## Bytecode
- `.atb` format defined in `docs/bytecode-format.md`.
- Debug info is emitted by default.

## JSON Support (v0.5+)
- JsonValue is the **only** exception to "no dynamic types" principle.
- Rationale: JSON is critical for AI agent workflows (APIs, config files, data interchange).
- Design follows **Rust's `serde_json`** pattern (ergonomic + type-safe):
  - Natural indexing: `data["user"]["name"]` (like Python/JS)
  - Explicit extraction: `.as_string()`, `.as_number()`, etc. (maintains type safety)
  - Returns `JsonValue::Null` for missing keys/indices (safe, no crashes)
- JsonValue is **isolated** from regular type system:
  - Cannot be assigned to non-JsonValue variables without extraction
  - Cannot be used in expressions (`json + 1` is type error)
  - Forces type checking at extraction boundaries
- Alternative approaches rejected:
  - ❌ General-purpose `any` type (violates strict typing principle)
  - ❌ Wait for union types (delays critical feature, union types complex)
  - ❌ Schema-based only (too rigid for dynamic APIs)
- Trade-off: Accept controlled dynamic typing for JSON to maintain AI-friendliness while preserving strict typing elsewhere.

**Implementation (v0.2):**
- JsonValue enum: 6 variants (Null, Bool, Number, String, Array, Object)
- Value::JsonValue(Rc<JsonValue>) variant for runtime values
- Type::JsonValue for type system
- Isolation enforced via Type::is_assignable_to() - only json->json allowed
- Safe indexing: index_str() and index_num() methods return JsonValue::Null for missing/invalid
- Both interpreter and VM support json[string|number] indexing
- Type checker allows both string and number indices, always returns Type::JsonValue
- 21 integration tests verify behavior and isolation

## Method Call Syntax (v0.2)
- **Syntax:** `value.method(args)` desugars to `Type::method(value, args)`
- **Rationale:** Rust-style approach - AI-friendly (zero ambiguity), type-safe (compile-time resolution), zero-cost abstraction
- **Design:** Methods are functions with special syntax. No runtime lookup, no prototype chains, no `this` binding complexity
- **Dual syntax:** Both `value.method()` and `Type::method(value)` valid - AI can use either form
- **Alternative approaches rejected:**
  - ❌ Python-style (everything-is-object adds magic, runtime overhead)
  - ❌ JavaScript prototype chains (implicit behavior, `this` binding issues)
  - ❌ Go interfaces (implicit satisfaction not AI-friendly)
- **Implementation:** Built-in methods for stdlib types (json, string, array). Trait system for user-defined methods in v0.3+

## Generic Types - Monomorphization (v0.2)
- **Strategy:** Monomorphization (Rust-style) - generate specialized code for each type instantiation
- **Rationale:** Performance and type safety. Follows Rust's proven approach.
- **Alternative approaches rejected:**
  - ❌ Type erasure (Java-style) - loses type information at runtime, worse performance
  - ❌ Runtime dispatch (Go-style) - requires interface boxing, slower execution
  - ❌ Template-only (C++-style) - code bloat without caching, harder to debug
- **Implementation (BLOCKER 02-C):**
  - Monomorphizer caches specialized instances: `(function_name, type_args) -> substitution_map`
  - Name mangling for VM dispatch: `identity<number>` → `identity$number`
  - Type inference (BLOCKER 02-B) determines concrete types at compile time
  - Both interpreter and VM use same monomorphization infrastructure
  - Interpreter can stay polymorphic (tracks values), VM requires bytecode generation per instance
- **Trade-offs:**
  - **Pro:** Zero runtime overhead, full type safety, proven in production (Rust, C++)
  - **Pro:** Easy debugging - each specialization is standalone code
  - **Con:** Code bloat for many instantiations (mitigated by caching)
  - **Con:** Longer compile times for generic-heavy code
- **Status:** Infrastructure complete (BLOCKER 02-C). Full pipeline in BLOCKER 02-D (Option<T>, Result<T,E>).

## Array API - Intrinsics vs Stdlib (v0.2)
- **Strategy:** Split array functions by callback requirements
- **Rationale:** Callback-based functions need runtime execution context to invoke user code
- **Implementation:**
  - **Pure functions** (10): Implemented in `stdlib/array.rs` - pop, shift, unshift, reverse, concat, flatten, indexOf, lastIndexOf, includes, slice
  - **Callback intrinsics** (11): Implemented in interpreter/VM directly - map, filter, reduce, forEach, find, findIndex, flatMap, some, every, sort, sortBy
- **Real-world precedent:**
  - V8 (JavaScript): `Array.prototype.map/filter/reduce` implemented as C++ runtime intrinsics
  - CPython: `map()`, `filter()` implemented as builtin types in C
  - Rust: Iterator methods like `map/filter` are compiler intrinsics for optimization
- **Trade-offs:**
  - **Pro:** Maintains clean stdlib interface, each engine uses native calling mechanism
  - **Pro:** No complex abstraction layer needed for 2-engine architecture
  - **Pro:** Matches production compiler patterns
  - **Con:** Code in two places (but already Atlas's dual-engine architecture)
- **Alternative rejected:** Create function-caller trait → adds complexity without benefit for 2 engines in same codebase
- **Status:** Phase stdlib/phase-02-complete-array-api.md (v0.2)
