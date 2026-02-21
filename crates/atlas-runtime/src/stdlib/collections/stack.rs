//! Stack collection - LIFO (Last-In-First-Out)
//!
//! Atlas Stack implementation using Vec for O(1) push/pop operations.
//! Provides efficient LIFO stack semantics.

use crate::value::Value;

/// Atlas Stack - LIFO collection with O(1) push/pop
///
/// Backed by Vec for optimal performance.
/// Supports all standard stack operations: push, pop, peek.
#[derive(Debug, Clone, PartialEq)]
pub struct AtlasStack {
    inner: Vec<Value>,
}

impl AtlasStack {
    /// Create new empty stack
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Create stack with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Push element onto top of stack (LIFO order)
    pub fn push(&mut self, value: Value) {
        self.inner.push(value);
    }

    /// Pop element from top of stack
    ///
    /// Returns `None` if stack is empty.
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    /// View top element without removing
    ///
    /// Returns `None` if stack is empty.
    pub fn peek(&self) -> Option<&Value> {
        self.inner.last()
    }

    /// Get number of elements in stack
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements from stack
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert stack to array (bottom to top order)
    ///
    /// Bottom of stack becomes first array element.
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
use crate::value::{RuntimeError, ValueStack};

fn extract_stack_ref<'a>(value: &'a Value, span: Span) -> Result<&'a ValueStack, RuntimeError> {
    match value {
        Value::Stack(s) => Ok(s),
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
    Ok(Value::Stack(ValueStack::new()))
}

/// Push element onto top of stack. Returns modified Stack (CoW).
pub fn push(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("Stack.push", 2, args.len(), span));
    }

    let element = args[1].clone();
    let mut stack_val = args[0].clone();
    if let Value::Stack(ref mut s) = stack_val {
        s.inner_mut().push(element);
    }
    Ok(stack_val)
}

/// Pop element from top of stack.
/// Returns [Option<Value>, modified Stack].
pub fn pop(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.pop", 1, args.len(), span));
    }

    let mut stack_val = args[0].clone();
    let value = if let Value::Stack(ref mut s) = stack_val {
        s.inner_mut().pop()
    } else {
        return Err(RuntimeError::TypeError {
            msg: format!("Expected Stack, got {}", args[0].type_name()),
            span,
        });
    };

    let item_opt = match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    };
    Ok(Value::array(vec![item_opt, stack_val]))
}

/// View top element without removing
pub fn peek(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.peek", 1, args.len(), span));
    }

    let stack = extract_stack_ref(&args[0], span)?;
    let value = stack.inner().peek().cloned();

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

    let stack = extract_stack_ref(&args[0], span)?;
    Ok(Value::Number(stack.inner().len() as f64))
}

/// Check if stack is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.isEmpty", 1, args.len(), span));
    }

    let stack = extract_stack_ref(&args[0], span)?;
    Ok(Value::Bool(stack.inner().is_empty()))
}

/// Remove all elements from stack. Returns cleared Stack.
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.clear", 1, args.len(), span));
    }

    let mut stack_val = args[0].clone();
    if let Value::Stack(ref mut s) = stack_val {
        s.inner_mut().clear();
    }
    Ok(stack_val)
}

/// Convert stack to array (bottom to top order)
pub fn to_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Stack.toArray", 1, args.len(), span));
    }

    let stack = extract_stack_ref(&args[0], span)?;
    let elements = stack.inner().to_vec();
    Ok(Value::array(elements))
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
