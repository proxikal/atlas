# atlas-runtime/src/

The core compiler + runtime. 95% of all Atlas work happens here.

## Directory Map

| Path | What it is |
|------|-----------|
| `value.rs` | **Value enum — all runtime types. Touch this = touch everything.** |
| `ast.rs` | AST nodes: `FunctionDecl`, `Param`, `TypeRef`, `Stmt`, `Expr` |
| `token.rs` | `TokenKind` enum + `is_keyword()` + `as_str()` |
| `lexer/mod.rs` | Tokenizer — keyword map, identifier promotion |
| `parser/mod.rs` | AST construction from token stream |
| `typechecker/` | Type resolution, inference, generics, call-site checks |
| `compiler/` | AST → bytecode (`mod.rs`, `expr.rs`, `stmt.rs`) |
| `interpreter/` | Tree-walking eval (`mod.rs`, `expr.rs`, `stmt.rs`) |
| `vm/mod.rs` | Bytecode execution engine (4100+ lines) |
| `bytecode/` | Opcode definitions, serialization |
| `stdlib/` | 25 modules, 300+ functions |
| `typechecker/mod.rs` | Function type resolution — `params` at line ~202 |
| `typechecker/expr.rs` | Call-site type checking |
| `diagnostic.rs` | Diagnostic registry — add new codes here |
| `binder.rs` | Name resolution pass |
| `resolver/` | Module resolution |
| `security/` | Permission model, sandbox |
| `ffi/` | Foreign function interface |
| `async_runtime/` | Tokio integration, AtlasFuture, channels |
| `debugger/` | Breakpoints, stepping, source mapping |
| `optimizer/` | Constant folding, dead code, peephole |

## Tests

**Location:** `crates/atlas-runtime/tests/`
**Rule: NO new test files.** Add to existing domain files:

| Domain | File |
|--------|------|
| Interpreter behavior | `tests/interpreter.rs` |
| VM behavior | `tests/vm.rs` |
| Collections/CoW | `tests/collections.rs` |
| Type system | `tests/typesystem.rs` |
| Pattern matching | `tests/pattern_matching.rs` |
| Stdlib | `tests/stdlib.rs` or `tests/stdlib/` |
| Closures | `tests/closures.rs` |
| Async | `tests/async_runtime.rs` |
| Frontend/parse | `tests/frontend_integration.rs` |
| Parity tests | Add to the relevant domain file with both engines |

## Critical Rules

**Parity is sacred.** Every behavior change must produce identical output in both
interpreter (`interpreter/mod.rs`) and VM (`vm/mod.rs`). If you touch one, you touch both.
Parity break = BLOCKING. Never ship a phase with parity divergence.

**CoW write-back pattern.** Collection mutation builtins return a NEW collection.
The interpreter (`apply_cow_writeback()`) and VM (`emit_cow_writeback_if_needed()`) write
the result back to the caller's variable. Both `let` and `var` bindings can be mutated
this way — it's content mutation, not rebinding. See DR-004 in auto-memory decisions/runtime.md.

**value.rs blast radius.** Adding a new `Value` variant requires updating:
`type_name()`, `Display`, `PartialEq`, equality semantics, bytecode serialization,
interpreter eval, VM execution, all stdlib functions that pattern-match on Value.

## Key Invariants (verified 2026-02-21)

- `ValueArray` = `Arc<Vec<Value>>` — CoW via `Arc::make_mut`
- `ValueHashMap` = `Arc<AtlasHashMap>` — CoW via `Arc::make_mut`
- `Shared<T>` = `Arc<Mutex<T>>` — explicit reference semantics only
- `FunctionRef` at `value.rs:464` — holds arity, bytecode_offset, local_count
- `Param` at `ast.rs:187` — name, type_ref, ownership, span (ownership added Block 2)
- Expression statements require semicolons — `f(x)` without `;` fails to parse
