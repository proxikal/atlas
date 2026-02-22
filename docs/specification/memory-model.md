# Atlas Memory Model Specification

**Status:** LOCKED — v0.3 implementation target
**Decision date:** 2026-02-21
**Supersedes:** Implicit Arc<Mutex<Value>> bootstrap model (v0.1–v0.2)

---

## Decision Summary

Atlas adopts **graduated value semantics with explicit ownership annotations** as its permanent
memory model. This is a first-class language decision, not a v0.x experiment.

**What this means:**
- Collections and objects are **copy-on-write value types** by default
- Shared mutable state requires **explicit `shared<T>` opt-in**
- Function parameters carry **explicit ownership annotations** (`own`, `borrow`, `shared`)
- **No garbage collector.** Ever. Deterministic allocation is a hard requirement.
- **No implicit borrow checker.** Ownership is expressed in syntax, not inferred from context.

---

## Why This Model

### Why not GC?
Go chose GC early and permanently closed the door to systems programming. GC introduces
nondeterministic pauses, prevents predictable latency, and makes Atlas unsuitable for embedded,
real-time, and OS-level work. Atlas's end goal is systems-level. GC is incompatible with that goal.

### Why not Rust's borrow checker?
Rust's borrow checker is the single hardest thing for LLMs to generate correctly. Lifetimes
are implicit, the rules are non-local, and AI models fail on borrow checker errors at a higher
rate than any other error category. Atlas is AI-first. Implicit complexity is the enemy of
reliable AI code generation.

Rust's *safety* is not the problem — it's the *implicitness*. Atlas achieves the same safety
guarantees by making ownership **explicit in syntax**. If it's in the signature, AI can generate
it. If it's inferred by context, AI will get it wrong.

### Why value semantics?
Copy-on-write value semantics eliminate the entire class of aliasing bugs without requiring
any ownership reasoning from the programmer or the AI. Most code never needs shared mutation.
Default-immutable-sharing with explicit mutation is the model that produces the fewest surprises
for both human programmers and AI agents.

### The Swift Precedent
Swift started as a high-level ARC language and is systematically adding ownership features
(`consuming`, `borrowing`, noncopyable types — Swift 5.9/6). It is now targeting embedded
systems. Atlas takes this same trajectory but with explicit syntax from the start, avoiding
the "retrofitting" problem.

---

## The Three Ownership Modes

### `own` — Single ownership (move semantics)
```atlas
fn process(own data: Buffer) -> Result<string, string> {
    // `data` is consumed here. Caller cannot use it after this call.
}
```
- Value is moved into the function
- Caller's binding is invalidated after the call
- No copy, no reference count — zero cost
- Default for resource types (file handles, sockets, buffers)

### `borrow` — Immutable borrow (read-only reference)
```atlas
fn read(borrow data: Buffer) -> number {
    // `data` is readable but not modifiable. Caller retains ownership.
}
```
- No copy, no ownership transfer
- Multiple simultaneous borrows are safe
- Cannot be mutated
- Default for "inspect but don't change" operations

### `shared` — Explicit reference semantics (opt-in)
```atlas
fn transform(shared data: Buffer) -> Buffer {
    // `data` is a reference-counted shared value.
    // Use when multiple owners genuinely need the same mutable state.
}
```
- Explicit Arc<T> under the hood
- Ref-counted, deterministic drop when count reaches zero
- NOT the default — must be explicitly requested
- Replaces the implicit `Arc<Mutex<Value>>` that was everywhere in v0.1–v0.2

---

## Value Types (Copy-on-Write)

All collection and object types are **value types** with copy-on-write semantics:

```atlas
let a = [1, 2, 3]
let b = a           // b is a logical copy — no heap allocation yet
b.push(4)           // mutation triggers copy — a is still [1, 2, 3]
print(a)            // [1, 2, 3]
print(b)            // [1, 2, 3, 4]
```

