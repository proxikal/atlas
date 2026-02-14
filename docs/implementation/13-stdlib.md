# Standard Library Implementation

**Version:** v0.2 patterns
**Location:** `crates/atlas-runtime/src/stdlib/`
**See also:** `docs/api/stdlib.md` for API reference

---

## Organization

Atlas stdlib is organized into modules:

```
crates/atlas-runtime/src/stdlib/
├── mod.rs          # Module exports and organization
├── prelude.rs      # Prelude functions + constant registration
├── string.rs       # String API (Phase 01)
├── array.rs        # Array API (Phase 02)
├── math.rs         # Math API (Phase 03)
├── json.rs         # JSON API (Phase 04)
└── ...             # Future modules
```

---

## Intrinsic System (v0.2)

All stdlib functions are implemented as **intrinsics** - direct Rust function calls identified by name.

### Intrinsic Definition

```rust
// In stdlib/prelude.rs or module file
pub fn builtin_print(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument {
            function: "print".to_string(),
            expected: "1 argument".to_string(),
            got: args.len(),
            span: Span::default(),
        });
    }

    let output = match &args[0] {
        Value::String(s) => (**s).clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "string|number|bool|null".to_string(),
                found: args[0].type_name().to_string(),
                span: Span::default(),
            })
        }
    };

    println!("{}", output);
    Ok(Value::Void)
}
```

### Intrinsic Registration

Intrinsics are registered in the interpreter and VM:

```rust
// In interpreter/mod.rs or vm/mod.rs
fn call_intrinsic(&self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match name {
        // Prelude
        "print" => stdlib::prelude::builtin_print(args),
        "len" => stdlib::prelude::builtin_len(args),
        "str" => stdlib::prelude::builtin_str(args),

        // String functions
        "split" => stdlib::string::split(args),
        "join" => stdlib::string::join(args),
        "trim" => stdlib::string::trim(args),
        // ... 18 total string functions

        // Array functions
        "pop" => stdlib::array::pop(args),
        "map" => stdlib::array::map(args, self),
        "filter" => stdlib::array::filter(args, self),
        // ... 21 total array functions

        // Math functions
        "abs" => stdlib::math::abs(args),
        "sqrt" => stdlib::math::sqrt(args),
        "sin" => stdlib::math::sin(args),
        // ... 18 total math functions

        _ => Err(RuntimeError::UnknownFunction {
            name: name.to_string(),
            span: Span::default(),
        }),
    }
}
```

### Constant Registration

Constants are registered in the prelude during initialization:

```rust
// In stdlib/prelude.rs
pub fn register_constants(symbol_table: &mut SymbolTable) {
    use std::f64::consts;

    // Math constants
    symbol_table.define_constant("PI", Value::Number(consts::PI));
    symbol_table.define_constant("E", Value::Number(consts::E));
    symbol_table.define_constant("SQRT2", Value::Number(consts::SQRT_2));
    symbol_table.define_constant("LN2", Value::Number(consts::LN_2));
    symbol_table.define_constant("LN10", Value::Number(consts::LN_10));
}
```

---

## Type Signatures

Functions are registered in the symbol table with full type signatures:

```rust
// In symbol_table initialization
pub fn register_prelude_functions(symbol_table: &mut SymbolTable) {
    // print(value: string|number|bool|null) -> void
    symbol_table.register_builtin(
        "print",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Union(vec![
                Type::String,
                Type::Number,
                Type::Bool,
                Type::Null,
            ])],
            return_type: Box::new(Type::Void),
        },
    );

    // len(value: string|T[]) -> number
    symbol_table.register_builtin(
        "len",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Union(vec![
                Type::String,
                Type::Array(Box::new(Type::Generic {
                    name: "T".to_string(),
                    type_args: vec![],
                })),
            ])],
            return_type: Box::new(Type::Number),
        },
    );

    // Generic function: map<T, U>(arr: T[], fn: (T) -> U) -> U[]
    symbol_table.register_builtin(
        "map",
        Type::Function {
            type_params: vec!["T".to_string(), "U".to_string()],
            params: vec![
                Type::Array(Box::new(Type::TypeParameter("T".to_string()))),
                Type::Function {
                    type_params: vec![],
                    params: vec![Type::TypeParameter("T".to_string())],
                    return_type: Box::new(Type::TypeParameter("U".to_string())),
                },
            ],
            return_type: Box::new(Type::Array(Box::new(Type::TypeParameter("U".to_string())))),
        },
    );
}
```

