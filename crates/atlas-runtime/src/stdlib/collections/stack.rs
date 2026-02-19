//! Stack collection - LIFO (Last-In-First-Out)
//!
//! Atlas Stack implementation using Vec for O(1) push/pop operations.
//! Provides efficient LIFO stack semantics.

use crate::value::Value;

/// Atlas Stack - LIFO collection with O(1) push/pop
///
/// Backed by Vec for optimal performance.
/// Supports all standard stack operations: push, pop, peek.
#[derive(Debug, Clone)]
pub struct AtlasStack {
    inner: Vec<Value>,
}

impl AtlasStack {
    /// Create new empty stack
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let stack = AtlasStack::new();
    /// assert!(stack.is_empty());
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Create stack with pre-allocated capacity
    ///
    /// Useful for performance when stack size is known.
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let stack = AtlasStack::with_capacity(100);
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Push element onto top of stack (LIFO order)
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// stack.push(Value::String("hello".into()));
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn push(&mut self, value: Value) {
        self.inner.push(value);
    }

    /// Pop element from top of stack
    ///
    /// Returns `None` if stack is empty.
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// assert_eq!(stack.pop(), Some(Value::Number(1.0)));
    /// assert_eq!(stack.pop(), None);
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    /// View top element without removing
    ///
    /// Returns `None` if stack is empty.
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(42.0));
    /// assert_eq!(stack.peek(), Some(&Value::Number(42.0)));
    /// assert_eq!(stack.len(), 1); // Still has 1 element
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn peek(&self) -> Option<&Value> {
        self.inner.last()
    }

    /// Get number of elements in stack
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let mut stack = AtlasStack::new();
    /// assert_eq!(stack.len(), 0);
    /// stack.push(Value::Number(1.0));
    /// assert_eq!(stack.len(), 1);
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if stack is empty
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let stack = AtlasStack::new();
    /// assert!(stack.is_empty());
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements from stack
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// stack.clear();
    /// assert!(stack.is_empty());
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert stack to array (bottom to top order)
    ///
    /// Bottom of stack becomes first array element.
    ///
    /// # Example
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// stack.push(Value::Number(2.0));
    /// let arr = stack.to_vec();
    /// assert_eq!(arr, vec![Value::Number(1.0), Value::Number(2.0)]);
    /// ```rust
    /// # use atlas_runtime::stdlib::collections::stack::AtlasStack;
    /// # use atlas_runtime::value::Value;
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.clone()
    }
}

impl Default for AtlasStack {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Stdlib Functions
// ============================================================================

use crate::span::Span;
use crate::stdlib::stdlib_arity_error;
use crate::value::RuntimeError;
use std::sync::Arc;
use std::sync::Mutex;

/// Extract stack from value
fn extract_stack(value: &Value, span: Span) -> Result<Arc<Mutex<AtlasStack>>, RuntimeError> {
    match value {
        Value::Stack(stack) => Ok(Arc::clone(stack)),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Stack, got {}", value.type_name()),
            span,
        }),
    }
}

/// Create new empty stack
pub fn new_stack(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("Stack.new", 0, args.len(), span));
    }
    Ok(Value::Stack(Arc::new(Mutex::new(AtlasStack::new()))))
}

/// Push element onto top of stack
pub fn push(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("Stack.push", 2, args.len(), span));
    }

    let stack = extract_stack(&args[0], span)?;
    let element = args[1].clone();

    stack.lock().unwrap().push(element);
    Ok(Value::Null)
}

/// Pop element from top of stack
pub fn pop(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.pop", 1, args.len(), span));
    }

    let stack = extract_stack(&args[0], span)?;
    let value = stack.lock().unwrap().pop();

    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// View top element without removing
pub fn peek(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.peek", 1, args.len(), span));
    }

    let stack = extract_stack(&args[0], span)?;
    let value = stack.lock().unwrap().peek().cloned();

    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// Get number of elements in stack
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.size", 1, args.len(), span));
    }

    let stack = extract_stack(&args[0], span)?;
    let len = stack.lock().unwrap().len();
    Ok(Value::Number(len as f64))
}

/// Check if stack is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.isEmpty", 1, args.len(), span));
    }

    let stack = extract_stack(&args[0], span)?;
    let empty = stack.lock().unwrap().is_empty();
    Ok(Value::Bool(empty))
}

/// Remove all elements from stack
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.clear", 1, args.len(), span));
    }

    let stack = extract_stack(&args[0], span)?;
    stack.lock().unwrap().clear();
    Ok(Value::Null)
}

/// Convert stack to array (bottom to top order)
pub fn to_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.toArray", 1, args.len(), span));
    }

    let stack = extract_stack(&args[0], span)?;
    let elements = stack.lock().unwrap().to_vec();
    Ok(Value::Array(Arc::new(Mutex::new(elements))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_is_empty() {
        let stack = AtlasStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_push_increases_size() {
        let mut stack = AtlasStack::new();
        stack.push(Value::Number(1.0));
        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());
    }

    #[test]
    fn test_pop_lifo_order() {
        let mut stack = AtlasStack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));

        assert_eq!(stack.pop(), Some(Value::Number(3.0)));
        assert_eq!(stack.pop(), Some(Value::Number(2.0)));
        assert_eq!(stack.pop(), Some(Value::Number(1.0)));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_peek_doesnt_remove() {
        let mut stack = AtlasStack::new();
        stack.push(Value::Number(42.0));

        assert_eq!(stack.peek(), Some(&Value::Number(42.0)));
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.peek(), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_clear() {
        let mut stack = AtlasStack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_to_vec_bottom_to_top() {
        let mut stack = AtlasStack::new();
        stack.push(Value::Number(1.0));
        stack.push(Value::Number(2.0));
        stack.push(Value::Number(3.0));

        let vec = stack.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], Value::Number(1.0));
        assert_eq!(vec[1], Value::Number(2.0));
        assert_eq!(vec[2], Value::Number(3.0));
    }
}
