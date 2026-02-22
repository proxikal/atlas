# Phase 18 — Spec Update + Block 3 AC Check

**Block:** 3 (Trait System)
**Depends on:** Phase 17 complete
**Estimated tests added:** 0 (doc-only)

---

## Objective

Update the Atlas language specification to document the trait system. Verify all Block 3
acceptance criteria are met. Update STATUS.md and V03_PLAN.md.

---

## Spec Files to Update

### `docs/specification/types.md`

Replace the "Current Limitations" note under Generic Types:
```
- No user-defined generic types (only built-in: Option, Result, Array)
- No type parameter constraints/bounds
```

With:
```
- No user-defined generic struct types (structs are v0.4)
- Type parameter bounds supported via `:` syntax (`T: Copy`)
```

Add a new **Trait System** section between "Generic Types" and "Pattern Matching":

```markdown
## Trait System

Traits define a set of method signatures that types can implement. A type that implements
a trait can be used wherever that trait is required.

### Declaring a Trait

```atlas
trait Display {
    fn display(self: Display) -> string;
}

trait Shape {
    fn area(self: Shape) -> number;
    fn perimeter(self: Shape) -> number;
}
```

Trait bodies contain **method signatures only** — no implementations. Each method
signature ends with `;` instead of a block body.

### Implementing a Trait

```atlas
impl Display for number {
    fn display(self: number) -> string {
        return str(self);
    }
}
```

All methods declared in the trait must be implemented. Method signatures must match
exactly (parameter types and return type). Extra methods are allowed.

### Calling Trait Methods

```atlas
let x: number = 42;
let s: string = x.display();  // calls the Display impl for number
```

Method dispatch is **static** — the implementation is resolved at compile time based
on the receiver's type.

### Built-in Traits

| Trait | Purpose | Methods |
|-------|---------|---------|
| `Copy` | Value semantics — types that can be freely copied | (marker, no methods) |
| `Move` | Resource types requiring explicit ownership transfer | (marker, no methods) |
| `Drop` | Custom destructor logic | `fn drop(self: T) -> void` |
| `Display` | Human-readable string conversion | `fn display(self: T) -> string` |
| `Debug` | Debug string representation | `fn debug_repr(self: T) -> string` |

All primitive types (`number`, `string`, `bool`, `null`) implement `Copy`.

### Trait Bounds on Generic Type Parameters

```atlas
fn safe_copy<T: Copy>(x: T) -> T {
    return x;
}

fn display_and_return<T: Display>(x: T) -> string {
    return x.display();
}

// Multiple bounds
fn show_copy<T: Copy + Display>(x: T) -> string {
    return x.display();
}
```

### Error Codes

| Code | Meaning |
|------|---------|
| AT3001 | Trait redefines a built-in trait |
| AT3002 | Trait already defined |
| AT3003 | Trait not found |
| AT3004 | Impl is missing a required method |
| AT3005 | Impl method has wrong signature |
| AT3006 | Type does not implement the required trait |
| AT3007 | Copy type required |
| AT3008 | Trait bound not satisfied |
| AT3009 | Impl already exists for (type, trait) |
| AT3010 | (Warning) Move type passed without ownership annotation |

### Current Limitations (v0.3)

- Static dispatch only (no trait objects / vtable dispatch — v0.4)
- No `impl Trait` in return position syntax (`-> impl Display` — v0.4)
- `Drop` is not automatically called at scope exit — explicit only (v0.4)
- User-defined generic types require structs (v0.4)
```

### `docs/specification/syntax.md`

Add to the Keywords section:
```
`trait`, `impl`
```

Add a new **Trait Declaration Syntax** section:

