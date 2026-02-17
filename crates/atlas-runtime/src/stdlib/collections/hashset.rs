//! HashSet Collection - Unique Value Storage
//!
//! Provides efficient set operations with O(1) average-case membership testing.
//! Backed by Rust's HashSet for proven performance and correctness.
//!
//! ## Features
//! - Unique value storage (automatic deduplication)
//! - Fast membership testing (O(1) average)
//! - Set operations: union, intersection, difference, symmetric difference
//! - Subset/superset testing
//! - Support for hashable types: number, string, bool, null

use crate::stdlib::collections::hash::HashKey;
use std::collections::HashSet as RustHashSet;

/// Atlas HashSet - unique value collection with O(1) operations
/// Backed by Rust's HashSet for proven performance
#[derive(Debug, Clone)]
pub struct AtlasHashSet {
    inner: RustHashSet<HashKey>,
}

impl AtlasHashSet {
    /// Create new empty HashSet
    pub fn new() -> Self {
        Self {
            inner: RustHashSet::new(),
        }
    }

    /// Create HashSet with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RustHashSet::with_capacity(capacity),
        }
    }

    /// Add element to set, returns true if inserted (false if already existed)
    pub fn insert(&mut self, element: HashKey) -> bool {
        self.inner.insert(element)
    }

    /// Remove element from set, returns true if existed
    pub fn remove(&mut self, element: &HashKey) -> bool {
        self.inner.remove(element)
    }

    /// Check if element exists in set
    pub fn contains(&self, element: &HashKey) -> bool {
        self.inner.contains(element)
    }

    /// Get number of elements
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert to vector of elements
    pub fn to_vec(&self) -> Vec<HashKey> {
        self.inner.iter().cloned().collect()
    }

    /// Set union: all elements in either set
    pub fn union(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self.inner.union(&other.inner).cloned().collect(),
        }
    }

    /// Set intersection: elements in both sets
    pub fn intersection(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self.inner.intersection(&other.inner).cloned().collect(),
        }
    }

    /// Set difference: elements in self but not in other
    pub fn difference(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self.inner.difference(&other.inner).cloned().collect(),
        }
    }

    /// Symmetric difference: elements in exactly one set
    pub fn symmetric_difference(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self
                .inner
                .symmetric_difference(&other.inner)
                .cloned()
                .collect(),
        }
    }

    /// Check if self is subset of other
    pub fn is_subset(&self, other: &AtlasHashSet) -> bool {
        self.inner.is_subset(&other.inner)
    }

    /// Check if self is superset of other
    pub fn is_superset(&self, other: &AtlasHashSet) -> bool {
        self.inner.is_superset(&other.inner)
    }

    /// Check if sets are disjoint (no common elements)
    pub fn is_disjoint(&self, other: &AtlasHashSet) -> bool {
        self.inner.is_disjoint(&other.inner)
    }
}

impl Default for AtlasHashSet {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Stdlib Functions
// ============================================================================

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;
use std::sync::Arc;

/// Create a new empty HashSet
pub fn new_set(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::HashSet(Arc::new(RefCell::new(AtlasHashSet::new()))))
}

/// Create HashSet from array of hashable elements
pub fn from_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let array = extract_array(&args[0], span)?;
    let mut set = AtlasHashSet::new();

    for element in array {
        let key = HashKey::from_value(&element, span)?;
        set.insert(key);
    }

    Ok(Value::HashSet(Arc::new(RefCell::new(set))))
}

/// Add element to HashSet
pub fn add(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set = extract_hashset(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    set.borrow_mut().insert(key);
    Ok(Value::Null)
}

/// Remove element from HashSet
pub fn remove(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set = extract_hashset(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let existed = set.borrow_mut().remove(&key);
    Ok(Value::Bool(existed))
}

/// Check if element exists in HashSet
pub fn has(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set = extract_hashset(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let exists = set.borrow().contains(&key);
    Ok(Value::Bool(exists))
}

/// Get number of elements in HashSet
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set = extract_hashset(&args[0], span)?;
    let len = set.borrow().len();
    Ok(Value::Number(len as f64))
}

/// Check if HashSet is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set = extract_hashset(&args[0], span)?;
    let empty = set.borrow().is_empty();
    Ok(Value::Bool(empty))
}

/// Clear all elements from HashSet
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set = extract_hashset(&args[0], span)?;
    set.borrow_mut().clear();
    Ok(Value::Null)
}

