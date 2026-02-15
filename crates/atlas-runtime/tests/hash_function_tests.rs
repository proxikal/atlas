//! Hash function tests for Atlas collections

mod common;
use common::*;

#[test]
fn test_hash_number_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, 42, "forty-two");
        let result = hashMapGet(hm, 42);
        unwrap(result)
    "#;
    assert_eval_string(code, "forty-two");
}

#[test]
fn test_hash_string_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "hello", "world");
        let result = hashMapGet(hm, "hello");
        unwrap(result)
    "#;
    assert_eval_string(code, "world");
}

#[test]
fn test_hash_bool_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, true, "yes");
        let result = hashMapGet(hm, true);
        unwrap(result)
    "#;
    assert_eval_string(code, "yes");
}

#[test]
fn test_hash_null_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, null, "null-value");
        let result = hashMapGet(hm, null);
        unwrap(result)
    "#;
    assert_eval_string(code, "null-value");
}

#[test]
fn test_cannot_hash_array() {
    let code = r#"
        let hm = hashMapNew();
        let arr = [1, 2, 3];
        hashMapPut(hm, arr, "value");
    "#;
    assert_error_code(code, "AT0140");
}

#[test]
fn test_mixed_key_types() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, 42, "number");
        hashMapPut(hm, "key", "string");
        hashMapPut(hm, true, "bool");
        hashMapPut(hm, null, "null");
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn test_hashhm_new() {
    let code = r#"
        let hm = hashMapNew();
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashhm_put_get() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "key", "value");
        let result = hashMapGet(hm, "key");
        unwrap(result)
    "#;
    assert_eval_string(code, "value");
}

#[test]
fn test_hashhm_get_nonexistent() {
    let code = r#"
        let hm = hashMapNew();
        let result = hashMapGet(hm, "nonexistent");
        is_none(result)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_remove() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "key", "value");
        let removed = hashMapRemove(hm, "key");
        is_some(removed)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_has() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "key", "value");
        hashMapHas(hm, "key")
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_size() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        hashMapPut(hm, "c", 3);
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_hashhm_is_empty() {
    let code = r#"
        let hm = hashMapNew();
        let empty1 = hashMapIsEmpty(hm);
        hashMapPut(hm, "key", "value");
        let empty2 = hashMapIsEmpty(hm);
        empty1 && !empty2
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_clear() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        hashMapClear(hm);
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashhm_keys() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        let keys = hashMapKeys(hm);
        len(keys)
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_hashhm_values() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        let values = hashMapValues(hm);
        len(values)
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_hashhm_entries() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        let entries = hashMapEntries(hm);
        len(entries)
    "#;
    assert_eval_number(code, 2.0);
}
