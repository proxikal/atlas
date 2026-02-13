# BLOCKER 02-A: Generic Type System Foundation

**Part:** 1 of 4 (Type System Foundation)
**Category:** Foundation - Type System Extension
**Estimated Effort:** 1 week
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Type system and type checker must be stable and well-tested.

**Verification:**
```bash
grep -n "enum Type" crates/atlas-runtime/src/types.rs
cargo test typechecker --no-fail-fast
cargo test binder --no-fail-fast
grep -c "test.*type" crates/atlas-runtime/tests/*.rs
```

**What's needed:**
- Stable Type enum with basic types
- Type checker with inference
- Binder with type resolution
- 200+ type system tests passing
- No existing type system bugs

**If missing:** Fix type system issues first.

---

## Objective

**THIS PHASE:** Add generic type syntax and AST representation. Enables parsing `Type<T>` syntax and represents it in the AST. Foundation for type checking and runtime implementation in later sub-phases.

**NOT in this phase:** Type checking, inference, monomorphization, runtime. Just syntax and AST.

**Success criteria:** Can parse `Result<T, E>`, `Option<T>`, `HashMap<K, V>` and represent in AST.

---

## Implementation

### Step 1: Define Syntax (Day 1)

**Generic type syntax:**
```atlas
// Type applications
let x: Result<number, string>;
let y: Option<bool>;
let z: HashMap<string, number>;

// Function with generic params
fn identity<T>(x: T) -> T {
    return x;
}

// Nested generics
let nested: Option<Result<T, E>>;
```

**Design decisions:**
- Use `<>` for type parameters (like Rust, TypeScript, Java)
- Support multiple parameters: `Type<T1, T2, T3>`
- Support nested: `Type<Other<T>>`
- Require explicit parameters initially (inference in 02-B)

### Step 2: Extend AST (Day 2)

**File:** `crates/atlas-runtime/src/ast.rs`

Add `TypeRef::Generic` variant:
```rust
pub enum TypeRef {
    Named(String, Span),
    Array(Box<TypeRef>, Span),
    Function {
        params: Vec<TypeRef>,
        return_type: Box<TypeRef>,
        span: Span,
    },
    // NEW: Generic type application
    Generic {
        name: String,              // "Result", "Option", "HashMap"
        type_args: Vec<TypeRef>,   // Type arguments
        span: Span,
    },
}
```

**Update TypeRef::span():**
```rust
impl TypeRef {
    pub fn span(&self) -> Span {
        match self {
            // ... existing variants
            TypeRef::Generic { span, .. } => *span,
        }
    }
}
```

### Step 3: Extend Type Representation (Day 2)

**File:** `crates/atlas-runtime/src/types.rs`

Add Type variants for generics:
```rust
pub enum Type {
    // ... existing variants

    // Generic type with instantiated arguments
    Generic {
        name: String,              // "Result", "Option", "HashMap"
        type_args: Vec<Type>,      // Resolved type arguments
    },

    // Type parameter (unresolved variable)
    TypeParameter {
        name: String,              // "T", "E", "K", "V"
        // Constraints empty for now (02-B will add)
    },
}
```

**Update Type::display_name():**
```rust
impl Type {
    pub fn display_name(&self) -> String {
        match self {
            // ... existing variants
            Type::Generic { name, type_args } => {
                let args = type_args.iter()
                    .map(|t| t.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args)
            }
            Type::TypeParameter { name } => name.clone(),
        }
    }
}
```

### Step 4: Parser Implementation (Days 3-4)

**File:** `crates/atlas-runtime/src/parser/expr.rs`

Add `parse_generic_type()` method:
```rust
// Parse generic type: Type<T1, T2>
fn parse_generic_type(&mut self, name: String, start: Span) -> Result<TypeRef, ParseError> {
    // Expect '<'
    self.expect(TokenKind::LessThan)?;

    // Parse type arguments
    let mut type_args = vec![];
    loop {
        type_args.push(self.parse_type_ref()?);

        if !self.match_token(TokenKind::Comma) {
            break;
        }
    }

    // Expect '>'
    let end = self.expect(TokenKind::GreaterThan)?;

    Ok(TypeRef::Generic {
        name,
        type_args,
        span: start.merge(end),
    })
}
```

