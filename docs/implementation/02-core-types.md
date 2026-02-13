# Core Types

These types are used throughout the codebase. Define them early in the foundation phase.

## Span (Source Location Tracking)

```rust
// span.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,   // Byte offset in source
    pub end: usize,     // Byte offset (exclusive)
    pub line: u32,      // 1-based line number
    pub column: u32,    // 1-based column number
}

impl Span {
    pub fn new(start: usize, end: usize, line: u32, column: u32) -> Self {
        Self { start, end, line, column }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn combine(start: Span, end: Span) -> Span {
        Span {
            start: start.start,
            end: end.end,
            line: start.line,
            column: start.column,
        }
    }
}

// For nodes that don't have a span yet
pub const DUMMY_SPAN: Span = Span { start: 0, end: 0, line: 0, column: 0 };
```

## Symbol Representation

```rust
// symbol.rs
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub kind: SymbolKind,
    pub span: Span,  // Where it was declared
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Builtin,  // For prelude functions
}
```

## Type Representation

```rust
// types.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Null,
    Void,
    Array(Box<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    JsonValue,  // Isolated dynamic type for JSON interop (v0.2+)
    Unknown,  // For error recovery
}

impl Type {
    pub fn is_assignable_to(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Unknown, _) | (_, Type::Unknown) => true,  // Error recovery
            (Type::Null, Type::Null) => true,
            (Type::Array(a), Type::Array(b)) => a.is_assignable_to(b),
            (Type::Function { params: p1, return_type: r1 },
             Type::Function { params: p2, return_type: r2 }) => {
                p1.len() == p2.len() &&
                p1.iter().zip(p2.iter()).all(|(a, b)| a.is_assignable_to(b)) &&
                r1.is_assignable_to(r2)
            }
            (Type::JsonValue, Type::JsonValue) => true,  // JsonValue isolated - only json->json
            _ => self == other,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Number => "number".to_string(),
            Type::String => "string".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Null => "null".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(elem) => format!("{}[]", elem.to_string()),
            Type::Function { params, return_type } => {
                let params_str = params.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", params_str, return_type.to_string())
            }
            Type::JsonValue => "json".to_string(),
            Type::Unknown => "<unknown>".to_string(),
        }
    }
}
```

## Usage Notes

- **Span**: Attach to every AST node and token for error reporting
- **Symbol**: Used by symbol table during binding phase
- **Type**: Used by typechecker and for function signatures
- **Type::Unknown**: Use for error recovery - allows typechecking to continue after errors
