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
#[derive(Debug, Clone, PartialEq)]
pub struct AtlasQueue {
    inner: VecDeque<Value>,
}

impl AtlasQueue {
    /// Create new empty queue
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    /// Create queue with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
        }
    }

    /// Add element to back of queue (FIFO order)
    pub fn enqueue(&mut self, value: Value) {
        self.inner.push_back(value);
    }

    /// Remove and return element from front of queue
    ///
    /// Returns `None` if queue is empty.
    pub fn dequeue(&mut self) -> Option<Value> {
        self.inner.pop_front()
    }

    /// View front element without removing
    ///
    /// Returns `None` if queue is empty.
    pub fn peek(&self) -> Option<&Value> {
        self.inner.front()
    }

    /// Get number of elements in queue
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements from queue
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert queue to array (preserves FIFO order)
    ///
    /// Front of queue becomes first array element.
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
use crate::stdlib::stdlib_arity_error;
use crate::value::{RuntimeError, ValueQueue};

fn extract_queue_ref(value: &Value, span: Span) -> Result<&ValueQueue, RuntimeError> {
    match value {
        Value::Queue(q) => Ok(q),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Queue, got {}", value.type_name()),
            span,
        }),
    }
}

/// Create new empty queue
pub fn new_queue(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("Queue.new", 0, args.len(), span));
    }
    Ok(Value::Queue(ValueQueue::new()))
}

/// Add element to back of queue. Returns modified Queue (CoW).
pub fn enqueue(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("Queue.enqueue", 2, args.len(), span));
    }

    let element = args[1].clone();
    let mut queue_val = args[0].clone();
    if let Value::Queue(ref mut q) = queue_val {
        q.inner_mut().enqueue(element);
    }
    Ok(queue_val)
}

/// Remove and return element from front of queue.
/// Returns [Option<Value>, modified Queue].
pub fn dequeue(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Queue.dequeue", 1, args.len(), span));
    }

    let mut queue_val = args[0].clone();
    let value = if let Value::Queue(ref mut q) = queue_val {
        q.inner_mut().dequeue()
    } else {
        return Err(RuntimeError::TypeError {
            msg: format!("Expected Queue, got {}", args[0].type_name()),
            span,
        });
    };

    let item_opt = match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    };
    Ok(Value::array(vec![item_opt, queue_val]))
}

/// View front element without removing
pub fn peek(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Queue.peek", 1, args.len(), span));
    }

    let queue = extract_queue_ref(&args[0], span)?;
    let value = queue.inner().peek().cloned();

    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// Get number of elements in queue
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Queue.size", 1, args.len(), span));
    }

    let queue = extract_queue_ref(&args[0], span)?;
    Ok(Value::Number(queue.inner().len() as f64))
}

/// Check if queue is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Queue.isEmpty", 1, args.len(), span));
    }

    let queue = extract_queue_ref(&args[0], span)?;
    Ok(Value::Bool(queue.inner().is_empty()))
}

/// Remove all elements from queue. Returns cleared Queue.
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Queue.clear", 1, args.len(), span));
    }

    let mut queue_val = args[0].clone();
    if let Value::Queue(ref mut q) = queue_val {
        q.inner_mut().clear();
    }
    Ok(queue_val)
}

/// Convert queue to array (FIFO order)
pub fn to_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("Queue.toArray", 1, args.len(), span));
    }

    let queue = extract_queue_ref(&args[0], span)?;
    let elements = queue.inner().to_vec();
    Ok(Value::array(elements))
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