**Update `parse_type_ref()`:**
```rust
fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
    match self.current.kind {
        TokenKind::Identifier => {
            let name = self.current.value.clone();
            let start = self.current.span;
            self.advance();

            // Check for generic type
            if self.check(TokenKind::LessThan) {
                // Could be generic OR comparison - need lookahead
                // For type position, assume generic
                return self.parse_generic_type(name, start);
            }

            Ok(TypeRef::Named(name, start))
        }
        // ... rest of existing cases
    }
}
```

**Handle nested generics:**
```rust
// This should work automatically via recursion
// Option<Result<T, E>> parses as:
// Generic { name: "Option", type_args: [
//   Generic { name: "Result", type_args: [TypeParameter("T"), TypeParameter("E")] }
// ]}
```

### Step 5: Error Handling (Day 5)

**Add parser errors:**
```rust
// Missing closing >
"expected '>' after type arguments"

// Empty type args
"generic type requires at least one type argument"

// Malformed syntax
"invalid type argument"
```

### Step 6: Testing (Days 6-7)

**File:** `crates/atlas-runtime/tests/generic_syntax_tests.rs`

Create 40+ tests covering:
```rust
// Basic syntax
"Result<T, E>"
"Option<T>"
"HashMap<K, V>"

// Nested
"Option<Result<T, E>>"
"HashMap<string, Option<number>>"

// With arrays
"Option<number[]>"
"Result<string[], Error>"

// In function signatures
"fn foo<T>(x: T) -> Result<T, E>"
"fn bar() -> Option<string>"

// Error cases
"Result<>"           // Empty
"Option<T"          // Missing >
"HashMap<K, V, X"   // Unterminated

// Complex nesting
"HashMap<string, Result<Option<T>, E>>"
```

**Add to parser tests:**
```rust
#[rstest]
#[case("Result<number, string>")]
#[case("Option<bool>")]
#[case("HashMap<string, number>")]
fn test_parse_generic_types(#[case] input: &str) {
    let mut parser = Parser::new(input);
    let type_ref = parser.parse_type_ref().unwrap();

    assert!(matches!(type_ref, TypeRef::Generic { .. }));
}
```

---

## Files

### Create
- `crates/atlas-runtime/tests/generic_syntax_tests.rs` (~300 lines)

### Modify
- `crates/atlas-runtime/src/ast.rs` (~30 lines) - Add TypeRef::Generic
- `crates/atlas-runtime/src/types.rs` (~40 lines) - Add Type::Generic, Type::TypeParameter
- `crates/atlas-runtime/src/parser/expr.rs` (~80 lines) - Parse generic syntax

**Total changes:** ~450 lines

---

## Acceptance Criteria

**Functionality:**
- ✅ Parse `Type<T>` syntax
- ✅ Parse `Type<T1, T2, T3>` (multiple params)
- ✅ Parse nested `Type<Other<T>>`
- ✅ TypeRef::Generic in AST
- ✅ Type::Generic and Type::TypeParameter in type system
- ✅ Error on malformed syntax

**Quality:**
- ✅ 40+ parser tests pass
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ Error messages clear

**Documentation:**
- ✅ Code comments explain design
- ✅ Examples in tests

**Next phase:** blocker-02-b-type-checker-inference.md

---

## Dependencies

**Requires:**
- Stable type system from v0.1 ✅

**Blocks:**
- BLOCKER 02-B (Type Checker & Inference)
- BLOCKER 02-C (Runtime Implementation)
- BLOCKER 02-D (Built-in Types)

**This phase MUST be complete before 02-B.**

---

## Verification Commands

**After implementation:**
```bash
# All tests pass
cargo test generic_syntax_tests

# No warnings
cargo clippy -- -D warnings

# Formatted
cargo fmt -- --check

# Can parse generics
echo 'let x: Result<number, string>;' | cargo run -- check -
```

---

## Notes

**Keep it focused:** Just syntax and AST. Don't implement type checking or runtime yet.

**Test thoroughly:** Parser is foundation. Errors here propagate everywhere.

**Clear errors:** Parser errors should guide fixes clearly.

**This is phase 1 of 4 for generic types. Stay focused on syntax only.**
