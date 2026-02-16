-- Migration: Add existing decisions and features to atlas-dev.db
-- Date: 2026-02-15
-- One-time migration to populate decisions and features tables

-- ============================================================
-- DECISIONS (16 total)
-- ============================================================

-- Language decisions (4)
INSERT INTO decisions (id, component, title, decision, rationale, status, date, created_at, updated_at)
VALUES
('language-DR-001', 'language', 'Strict Type System with No Implicit Coercion',
 'Strict typing with zero implicit coercion: No implicit any or nullable types, no truthy/falsy coercion, conditionals require bool, + supports only number+number and string+string, comparisons only for numbers, equality requires same-type operands',
 'AI-first principle: Explicit behavior reduces ambiguity for AI code generation. LLMs can reason precisely about type behavior without guessing coercion rules. Catches errors at compile time. Follows TypeScript strict mode, Rust, Go approach.',
 'accepted', '2024-01-15', datetime('now'), datetime('now')),

('language-DR-002', 'language', 'Scientific Notation for Number Literals',
 'Support scientific notation in number literals: Format digit{digit}[.digit{digit}][(e|E)[+|-]digit{digit}]. Examples: 123, 3.14, 1e10, 1.5e-3, 2.5E+10',
 'AI-friendliness: Scientific notation far more readable and token-efficient than 300+ digit literals. For AI-first language, improves both human and AI code generation. Industry standard in every production language.',
 'accepted', '2024-01-20', datetime('now'), datetime('now')),

('language-DR-003', 'language', 'Method Call Syntax - Rust-Style Desugaring',
 'Syntax value.method(args) desugars to Type::method(value, args). Methods are functions with special syntax. No runtime lookup, no prototype chains, no this binding complexity. Both value.method() and Type::method(value) valid.',
 'AI-friendly: Zero ambiguity - compile-time resolution, no runtime magic. Type-safe: Methods resolved during type checking. Zero-cost abstraction: Pure syntactic sugar compiles to direct function calls. Rust precedent proven in production.',
 'accepted', '2024-11-20', datetime('now'), datetime('now')),

('language-DR-004', 'language', 'Prelude with Shadowing Protection',
 'Built-ins print, len, str always in scope (prelude). Global shadowing of prelude names is illegal (AT1012 diagnostic).',
 'Developer experience: Core functions available immediately, no boilerplate imports. Safety: Prevent accidental shadowing that breaks built-in behavior. AI-friendly: Prelude predictable - AI knows names always available and cannot be redefined.',
 'accepted', '2024-02-20', datetime('now'), datetime('now'));

-- Runtime decisions (2)
INSERT INTO decisions (id, component, title, decision, rationale, status, date, created_at, updated_at)
VALUES
('runtime-DR-001', 'runtime', 'Shared Value Enum with Reference Counting',
 'Single shared Value enum across interpreter and VM. Reference counting (Rc<T> for single-threaded, Arc<T> for potential threading). No garbage collector in v0.1. Strings immutable, arrays mutable and shared by reference. Function arguments passed by value with shared references for heap types.',
 'Code reuse: Both engines use identical value representation, reducing duplication ensuring consistent behavior. Memory safety: Rust Rc<T> provides automatic memory management without GC complexity. Performance: Reference counting has predictable performance (no GC pauses).',
 'accepted', '2024-01-18', datetime('now'), datetime('now')),

('runtime-DR-002', 'runtime', 'Raw Pointer Threading for SecurityContext',
 'SecurityContext threaded through runtime via raw pointer. Interpreter/VM store current_security: Option<*const SecurityContext>. Set during eval()/run() calls. Stdlib functions accept &SecurityContext parameter. Access via unsafe dereference in builtin calls.',
 'Simplicity: Avoids lifetime complexity while maintaining security checks. Safety: SecurityContext lifetime guaranteed valid for duration of eval()/run() execution - pointer always valid when dereferenced. Performance: Zero overhead compared to reference passing.',
 'accepted', '2024-02-05', datetime('now'), datetime('now'));

