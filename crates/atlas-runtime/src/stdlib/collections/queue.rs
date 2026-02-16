//! Queue collection - FIFO (First-In-First-Out)
//!
//! Atlas Queue implementation using VecDeque for O(1) enqueue/dequeue operations.
//! Provides circular buffer for efficient FIFO queue semantics.

use crate::value::Value;
use std::collections::VecDeque;

/// Atlas Queue - FIFO collection with O(1) enqueue/dequeue
///
/// Backed by VecDeque (circular buffer) for optimal performance.
/// Supports all standard queue operations: enqueue, dequeue, peek.
#[derive(Debug, Clone)]
pub struct AtlasQueue {
    inner: VecDeque<Value>,
}

impl AtlasQueue {
    /// Create new empty queue
    ///
    /// # Example
    /// ```
    /// let queue = AtlasQueue::new();
    /// assert!(queue.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    /// Create queue with pre-allocated capacity
    ///
    /// Useful for performance when queue size is known.
    ///
    /// # Example
    /// ```
    /// let queue = AtlasQueue::with_capacity(100);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
        }
    }

    /// Add element to back of queue (FIFO order)
    ///
    /// # Example
    /// ```
    /// let mut queue = AtlasQueue::new();
    /// queue.enqueue(Value::Number(1.0));
    /// queue.enqueue(Value::String("hello".into()));
    /// ```
    pub fn enqueue(&mut self, value: Value) {
        self.inner.push_back(value);
    }

    /// Remove and return element from front of queue
    ///
    /// Returns `None` if queue is empty.
    ///
    /// # Example
    /// ```
    /// let mut queue = AtlasQueue::new();
    /// queue.enqueue(Value::Number(1.0));
    /// assert_eq!(queue.dequeue(), Some(Value::Number(1.0)));
    /// assert_eq!(queue.dequeue(), None);
    /// ```
    pub fn dequeue(&mut self) -> Option<Value> {
        self.inner.pop_front()
    }

    /// View front element without removing
    ///
    /// Returns `None` if queue is empty.
    ///
    /// # Example
    /// ```
    /// let mut queue = AtlasQueue::new();
    /// queue.enqueue(Value::Number(42.0));
    /// assert_eq!(queue.peek(), Some(&Value::Number(42.0)));
    /// assert_eq!(queue.len(), 1); // Still has 1 element
    /// ```
    pub fn peek(&self) -> Option<&Value> {
        self.inner.front()
    }

    /// Get number of elements in queue
    ///
    /// # Example
    /// ```
    /// let mut queue = AtlasQueue::new();
    /// assert_eq!(queue.len(), 0);
    /// queue.enqueue(Value::Number(1.0));
    /// assert_eq!(queue.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if queue is empty
    ///
    /// # Example
    /// ```
    /// let queue = AtlasQueue::new();
    /// assert!(queue.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements from queue
    ///
    /// # Example
    /// ```
    /// let mut queue = AtlasQueue::new();
    /// queue.enqueue(Value::Number(1.0));
    /// queue.clear();
    /// assert!(queue.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert queue to array (preserves FIFO order)
    ///
    /// Front of queue becomes first array element.
    ///
    /// # Example
    /// ```
    /// let mut queue = AtlasQueue::new();
    /// queue.enqueue(Value::Number(1.0));
    /// queue.enqueue(Value::Number(2.0));
    /// let arr = queue.to_vec();
    /// assert_eq!(arr, vec![Value::Number(1.0), Value::Number(2.0)]);
    /// ```
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.iter().cloned().collect()
    }
}

impl Default for AtlasQueue {
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

/// Extract queue from value
fn extract_queue(value: &Value, span: Span) -> Result<Rc<RefCell<AtlasQueue>>, RuntimeError> {
    match value {
        Value::Queue(queue) => Ok(Rc::clone(queue)),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Queue, got {}", value.type_name()),
            span,
        }),
    }
}

/// Create new empty queue
pub fn new_queue(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }
    Ok(Value::Queue(Rc::new(RefCell::new(AtlasQueue::new()))))
}

/// Add element to back of queue
pub fn enqueue(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let queue = extract_queue(&args[0], span)?;
    let element = args[1].clone();

    queue.borrow_mut().enqueue(element);
    Ok(Value::Null)
}

/// Remove and return element from front of queue
pub fn dequeue(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let queue = extract_queue(&args[0], span)?;
    let value = queue.borrow_mut().dequeue();

    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// View front element without removing
pub fn peek(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let queue = extract_queue(&args[0], span)?;
    let value = queue.borrow().peek().cloned();

    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// Get number of elements in queue
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let queue = extract_queue(&args[0], span)?;
    let len = queue.borrow().len();
    Ok(Value::Number(len as f64))
}

/// Check if queue is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let queue = extract_queue(&args[0], span)?;
    let empty = queue.borrow().is_empty();
    Ok(Value::Bool(empty))
}

/// Remove all elements from queue
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let queue = extract_queue(&args[0], span)?;
    queue.borrow_mut().clear();
    Ok(Value::Null)
}

/// Convert queue to array (FIFO order)
pub fn to_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let queue = extract_queue(&args[0], span)?;
    let elements = queue.borrow().to_vec();
    Ok(Value::Array(Rc::new(RefCell::new(elements))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue_is_empty() {
        let queue = AtlasQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_enqueue_increases_size() {
        let mut queue = AtlasQueue::new();
        queue.enqueue(Value::Number(1.0));
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[test]
    fn test_dequeue_fifo_order() {
        let mut queue = AtlasQueue::new();
        queue.enqueue(Value::Number(1.0));
        queue.enqueue(Value::Number(2.0));
        queue.enqueue(Value::Number(3.0));

        assert_eq!(queue.dequeue(), Some(Value::Number(1.0)));
        assert_eq!(queue.dequeue(), Some(Value::Number(2.0)));
        assert_eq!(queue.dequeue(), Some(Value::Number(3.0)));
        assert_eq!(queue.dequeue(), None);
    }

    #[test]
    fn test_peek_doesnt_remove() {
        let mut queue = AtlasQueue::new();
        queue.enqueue(Value::Number(42.0));

        assert_eq!(queue.peek(), Some(&Value::Number(42.0)));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.peek(), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_clear() {
        let mut queue = AtlasQueue::new();
        queue.enqueue(Value::Number(1.0));
        queue.enqueue(Value::Number(2.0));

        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_to_vec_preserves_order() {
        let mut queue = AtlasQueue::new();
        queue.enqueue(Value::Number(1.0));
        queue.enqueue(Value::Number(2.0));
        queue.enqueue(Value::Number(3.0));

        let vec = queue.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], Value::Number(1.0));
        assert_eq!(vec[1], Value::Number(2.0));
        assert_eq!(vec[2], Value::Number(3.0));
    }
}
