use atlas_runtime::Value;
use std::sync::Arc;
use std::sync::Mutex;

#[test]
fn test_value_is_send() {
    // This test MUST compile - proves Value is Send
    fn assert_send<T: Send>() {}
    assert_send::<Value>();
}

#[test]
fn test_value_can_be_sent_to_thread() {
    use std::thread;

    let value = Value::String(Arc::new("test".to_string()));

    // This MUST work now (Rc would fail here)
    let handle = thread::spawn(move || {
        // Use value in different thread
        value
    });

    let result = handle.join().unwrap();
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_array_can_be_sent_to_thread() {
    use std::thread;

    let arr = Value::Array(Arc::new(Mutex::new(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ])));

    let handle = thread::spawn(move || arr);
    let result = handle.join().unwrap();
    assert!(matches!(result, Value::Array(_)));
}