-- Stdlib decisions (6)
INSERT INTO decisions (id, component, title, decision, rationale, status, date, created_at, updated_at)
VALUES
('stdlib-DR-001', 'stdlib', 'JsonValue - Controlled Dynamic Typing Exception',
 'JsonValue is the only exception to no dynamic types principle. Follows Rust serde_json pattern (ergonomic + type-safe). Natural indexing: data["user"]["name"]. Explicit extraction: .as_string(), .as_number(). Returns JsonValue::Null for missing keys/indices. Isolated from regular type system - cannot be assigned to non-JsonValue variables without extraction.',
 'AI-first necessity: JSON critical for AI agent workflows. Delaying harms adoption. Controlled exception: Isolation via type system prevents dynamic typing from leaking. Industry precedent: Rust serde_json::Value proves this pattern works at scale.',
 'accepted', '2024-10-15', datetime('now'), datetime('now')),

('stdlib-DR-002', 'stdlib', 'Array API - Intrinsics vs Stdlib Split',
 'Split array functions by callback requirements. Pure functions (10) implemented in stdlib/array.rs: pop, shift, unshift, reverse, concat, flatten, indexOf, lastIndexOf, includes, slice. Callback intrinsics (11) implemented in interpreter/VM directly: map, filter, reduce, forEach, find, findIndex, flatMap, some, every, sort, sortBy.',
 'Callback functions need runtime execution context: To invoke user code, callbacks require access to interpreter/VM internals. Clean stdlib interface: Each engine uses native calling mechanism without complex abstraction layer. Industry precedent: V8, CPython, Rust all implement callbacks as runtime intrinsics.',
 'accepted', '2025-02-05', datetime('now'), datetime('now')),

('stdlib-DR-003', 'stdlib', 'Hash Function Design for Collections',
 'Implement deterministic hash function using Rust std::hash::Hash trait with DefaultHasher. Hashable types: Number (IEEE 754 bits), String (UTF-8 bytes), Bool (0/1), Null (fixed constant). Non-hashable: Array, Function, JsonValue, Option/Result (runtime error AT0140). Separate chaining collision strategy, automatic resizing at 0.75 load factor, initial capacity 16 buckets.',
 'Deterministic hashing critical for reproducible behavior AI agents expect. Type-safe: Only hashable types allowed at compile time. Performance: Rust DefaultHasher proven fast and collision-resistant. Simple: No custom hash functions needed for v0.2.',
 'accepted', '2026-02-15', datetime('now'), datetime('now')),

('stdlib-DR-004', 'stdlib', 'Collection Value Representation',
 'Add four new Value variants using Rc<RefCell<>> for shared mutable ownership: HashMap(Rc<RefCell<AtlasHashMap>>), HashSet(Rc<RefCell<AtlasHashSet>>), Queue(Rc<RefCell<AtlasQueue>>), Stack(Rc<RefCell<AtlasStack>>).',
 'Consistent with existing Array pattern: Arrays use Rc<RefCell<Vec<Value>>> for shared mutable access. Interior mutability: Required for collection methods that modify contents. Reference semantics: Collections behave like objects - assignments share reference, not deep copy.',
 'accepted', '2026-02-15', datetime('now'), datetime('now')),

('stdlib-DR-005', 'stdlib', 'Collection API Design and Iteration',
 'Use function-based API (not methods) with explicit collection types. HashMap API (15 functions), HashSet API (12 functions), Queue API (7 functions), Stack API (6 functions). All return void for mutations, Option<T> for removals, explicit types for clarity.',
 'Consistency with stdlib philosophy: All stdlib uses function-based API (not methods). Explicit over implicit: Clear type signatures, no method resolution ambiguity. AI-friendly: Function calls explicit, no hidden receiver type resolution. Iteration via functional style: keys/values/entries return arrays, use existing array iteration.',
 'accepted', '2026-02-15', datetime('now'), datetime('now')),

('stdlib-DR-006', 'stdlib', 'HashMap Stdlib Architecture Adaptation',
 'Adapt HashMap implementation to existing stdlib/mod.rs architecture. Function registration via is_builtin() and call_builtin() pattern. Direct implementation in stdlib/mod.rs without separate prelude.rs file.',
 'Architecture alignment: Existing stdlib uses single mod.rs with is_builtin() pattern - no prelude.rs file exists. Consistency: All stdlib functions follow same registration pattern. Simplicity: Single registration point reduces code duplication and maintenance burden.',
 'accepted', '2026-02-15', datetime('now'), datetime('now'));

