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
use crate::stdlib::{stdlib_arg_error, stdlib_arity_error};
use std::collections::HashSet as RustHashSet;

/// Atlas HashSet - unique value collection with O(1) operations
/// Backed by Rust's HashSet for proven performance
#[derive(Debug, Clone, PartialEq)]
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
use std::sync::Arc;
use std::sync::Mutex;

/// Create a new empty HashSet
pub fn new_set(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("HashSet.new", 0, args.len(), span));
    }

    Ok(Value::HashSet(Arc::new(Mutex::new(AtlasHashSet::new()))))
}

/// Create HashSet from array of hashable elements
pub fn from_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashSet.fromArray", 1, args.len(), span));
    }

    let array = extract_array("HashSet.fromArray", &args[0], span)?;
    let mut set = AtlasHashSet::new();

    for element in array {
        let key = HashKey::from_value(&element, span)?;
        set.insert(key);
    }

    Ok(Value::HashSet(Arc::new(Mutex::new(set))))
}

/// Add element to HashSet
pub fn add(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashSet.add", 2, args.len(), span));
    }

    let set = extract_hashset("HashSet.add", &args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    set.lock().unwrap().insert(key);
    Ok(Value::Null)
}

/// Remove element from HashSet
pub fn remove(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashSet.remove", 2, args.len(), span));
    }

    let set = extract_hashset("HashSet.remove", &args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let existed = set.lock().unwrap().remove(&key);
    Ok(Value::Bool(existed))
}

/// Check if element exists in HashSet
pub fn has(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashSet.has", 2, args.len(), span));
    }

    let set = extract_hashset("HashSet.has", &args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let exists = set.lock().unwrap().contains(&key);
    Ok(Value::Bool(exists))
}

/// Get number of elements in HashSet
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashSet.size", 1, args.len(), span));
    }

    let set = extract_hashset("HashSet.size", &args[0], span)?;
    let len = set.lock().unwrap().len();
    Ok(Value::Number(len as f64))
}

/// Check if HashSet is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashSet.isEmpty", 1, args.len(), span));
    }

    let set = extract_hashset("HashSet.isEmpty", &args[0], span)?;
    let empty = set.lock().unwrap().is_empty();
    Ok(Value::Bool(empty))
}

/// Clear all elements from HashSet
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashSet.clear", 1, args.len(), span));
    }

    let set = extract_hashset("HashSet.clear", &args[0], span)?;
    set.lock().unwrap().clear();
    Ok(Value::Null)
}

/// Union of two HashSets
pub fn union(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashSet.union", 2, args.len(), span));
    }

    let set_a = extract_hashset("HashSet.union", &args[0], span)?;
    let set_b = extract_hashset("HashSet.union", &args[1], span)?;

    let result = set_a.lock().unwrap().union(&set_b.lock().unwrap());
    Ok(Value::HashSet(Arc::new(Mutex::new(result))))
}

/// Intersection of two HashSets
pub fn intersection(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error(
            "HashSet.intersection",
            2,
            args.len(),
            span,
        ));
    }

    let set_a = extract_hashset("HashSet.intersection", &args[0], span)?;
    let set_b = extract_hashset("HashSet.intersection", &args[1], span)?;

    let result = set_a.lock().unwrap().intersection(&set_b.lock().unwrap());
    Ok(Value::HashSet(Arc::new(Mutex::new(result))))
}

/// Difference of two HashSets (elements in A but not in B)
pub fn difference(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error(
            "HashSet.difference",
            2,
            args.len(),
            span,
        ));
    }

    let set_a = extract_hashset("HashSet.difference", &args[0], span)?;
    let set_b = extract_hashset("HashSet.difference", &args[1], span)?;

    let result = set_a.lock().unwrap().difference(&set_b.lock().unwrap());
    Ok(Value::HashSet(Arc::new(Mutex::new(result))))
}

/// Symmetric difference of two HashSets (elements in exactly one set)
pub fn symmetric_difference(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error(
            "HashSet.symmetricDifference",
            2,
            args.len(),
            span,
        ));
    }

    let set_a = extract_hashset("HashSet.symmetricDifference", &args[0], span)?;
    let set_b = extract_hashset("HashSet.symmetricDifference", &args[1], span)?;

    let result = set_a
        .lock()
        .unwrap()
        .symmetric_difference(&set_b.lock().unwrap());
    Ok(Value::HashSet(Arc::new(Mutex::new(result))))
}

/// Check if set A is subset of set B
pub fn is_subset(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashSet.isSubset", 2, args.len(), span));
    }

    let set_a = extract_hashset("HashSet.isSubset", &args[0], span)?;
    let set_b = extract_hashset("HashSet.isSubset", &args[1], span)?;

    let is_sub = set_a.lock().unwrap().is_subset(&set_b.lock().unwrap());
    Ok(Value::Bool(is_sub))
}

/// Check if set A is superset of set B
pub fn is_superset(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error(
            "HashSet.isSuperset",
            2,
            args.len(),
            span,
        ));
    }

    let set_a = extract_hashset("HashSet.isSuperset", &args[0], span)?;
    let set_b = extract_hashset("HashSet.isSuperset", &args[1], span)?;

    let is_super = set_a.lock().unwrap().is_superset(&set_b.lock().unwrap());
    Ok(Value::Bool(is_super))
}

/// Convert HashSet to array
pub fn to_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashSet.toArray", 1, args.len(), span));
    }

    let set = extract_hashset("HashSet.toArray", &args[0], span)?;
    let elements: Vec<Value> = set
        .lock()
        .unwrap()
        .to_vec()
        .into_iter()
        .map(|key| key.to_value())
        .collect();
    Ok(Value::Array(Arc::new(Mutex::new(elements))))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract HashSet from value
fn extract_hashset(
    func_name: &str,
    value: &Value,
    span: Span,
) -> Result<Arc<Mutex<AtlasHashSet>>, RuntimeError> {
    match value {
        Value::HashSet(set) => Ok(Arc::clone(set)),
        _ => Err(stdlib_arg_error(func_name, "HashSet", value, span)),
    }
}

/// Extract array from value
fn extract_array(func_name: &str, value: &Value, span: Span) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr.lock().unwrap().clone()),
        _ => Err(stdlib_arg_error(func_name, "array", value, span)),
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
