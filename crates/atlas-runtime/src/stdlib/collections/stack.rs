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
    /// ```
    /// let stack = AtlasStack::new();
    /// assert!(stack.is_empty());
    /// ```
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Create stack with pre-allocated capacity
    ///
    /// Useful for performance when stack size is known.
    ///
    /// # Example
    /// ```
    /// let stack = AtlasStack::with_capacity(100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Push element onto top of stack (LIFO order)
    ///
    /// # Example
    /// ```
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// stack.push(Value::String("hello".into()));
    /// ```
    pub fn push(&mut self, value: Value) {
        self.inner.push(value);
    }

    /// Pop element from top of stack
    ///
    /// Returns `None` if stack is empty.
    ///
    /// # Example
    /// ```
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// assert_eq!(stack.pop(), Some(Value::Number(1.0)));
    /// assert_eq!(stack.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    /// View top element without removing
    ///
    /// Returns `None` if stack is empty.
    ///
    /// # Example
    /// ```
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(42.0));
    /// assert_eq!(stack.peek(), Some(&Value::Number(42.0)));
    /// assert_eq!(stack.len(), 1); // Still has 1 element
    /// ```
    pub fn peek(&self) -> Option<&Value> {
        self.inner.last()
    }

    /// Get number of elements in stack
    ///
    /// # Example
    /// ```
    /// let mut stack = AtlasStack::new();
    /// assert_eq!(stack.len(), 0);
    /// stack.push(Value::Number(1.0));
    /// assert_eq!(stack.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if stack is empty
    ///
    /// # Example
    /// ```
    /// let stack = AtlasStack::new();
    /// assert!(stack.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements from stack
    ///
    /// # Example
    /// ```
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// stack.clear();
    /// assert!(stack.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert stack to array (bottom to top order)
    ///
    /// Bottom of stack becomes first array element.
    ///
    /// # Example
    /// ```
    /// let mut stack = AtlasStack::new();
    /// stack.push(Value::Number(1.0));
    /// stack.push(Value::Number(2.0));
    /// let arr = stack.to_vec();
    /// assert_eq!(arr, vec![Value::Number(1.0), Value::Number(2.0)]);
    /// ```
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
use crate::value::RuntimeError;
use std::cell::RefCell;
use std::rc::Rc;

/// Extract stack from value
fn extract_stack(value: &Value, span: Span) -> Result<Rc<RefCell<AtlasStack>>, RuntimeError> {
    match value {
        Value::Stack(stack) => Ok(Rc::clone(stack)),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Stack, got {}", value.type_name()),
            span,
        }),
    }
}

/// Create new empty stack
pub fn new_stack(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }
    Ok(Value::Stack(Rc::new(RefCell::new(AtlasStack::new()))))
}

/// Push element onto top of stack
pub fn push(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let stack = extract_stack(&args[0], span)?;
    let element = args[1].clone();

    stack.borrow_mut().push(element);
    Ok(Value::Null)
}

/// Pop element from top of stack
pub fn pop(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let stack = extract_stack(&args[0], span)?;
    let value = stack.borrow_mut().pop();

    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// View top element without removing
pub fn peek(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let stack = extract_stack(&args[0], span)?;
    let value = stack.borrow().peek().cloned();

    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// Get number of elements in stack
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let stack = extract_stack(&args[0], span)?;
    let len = stack.borrow().len();
    Ok(Value::Number(len as f64))
}

/// Check if stack is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let stack = extract_stack(&args[0], span)?;
    let empty = stack.borrow().is_empty();
    Ok(Value::Bool(empty))
}

/// Remove all elements from stack
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let stack = extract_stack(&args[0], span)?;
    stack.borrow_mut().clear();
    Ok(Value::Null)
}

/// Convert stack to array (bottom to top order)
pub fn to_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let stack = extract_stack(&args[0], span)?;
    let elements = stack.borrow().to_vec();
    Ok(Value::Array(Rc::new(RefCell::new(elements))))
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