**CoW guarantees:**
- Reading a value never allocates
- Mutation of an exclusively-owned value is in-place (no copy)
- Mutation of a shared value triggers copy before mutation
- AI can treat all values as independent — no aliasing to reason about

**Types that are value types:**
- `array<T>` — was `Arc<Mutex<Vec<Value>>>`
- `map<K, V>` — was `Arc<Mutex<HashMap<...>>>`
- `string` — already value type, no change
- `number`, `bool`, `null` — already value types, no change
- User-defined structs (v0.3+) — value type by default

**Types that are explicit reference types:**
- `shared<T>` — explicit opt-in
- File handles, socket connections, OS resources — always owned (own)

---

## Ownership in Practice

### Parameter annotations
```atlas
// Takes ownership — caller cannot use `buf` after this
fn send(own buf: Buffer) -> Result<null, string>

// Borrows — caller retains ownership
fn checksum(borrow data: array<number>) -> number

// Shared reference — both caller and callee share the value
fn register(shared handler: EventHandler) -> null
```

### Return ownership
```atlas
// Returns a new owned value
fn allocate(size: number) -> own Buffer

// Returns a borrow (lifetime tied to input)
fn first(borrow arr: array<number>) -> borrow number
```

### Inference rules
When no annotation is specified:
- Value types (number, bool, string, array, map): **copy** (implicit CoW)
- Resource types (Buffer, File, Socket): compiler error — must annotate
- `shared<T>`: must be explicit — never inferred

---

## Implementation Notes (v0.3 — Block 1 complete)

The v0.3 Block 1 migration replaced all implicit reference semantics with CoW value types.
These notes document the concrete Rust types behind each Atlas concept.

### CoW Wrapper Types

| Atlas type | Rust implementation | CoW mechanism |
|-----------|-------------------|---------------|
| `array<T>` | `ValueArray(Arc<Vec<Value>>)` | `Arc::make_mut` on mutation |
| `map<K,V>` | `ValueHashMap(Arc<AtlasHashMap>)` | `Arc::make_mut` on mutation |
| `set<T>` | `ValueHashSet(Arc<AtlasHashSet>)` | `Arc::make_mut` on mutation |
| `Queue<T>` | `ValueQueue(Arc<VecDeque<Value>>)` | `Arc::make_mut` on mutation |
| `Stack<T>` | `ValueStack(Arc<Vec<Value>>)` | `Arc::make_mut` on mutation |
| `shared<T>` | `Arc<Mutex<Value>>` | Explicit reference semantics |

### Mutation Write-Back Pattern

Stdlib mutation functions return the updated collection. Both engines (interpreter and VM)
detect when the first argument is a local variable and automatically write the returned
collection back to that variable. This preserves value semantics transparently:

```atlas
arr.push(x)    // interpreter/VM calls arrayPush(arr, x), writes result back to arr
arr.pop()      // calls arrayPop(arr), writes [result, new_arr] pair; arr shrinks
```

The write-back is triggered by `TypeTag::Array` / `TypeTag::HashMap` in method dispatch.
Free-function variants (e.g. `push(arr, x)`) also write back via the same mechanism.

### Equality Semantics

| Type | Equality |
|------|----------|
| `number`, `bool`, `null` | Value equality |
| `string` | Content equality |
| `array`, `map`, `set`, `queue`, `stack` | Structural (content) equality |
| `shared<T>` | Reference equality (same allocation = equal) |
| Functions, handles | Reference equality |

---

## Migration from Arc<Mutex<Value>> (v0.2 → v0.3)

The v0.2 implementation used `Arc<Mutex<Value>>` for all heap values. This is the bootstrap
model and is entirely replaced in v0.3.

**Migration strategy:**
1. Replace `Arc<Mutex<Vec<Value>>>` (arrays) with CoW `ValueArray` struct
2. Replace `Arc<Mutex<HashMap<...>>>` (maps) with CoW `ValueMap` struct
3. Introduce `Shared<T>` wrapper for explicit reference semantics
4. Update all 300+ stdlib functions to operate on value types
5. Update interpreter and VM to use value semantics throughout
6. Maintain parity between engines throughout migration