```markdown
## Trait Declarations

### Trait Declaration

```
trait_decl := "trait" IDENT type_params? "{" trait_method_sig* "}"
trait_method_sig := "fn" IDENT type_params? "(" params ")" "->" type_ref ";"
```

Note the `;` terminator — trait method signatures have no body.

### Impl Blocks

```
impl_block := "impl" IDENT type_args? "for" IDENT "{" impl_method* "}"
impl_method := "fn" IDENT type_params? "(" params ")" "->" type_ref block
```

### Type Parameter Bounds

```
type_params := "<" type_param ("," type_param)* ">"
type_param := IDENT (":" trait_bound ("+" trait_bound)*)?
trait_bound := IDENT
```
```

---

## Block 3 Acceptance Criteria Check

From `V03_PLAN.md`:

- [ ] **`trait` and `impl` declarations parse and typecheck**
  → Phases 01–08 complete: parser handles both, typechecker validates conformance

- [ ] **Built-in `Copy`, `Move`, `Drop` traits work**
  → Phase 06: registered in TraitRegistry. Phase 09: Copy/Move ownership integration.
  Drop: declarable and explicitly callable.

- [ ] **Trait bounds on generics compile and enforce correctly**
  → Phase 05: parser. Phase 10: typechecker enforcement. Phase 12: compiler static dispatch.

- [ ] **Ownership traits integrate with Block 2 annotations**
  → Phase 09: `is_copy_type()` + ownership interaction logic. AT3010 warning for Move types.

- [ ] **Both engines dispatch trait methods identically**
  → Phase 14: 20 basic parity tests. Phase 17: 20 extended parity tests. Zero divergences.

---

## V03_PLAN.md Update

Update Block 3 section with "Planned vs. Actual" discoveries:

```markdown
### Planned vs. Actual

- **Phases:** Estimated 20–25, delivered exactly 18.
- **Mangled names:** Impl methods compile to `__impl__Type__Trait__Method` named functions
  (static dispatch via existing `Call` opcode — no new opcodes needed).
- **Drop:** Defined as a trait; explicit invocation only in Block 3.
  Automatic scope-exit drop deferred to v0.4 (requires scope tracking in both engines).
- **Display integration:** `Display` trait and `str()` stdlib are independent.
  `str()` does not dispatch through `Display` in Block 3 — types must call `.display()`
  explicitly. Automatic `str()` integration via Display is v0.4.
- **`Type::Named` investigation:** [fill in result during execution]
- **`Colon` token:** [fill in whether it existed or was added]
```

---

## STATUS.md Update

Update Block 3 row:
```
| 3 | Trait System (`trait`, `impl`, Copy/Move/Drop) | 18 | ✅ Complete (YYYY-MM-DD) |
```

Update Current State:
```
**Status:** Block 3 COMPLETE — ready for Block 4/5/6 scaffolding (all unblock)
**Last Completed:** Block 3 Phase 18 — Spec update + AC check (X,XXX tests passing)
**Next:** Scaffold Block 4 (Closures), Block 5 (Type Inference), or Block 6 (Error Handling)
```

---

## Auto-Memory Updates

After completing this phase, update:

1. `decisions/runtime.md` — Add DR-B03-01 (mangled name format for impl methods)
2. `testing-patterns.md` — Add trait test file locations
3. `patterns.md` — Add trait dispatch pattern (static dispatch, mangled names)
4. `decisions/typechecker.md` — Add DR-B03-02 (trait registry architecture)

---

## Acceptance Criteria

- [ ] `types.md` has complete Trait System section
- [ ] `syntax.md` has trait/impl grammar and `trait`/`impl` in keywords list
- [ ] All 5 Block 3 ACs verified and checked
- [ ] V03_PLAN.md "Planned vs. Actual" section filled in
- [ ] STATUS.md Block 3 row updated to ✅ Complete
- [ ] Auto-memory files updated with block discoveries
- [ ] Final `cargo test` run: all tests pass, 0 failures
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- This phase is docs-only: CI skips Rust jobs for docs-only PRs, merges fast.
- The spec should reflect what was ACTUALLY implemented, not the plan.
  Fill in "Planned vs. Actual" accurately during execution.
- Test count target: Block 3 should add 130–200 tests (18 phases × ~8–12 tests each).
  Total at block completion: ~9,400–9,450 tests.
