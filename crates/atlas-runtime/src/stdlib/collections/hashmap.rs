//! HashMap collection implementation

use super::hash::HashKey;
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Atlas HashMap - key-value collection with O(1) average operations
///
/// Uses Rust's standard HashMap internally with deterministic hashing.
/// Only hashable types (number, string, bool, null) can be used as keys.
#[derive(Debug, Clone)]
pub struct AtlasHashMap {
    inner: HashMap<HashKey, Value>,
}

impl AtlasHashMap {
    /// Create new empty HashMap with default capacity
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Create HashMap with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Insert key-value pair, returns previous value if existed
    pub fn insert(&mut self, key: HashKey, value: Value) -> Option<Value> {
        self.inner.insert(key, value)
    }

    /// Get value by key (returns None if not found)
    pub fn get(&self, key: &HashKey) -> Option<&Value> {
        self.inner.get(key)
    }

    /// Remove key-value pair, returns value if existed
    pub fn remove(&mut self, key: &HashKey) -> Option<Value> {
        self.inner.remove(key)
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &HashKey) -> bool {
        self.inner.contains_key(key)
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all entries
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get all keys as vector
    pub fn keys(&self) -> Vec<HashKey> {
        self.inner.keys().cloned().collect()
    }

    /// Get all values as vector
    pub fn values(&self) -> Vec<Value> {
        self.inner.values().cloned().collect()
    }

    /// Get all entries as vector of (key, value) pairs
    pub fn entries(&self) -> Vec<(HashKey, Value)> {
        self.inner
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for AtlasHashMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract array from value with better error message
fn extract_array(value: &Value, span: Span) -> Result<Rc<RefCell<Vec<Value>>>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(Rc::clone(arr)),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Extract hashmap from value
fn extract_hashmap(value: &Value, span: Span) -> Result<Rc<RefCell<AtlasHashMap>>, RuntimeError> {
    match value {
        Value::HashMap(map) => Ok(Rc::clone(map)),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected HashMap, got {}", value.type_name()),
            span,
        }),
    }
}

/// Create new empty HashMap
///
/// # Arguments
/// * `args` - Empty (no arguments)
///
/// # Returns
/// New empty HashMap
pub fn new_map(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }
    Ok(Value::HashMap(Rc::new(RefCell::new(AtlasHashMap::new()))))
}

/// Create HashMap from array of [key, value] entries
///
/// # Arguments
/// * `args[0]` - Array of [key, value] pairs
///
/// # Returns
/// HashMap with entries
pub fn from_entries(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let entries_array = extract_array(&args[0], span)?;
    let mut map = AtlasHashMap::new();

    for entry in entries_array.borrow().iter() {
        let pair = extract_array(entry, span)?;
        let pair_borrow = pair.borrow();

        if pair_borrow.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "Entry must be [key, value] array with exactly 2 elements".to_string(),
                span,
            });
        }

        let key = HashKey::from_value(&pair_borrow[0], span)?;
        let value = pair_borrow[1].clone();
        map.insert(key, value);
    }

    Ok(Value::HashMap(Rc::new(RefCell::new(map))))
}

/// Insert or update key-value pair
///
/// # Arguments
/// * `args[0]` - HashMap
/// * `args[1]` - Key (hashable type)
/// * `args[2]` - Value
///
/// # Returns
/// Null
pub fn put(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;
    let value = args[2].clone();

    map.borrow_mut().insert(key, value);
    Ok(Value::Null)
}

/// Get value by key
///
/// # Arguments
/// * `args[0]` - HashMap
/// * `args[1]` - Key
///
/// # Returns
/// Option<Value> - Some(value) if key exists, None otherwise
pub fn get(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let value = map.borrow().get(&key).cloned();
    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// Remove key-value pair
///
/// # Arguments
/// * `args[0]` - HashMap
/// * `args[1]` - Key
///
/// # Returns
/// Option<Value> - Some(value) if key existed, None otherwise
pub fn remove(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let removed = map.borrow_mut().remove(&key);
    Ok(match removed {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// Check if key exists
///
/// # Arguments
/// * `args[0]` - HashMap
/// * `args[1]` - Key
///
/// # Returns
/// Bool - true if key exists, false otherwise
pub fn has(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let exists = map.borrow().contains_key(&key);
    Ok(Value::Bool(exists))
}

/// Get number of entries
///
/// # Arguments
/// * `args[0]` - HashMap
///
/// # Returns
/// Number - count of entries
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let len = map.borrow().len();
    Ok(Value::Number(len as f64))
}

/// Check if HashMap is empty
///
/// # Arguments
/// * `args[0]` - HashMap
///
/// # Returns
/// Bool - true if empty, false otherwise
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let empty = map.borrow().is_empty();
    Ok(Value::Bool(empty))
}

/// Remove all entries
///
/// # Arguments
/// * `args[0]` - HashMap
///
/// # Returns
/// Null
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    map.borrow_mut().clear();
    Ok(Value::Null)
}

/// Get all keys as array
///
/// # Arguments
/// * `args[0]` - HashMap
///
/// # Returns
/// Array of keys
pub fn keys(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let keys: Vec<Value> = map
        .borrow()
        .keys()
        .into_iter()
        .map(|k| k.to_value())
        .collect();
    Ok(Value::Array(Rc::new(RefCell::new(keys))))
}

/// Get all values as array
///
/// # Arguments
/// * `args[0]` - HashMap
///
/// # Returns
/// Array of values
pub fn values(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let vals = {
        let borrowed = map.borrow();
        borrowed.values()
    };
    Ok(Value::Array(Rc::new(RefCell::new(vals))))
}

/// Get all entries as array of [key, value] pairs
///
/// # Arguments
/// * `args[0]` - HashMap
///
/// # Returns
/// Array of [key, value] arrays
pub fn entries(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let map = extract_hashmap(&args[0], span)?;
    let entries: Vec<Value> = map
        .borrow()
        .entries()
        .into_iter()
        .map(|(k, v)| {
            let pair = vec![k.to_value(), v];
            Value::Array(Rc::new(RefCell::new(pair)))
        })
        .collect();
    Ok(Value::Array(Rc::new(RefCell::new(entries))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_hashmap_new() {
        let map = AtlasHashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_atlas_hashmap_insert_get() {
        let mut map = AtlasHashMap::new();
        let key = HashKey::Number(ordered_float::OrderedFloat(42.0));
        let value = Value::string("hello");

        map.insert(key.clone(), value.clone());
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&key), Some(&value));
    }

    #[test]
    fn test_hashmap_new_function() {
        let result = new_map(&[], Span::dummy()).unwrap();
        match result {
            Value::HashMap(map) => {
                assert_eq!(map.borrow().len(), 0);
            }
            _ => panic!("Expected HashMap"),
        }
    }

    #[test]
    fn test_hashmap_put_get() {
        let map_value = new_map(&[], Span::dummy()).unwrap();
        let key = Value::string("name");
        let value = Value::string("Alice");

        // Put
        put(
            &[map_value.clone(), key.clone(), value.clone()],
            Span::dummy(),
        )
        .unwrap();

        // Get
        let result = get(&[map_value, key], Span::dummy()).unwrap();
        match result {
            Value::Option(Some(v)) => {
                assert_eq!(*v, value);
            }
            _ => panic!("Expected Some(value)"),
        }
    }
}