---

## Callbacks (v0.2)

Array functions (`map`, `filter`, `reduce`, etc.) accept **named function references only**.

### Callback Invocation

```rust
// In stdlib/array.rs
pub fn map(args: Vec<Value>, interpreter: &Interpreter) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(/* arity error */);
    }

    let array = match &args[0] {
        Value::Array(arr) => arr,
        _ => return Err(/* type error */),
    };

    let callback = &args[1];  // Function value

    let mut result = Vec::new();
    for element in array.borrow().iter() {
        // Call the function for each element
        let output = interpreter.call_function(callback, vec![element.clone()])?;
        result.push(output);
    }

    Ok(Value::Array(Rc::new(RefCell::new(result))))
}
```

**Note:** v0.2 doesn't support closures or anonymous functions - callbacks must be named functions.

---

## Error Handling

Stdlib functions use `RuntimeError` for all errors:

```rust
pub enum RuntimeError {
    InvalidStdlibArgument {
        function: String,
        expected: String,
        got: usize,
        span: Span,
    },
    TypeError {
        expected: String,
        found: String,
        span: Span,
    },
    ValueError {
        message: String,
        span: Span,
    },
    // ... other variants
}
```

**Conventions:**
- Arity errors: `InvalidStdlibArgument`
- Type errors: `TypeError`
- Domain errors (sqrt(-1), etc.): Return `NaN` or use `ValueError`
- Permission errors: `FilesystemPermissionDenied`, etc.

---

## Interpreter/VM Parity

**Critical:** Every stdlib function must work identically in both engines.

### Testing Pattern

```rust
#[rstest]
fn test_string_split_basic() {
    let source = r#"
        let result = split("a,b,c", ",");
        result
    "#;

    // Test interpreter
    let interp_result = eval_interpreter(source);
    assert!(matches!(interp_result, Value::Array(_)));

    // Test VM
    let vm_result = eval_vm(source);
    assert!(matches!(vm_result, Value::Array(_)));

    // Results must be identical
    assert_eq!(interp_result, vm_result);
}
```

---

## Implementation Checklist

When adding a new stdlib function:

- [ ] Define intrinsic in appropriate module (`string.rs`, `array.rs`, etc.)
- [ ] Register in `call_intrinsic()` for both interpreter and VM
- [ ] Add type signature to symbol table registration
- [ ] Implement comprehensive error handling
- [ ] Add interpreter tests using `rstest`
- [ ] Add VM tests using `rstest`
- [ ] Verify parity between interpreter and VM
- [ ] Update `docs/api/stdlib.md` with API documentation
- [ ] Add insta snapshots for diagnostics if applicable
- [ ] Verify no clippy warnings

---

## Best Practices

1. **Use Result<Value, RuntimeError>** - Never panic in stdlib code
2. **Validate arity first** - Check argument count before accessing
3. **Validate types second** - Check types with clear error messages
4. **Follow IEEE 754 for math** - Use Rust's `std::f64` methods
5. **Preserve immutability** - Arrays are always cloned, never mutated
6. **Test edge cases** - NaN, infinity, empty arrays, Unicode, etc.
7. **Document in API reference** - Keep `docs/api/stdlib.md` up to date

---

## See Also

- **API Reference:** `docs/api/stdlib.md`
- **Phase Files:**
  - Phase 01: `phases/stdlib/phase-01-complete-string-api.md`
  - Phase 02: `phases/stdlib/phase-02-complete-array-api.md`
  - Phase 03: `phases/stdlib/phase-03-complete-math-api.md`
- **Testing Guide:** `docs/guides/testing-guide.md`
- **Value System:** `docs/implementation/02-core-types.md`
