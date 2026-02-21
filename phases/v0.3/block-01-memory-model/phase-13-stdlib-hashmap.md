# Phase 13: Stdlib — HashMap Module

**Block:** 1 (Memory Model)
**Depends on:** Phase 03 complete (ValueHashMap exists)

---

## Objective

Update `stdlib/collections/hashmap.rs` to use the `ValueHashMap` CoW wrapper.
Remove all `Arc<Mutex<AtlasHashMap>>` extraction patterns. Update `extract_hashmap`
helper and all functions that use it.

---

## Current State (verified 2026-02-21)

`hashmap.rs` has:
- `extract_hashmap` returning `Result<Arc<Mutex<AtlasHashMap>>, RuntimeError>` (line 108)
- `extract_array` returning `Result<Arc<Mutex<Vec<Value>>>, RuntimeError>` (line 100)
  (this helper returns a Vec for iteration — update to `ValueArray`)

All hashmap stdlib functions call `extract_hashmap` then `.lock().unwrap()`.

---

## Implementation

### Update `extract_hashmap`
```rust
// OLD:
fn extract_hashmap(value: &Value, span: Span) -> Result<Arc<Mutex<AtlasHashMap>>, RuntimeError> {
    match value {
        Value::HashMap(m) => Ok(Arc::clone(m)),
        _ => Err(RuntimeError::type_error("expected HashMap", span))
    }
}

// NEW — for reads:
fn extract_hashmap_ref(value: &Value, span: Span) -> Result<&ValueHashMap, RuntimeError> {
    match value {
        Value::HashMap(m) => Ok(m),
        _ => Err(RuntimeError::type_error("expected HashMap", span))
    }
}

// For mutations, the caller clones:
// let mut map = args[0].clone();
// if let Value::HashMap(ref mut m) = map { m.inner_mut().insert(...); }
// Ok(map)
```

### Update `extract_array` in hashmap.rs

This helper (line 100) returns a `Vec<Value>` from an array argument. Update:
```rust
// OLD: Result<Arc<Mutex<Vec<Value>>>, RuntimeError>
// NEW: return the inner slice directly
fn extract_array_ref(value: &Value, span: Span) -> Result<&ValueArray, RuntimeError> {
    match value {
        Value::Array(a) => Ok(a),
        _ => Err(RuntimeError::type_error("expected array", span))
    }
}
```

### Update all hashmap stdlib functions

Pattern for read operations:
```rust
fn hashmap_get(args: &[Value]) -> Result<Value, RuntimeError> {
    let map = extract_hashmap_ref(&args[0], span)?;
    let key = args[1].as_string()?;
    Ok(map.inner().get(key).cloned().unwrap_or(Value::Null))
}
```

Pattern for mutation operations:
```rust
fn hashmap_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut map_val = args[0].clone();
    if let Value::HashMap(ref mut m) = map_val {
        m.inner_mut().insert(key.to_string(), value.clone());
    }
    Ok(map_val) // return modified map
}
```

---

## Acceptance Criteria

- [ ] `extract_hashmap` updated (no `Arc<Mutex>` return)
- [ ] All hashmap functions use `.inner()` / `.inner_mut()` API
- [ ] Mutation functions return the modified `Value::HashMap`
- [ ] No `.lock().unwrap()` in `hashmap.rs`
- [ ] `cargo nextest run -p atlas-runtime -- stdlib::collections::hashmap` 100% pass