**Breaking changes:**
- Array/map mutation semantics change: mutations no longer affect aliased copies
- `Arc::ptr_eq` identity checks (like the hashset fix) become unnecessary
- Thread-safety model changes: shared mutation requires explicit `shared<T>`

---

## Compile-Time Verification (v0.4)

v0.3 implements ownership annotations with **runtime verification** (debug assertions).
v0.4 implements **compile-time verification** — the static analysis pass that proves ownership
annotations are correct without running the program.

This sequencing is intentional:
- v0.3: syntax and semantics locked in, runtime verified
- v0.4: compile-time proof layer added on top of stable v0.3 foundation

The compile-time verifier does NOT need to be a Rust-style borrow checker. Because ownership
is explicit in the syntax, the verifier is a dataflow pass over the typed AST — significantly
simpler than Rust's NLL (non-lexical lifetimes) analysis.

---

## Implementation Notes (v0.3 — Block 2 complete)

**Completed:** 2026-02-22

### Ownership Annotation Tokens
`own`, `borrow`, `shared` are reserved keywords (added in Block 2).
They are invalid as identifiers in Atlas v0.3+.

### AST Representation
- `Param.ownership: Option<OwnershipAnnotation>` — `None` = unannotated
- `FunctionDecl.return_ownership: Option<OwnershipAnnotation>`
- `OwnershipAnnotation` enum: `Own | Borrow | Shared`

### Runtime Enforcement (v0.3 — debug assertions)
- `own` param: caller binding marked consumed; reuse → runtime error (debug mode)
- `shared` param: argument must be `Value::SharedValue(_)` (debug mode assertion)
- `borrow` param: no runtime enforcement — value semantics + CoW provide the guarantee
- Both engines (interpreter + VM) enforce identically — verified by 22 parity tests

### Compile-Time Enforcement (v0.4)
v0.4 adds a static dataflow pass over the typed AST. No syntax changes required —
the annotation system is already complete. v0.4 only adds the verification layer.

### Diagnostic Codes
- `AT_OWN_ON_PRIMITIVE` (warning) — `own` annotation on primitive type
- `AT_BORROW_ON_SHARED` (warning) — `borrow` annotation on `shared<T>` type
- `AT_BORROW_TO_OWN` (warning) — passing borrowed value to `own` parameter
- `AT_NON_SHARED_TO_SHARED` (error) — non-`shared<T>` value to `shared` parameter

---

## AI Code Generation Guidelines

When generating Atlas code, AI agents should follow these rules:

1. **Use value types by default.** Never reach for `shared<T>` unless you have a specific
   reason that two parties need to mutate the same object.

2. **Annotate resource parameters explicitly.** If a function takes a file handle, buffer,
   or socket, always annotate with `own` (if consuming) or `borrow` (if reading).

3. **Prefer `borrow` over `shared`.** Most read-only operations should use `borrow`. Only
   use `shared` when multiple callers need to mutate.

4. **Value mutations are safe.** You can freely mutate array/map locals. The CoW model ensures
   callers are not affected.

5. **No lifetime annotations.** Atlas does not have `'a` lifetime syntax. Ownership is
   function-local and expressed through parameter annotations only.

---

## Relationship to Systems Language Goals

This memory model is the foundation that makes Atlas systems-capable:

| Systems requirement | Atlas mechanism |
|---|---|
| No GC / deterministic alloc | Value types + owned resources, no tracing GC |
| Zero-cost abstractions | CoW with move optimization, `own` = no copy |
| Safe concurrency | `shared<T>` requires explicit opt-in; CoW values are safe to share |
| Embeddable | No GC thread, no runtime overhead for simple programs |
| Predictable performance | No GC pauses, CoW only allocates on actual mutation |
| FFI safety | `own` resources have clear handoff semantics across FFI boundary |

---

*This specification is final. Changes require explicit architectural decision record.*
*See `memory/decisions/runtime.md` for the DR entry.*
