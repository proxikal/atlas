# Runtime Value Model

Shared value representation for interpreter and VM.

## Value Definition

```rust
// value.rs
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Null,
    Array(Rc<RefCell<Vec<Value>>>),
    Function(FunctionRef),
    JsonValue(Rc<JsonValue>),  // Isolated dynamic type for JSON interop (v0.2+)
}

#[derive(Debug, Clone)]
pub struct FunctionRef {
    pub name: String,
    pub arity: usize,
    pub bytecode_offset: usize,  // For VM
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::Array(_) => "array",
            Value::Function(_) => "function",
            Value::JsonValue(_) => "json",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false,  // Only bool values in Atlas
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.as_ref().clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                let elements: Vec<String> = borrowed.iter()
                    .map(|v| v.to_string())
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Function(f) => format!("<fn {}>", f.name),
            Value::JsonValue(j) => j.to_string(),  // Delegates to JsonValue's Display
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b),  // Reference identity
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            (Value::JsonValue(a), Value::JsonValue(b)) => a == b,  // Structural equality
            _ => false,
        }
    }
}
```

## Runtime Errors

```rust
#[derive(Debug)]
pub enum RuntimeError {
    TypeError,
    DivideByZero,
    OutOfBounds,
    InvalidNumericResult,  // NaN, Infinity
    UnknownOpcode,
    StackUnderflow,
    UnknownFunction(String),
    InvalidStdlibArgument,
    InvalidIndex,  // Non-integer index
}
```

## Memory Model

- **Numbers/Bools/Null:** Immediate values (stack-allocated)
- **Strings:** Heap-allocated, reference-counted (`Rc<String>`), immutable
- **Arrays:** Heap-allocated, reference-counted (`Rc<RefCell<Vec<Value>>>`), mutable
- **Functions:** Reference to bytecode or builtin

## Mutability Rules

- Strings are **immutable**
- Arrays are **mutable** through interior mutability (`RefCell`)
- Assignment copies references, not values
- Array element assignment is visible to all references

## Example

```rust
let arr1 = Value::Array(Rc::new(RefCell::new(vec![
    Value::Number(1.0),
    Value::Number(2.0),
])));

let arr2 = arr1.clone();  // Same reference

// Mutate through arr1
if let Value::Array(a) = &arr1 {
    a.borrow_mut()[0] = Value::Number(42.0);
}

// arr2 sees the change
// arr2[0] is now 42.0
```