/// Union of two HashSets
pub fn union(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set_a = extract_hashset(&args[0], span)?;
    let set_b = extract_hashset(&args[1], span)?;

    let result = set_a.borrow().union(&set_b.borrow());
    Ok(Value::HashSet(Arc::new(RefCell::new(result))))
}

/// Intersection of two HashSets
pub fn intersection(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set_a = extract_hashset(&args[0], span)?;
    let set_b = extract_hashset(&args[1], span)?;

    let result = set_a.borrow().intersection(&set_b.borrow());
    Ok(Value::HashSet(Arc::new(RefCell::new(result))))
}

/// Difference of two HashSets (elements in A but not in B)
pub fn difference(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set_a = extract_hashset(&args[0], span)?;
    let set_b = extract_hashset(&args[1], span)?;

    let result = set_a.borrow().difference(&set_b.borrow());
    Ok(Value::HashSet(Arc::new(RefCell::new(result))))
}

/// Symmetric difference of two HashSets (elements in exactly one set)
pub fn symmetric_difference(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set_a = extract_hashset(&args[0], span)?;
    let set_b = extract_hashset(&args[1], span)?;

    let result = set_a.borrow().symmetric_difference(&set_b.borrow());
    Ok(Value::HashSet(Arc::new(RefCell::new(result))))
}

/// Check if set A is subset of set B
pub fn is_subset(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set_a = extract_hashset(&args[0], span)?;
    let set_b = extract_hashset(&args[1], span)?;

    let is_sub = set_a.borrow().is_subset(&set_b.borrow());
    Ok(Value::Bool(is_sub))
}

/// Check if set A is superset of set B
pub fn is_superset(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set_a = extract_hashset(&args[0], span)?;
    let set_b = extract_hashset(&args[1], span)?;

    let is_super = set_a.borrow().is_superset(&set_b.borrow());
    Ok(Value::Bool(is_super))
}

/// Convert HashSet to array
pub fn to_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let set = extract_hashset(&args[0], span)?;
    let elements: Vec<Value> = set
        .borrow()
        .to_vec()
        .into_iter()
        .map(|key| key.to_value())
        .collect();
    Ok(Value::Array(Arc::new(RefCell::new(elements))))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract HashSet from value
fn extract_hashset(value: &Value, span: Span) -> Result<Arc<RefCell<AtlasHashSet>>, RuntimeError> {
    match value {
        Value::HashSet(set) => Ok(Arc::clone(set)),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Extract array from value
fn extract_array(value: &Value, span: Span) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr.borrow().clone()),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_hashset_new() {
        let set = AtlasHashSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_atlas_hashset_insert() {
        let mut set = AtlasHashSet::new();
        let key = HashKey::Number(ordered_float::OrderedFloat(42.0));

        // First insert returns true
        assert!(set.insert(key.clone()));
        assert_eq!(set.len(), 1);

        // Second insert returns false (already exists)
        assert!(!set.insert(key));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_atlas_hashset_contains() {
        let mut set = AtlasHashSet::new();
        let key = HashKey::String("test".to_string().into());

        assert!(!set.contains(&key));
        set.insert(key.clone());
        assert!(set.contains(&key));
    }

    #[test]
    fn test_atlas_hashset_remove() {
        let mut set = AtlasHashSet::new();
        let key = HashKey::Bool(true);

        set.insert(key.clone());
        assert!(set.remove(&key));
        assert_eq!(set.len(), 0);
        assert!(!set.remove(&key)); // Second remove returns false
    }

    #[test]
    fn test_atlas_hashset_union() {
        let mut set_a = AtlasHashSet::new();
        let mut set_b = AtlasHashSet::new();

        set_a.insert(HashKey::Number(ordered_float::OrderedFloat(1.0)));
        set_a.insert(HashKey::Number(ordered_float::OrderedFloat(2.0)));

        set_b.insert(HashKey::Number(ordered_float::OrderedFloat(2.0)));
        set_b.insert(HashKey::Number(ordered_float::OrderedFloat(3.0)));

        let union = set_a.union(&set_b);
        assert_eq!(union.len(), 3); // {1, 2, 3}
    }
}