-- Typechecker decisions (2)
INSERT INTO decisions (id, component, title, decision, rationale, status, date, created_at, updated_at)
VALUES
('typechecker-DR-001', 'typechecker', 'Monomorphization for Generic Types',
 'Monomorphization (Rust-style): Generate specialized code for each type instantiation. Monomorphizer caches specialized instances. Name mangling for VM dispatch: identity<number> becomes identity$number. Type inference determines concrete types at compile time. Both interpreter and VM use same monomorphization infrastructure.',
 'Performance: Zero runtime overhead - specialized code for each type. Type safety: Full type information available at compile time. Proven approach: Rust and C++ use monomorphization successfully. Debuggability: Each specialization is standalone code - easier to debug than type-erased versions.',
 'accepted', '2025-01-10', datetime('now'), datetime('now')),

('typechecker-DR-002', 'typechecker', 'TypeChecker-Owned Usage Tracking',
 'TypeChecker maintains internal declared_symbols and used_symbols tracking per function. No used field on Symbol struct. Symbol table remains immutable during type checking. Warnings emitted at end of each function scope.',
 'Architectural clarity: Binder creates/destroys scopes during binding phase, before type checking runs. Separation of concerns: Symbol = type information (immutable), TypeChecker = usage analysis (mutable tracking). AI-friendly: Clear separation, no unused fields to cause confusion.',
 'accepted', '2024-09-10', datetime('now'), datetime('now'));

-- VM decision (1)
INSERT INTO decisions (id, component, title, decision, rationale, status, date, created_at, updated_at)
VALUES
('vm-DR-001', 'vm', '.atb Bytecode Format with Debug Info',
 '.atb format defined in docs/bytecode-format.md: Binary format for compiled Atlas code. Debug info emitted by default (source maps, line numbers). Versioned format for future evolution. Serializable/deserializable for compilation caching.',
 'Binary format: Faster loading than text-based formats, smaller file size. Debug info by default: Development-friendly - errors show source locations. Production builds can strip if needed. Versioning: Future-proof for bytecode evolution.',
 'accepted', '2024-03-15', datetime('now'), datetime('now'));

-- ============================================================
-- FEATURES (7 total)
-- ============================================================

INSERT INTO features (name, display_name, version, status, description, created_at, updated_at)
VALUES
('build-system-core', 'Build System - Core Infrastructure', 'v0.2', 'Implemented',
 'Professional-grade build infrastructure for compiling Atlas projects. Core orchestration, multiple build targets, dependency ordering with topological sort. Phases 11a-11c.',
 datetime('now'), datetime('now')),

('build-system-incremental', 'Build System - Incremental Compilation', 'v0.2', 'Implemented',
 'Incremental compilation, intelligent build caching, profile-aware builds, build scripts, and CLI integration. Fast rebuilds recompile only changed modules and their dependents. Phases 11b-11c.',
 datetime('now'), datetime('now')),

('error-handling', 'Error Handling with Result<T,E>', 'v0.2', 'Implemented',
 'Rust-style explicit error handling with Result<T, E> types and ? operator for error propagation. Eliminates runtime exceptions. Errors are values, not exceptions. Function signatures show what can fail.',
 datetime('now'), datetime('now')),

('ffi-guide', 'Foreign Function Interface (FFI)', 'v0.2', 'Implemented',
 'Complete FFI system for interoperability with C libraries. Enables calling C functions, type marshaling, callbacks, and integration. Phase 10c complete.',
 datetime('now'), datetime('now')),

('first-class-functions', 'First-Class Functions', 'v0.2', 'Implemented',
 'Functions as first-class values. Can be assigned to variables, passed as arguments, returned from functions. Closures capture environment. Implemented in v0.2.',
 datetime('now'), datetime('now')),

('generics', 'Generic Types (Option<T>, Result<T,E>)', 'v0.2', 'Implemented',
 'Generic type system with parametric polymorphism. Option<T> for nullable values, Result<T,E> for error handling. Monomorphization for zero-cost abstraction. v0.2 BLOCKER 02-03.',
 datetime('now'), datetime('now')),

('module-system', 'Module System', 'v0.2', 'Implemented',
 'Module system for code organization. Import/export syntax, module resolution, dependency management. Supports relative and absolute imports. v0.2 BLOCKER 04-06.',
 datetime('now'), datetime('now'));

-- ============================================================
-- MIGRATION COMPLETE
-- ============================================================
-- 16 decisions migrated
-- 7 features migrated
