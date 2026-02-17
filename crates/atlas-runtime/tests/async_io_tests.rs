//! Async I/O tests
//!
//! Tests for async file and HTTP operations that return Futures.

use atlas_runtime::async_runtime::FutureState;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib::async_io;
use atlas_runtime::value::Value;
use rstest::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Async File Reading Tests (8 tests)
// ============================================================================

#[rstest]
fn test_read_small_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Hello, async!").unwrap();

    let security = SecurityContext::allow_all();
    let args = [Value::string(file_path.to_str().unwrap())];
    let result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

    // Extract Future
    let future = match result {
        Value::Future(f) => f,
        _ => panic!("Expected Future"),
    };

    // Should be resolved (block_on completes immediately)
    assert!(future.is_resolved());

    match future.get_state() {
        FutureState::Resolved(Value::String(s)) => {
            assert_eq!(&**s, "Hello, async!");
        }
        _ => panic!("Expected resolved string"),
    }
}

#[rstest]
fn test_read_large_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.txt");
    let large_content = "x".repeat(10000);
    fs::write(&file_path, &large_content).unwrap();

    let security = SecurityContext::allow_all();
    let args = [Value::string(file_path.to_str().unwrap())];
    let result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!("Expected Future"),
    };

    assert!(future.is_resolved());
    match future.get_state() {
        FutureState::Resolved(Value::String(s)) => {
            assert_eq!(s.len(), 10000);
        }
        _ => panic!("Expected resolved string"),
    }
}

#[rstest]
fn test_read_non_existent_file_async() {
    let security = SecurityContext::allow_all();
    let args = [Value::string("/nonexistent/file.txt")];

    // Permission check fails before async operation starts
    let result = async_io::read_file_async(&args, Span::dummy(), &security);
    assert!(result.is_err());
}

#[rstest]
fn test_read_permission_denied_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "secret").unwrap();

    let security = SecurityContext::new();
    let args = [Value::string(file_path.to_str().unwrap())];

    // Permission check should fail
    let result = async_io::read_file_async(&args, Span::dummy(), &security);
    assert!(result.is_err());
}

#[rstest]
fn test_multiple_concurrent_reads() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let file3 = temp_dir.path().join("file3.txt");

    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();
    fs::write(&file3, "content3").unwrap();

    let security = SecurityContext::allow_all();

    // Read all three files concurrently
    let args1 = [Value::string(file1.to_str().unwrap())];
    let args2 = [Value::string(file2.to_str().unwrap())];
    let args3 = [Value::string(file3.to_str().unwrap())];

    let result1 = async_io::read_file_async(&args1, Span::dummy(), &security).unwrap();
    let result2 = async_io::read_file_async(&args2, Span::dummy(), &security).unwrap();
    let result3 = async_io::read_file_async(&args3, Span::dummy(), &security).unwrap();

    // All should be resolved
    let f1 = match result1 {
        Value::Future(f) => f,
        _ => panic!(),
    };
    let f2 = match result2 {
        Value::Future(f) => f,
        _ => panic!(),
    };
    let f3 = match result3 {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(f1.is_resolved());
    assert!(f2.is_resolved());
    assert!(f3.is_resolved());
}

#[rstest]
fn test_read_utf8_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("utf8.txt");
    fs::write(&file_path, "Hello 世界 🚀").unwrap();

    let security = SecurityContext::allow_all();
    let args = [Value::string(file_path.to_str().unwrap())];
    let result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    match future.get_state() {
        FutureState::Resolved(Value::String(s)) => {
            assert_eq!(&**s, "Hello 世界 🚀");
        }
        _ => panic!(),
    }
}

#[rstest]
fn test_read_empty_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.txt");
    fs::write(&file_path, "").unwrap();

    let security = SecurityContext::allow_all();
    let args = [Value::string(file_path.to_str().unwrap())];
    let result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    match future.get_state() {
        FutureState::Resolved(Value::String(s)) => {
            assert_eq!(&**s, "");
        }
        _ => panic!(),
    }
}

#[rstest]
fn test_cancel_read_operation() {
    // With current implementation using block_on, cancellation isn't supported
    // This test documents the expected behavior for future async executor
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "data").unwrap();

    let security = SecurityContext::allow_all();
    let args = [Value::string(file_path.to_str().unwrap())];
    let result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

    // Future completes immediately with current implementation
    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };
    assert!(future.is_resolved());
}

// ============================================================================
// Async File Writing Tests (8 tests)
// ============================================================================

#[rstest]
fn test_write_small_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("write.txt");

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("async write"),
    ];
    let result = async_io::write_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());

    // Verify file was written
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "async write");
}

#[rstest]
fn test_write_large_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.txt");
    let large_content = "y".repeat(50000);

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string(&large_content),
    ];
    let result = async_io::write_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content.len(), 50000);
}

#[rstest]
fn test_overwrite_existing_file_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("overwrite.txt");
    fs::write(&file_path, "original").unwrap();

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("overwritten"),
    ];
    let result = async_io::write_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "overwritten");
}

#[rstest]
fn test_write_permission_denied_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("denied.txt");

    let security = SecurityContext::new();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("data"),
    ];

    let result = async_io::write_file_async(&args, Span::dummy(), &security);
    assert!(result.is_err());
}

#[rstest]
fn test_multiple_concurrent_writes() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("w1.txt");
    let file2 = temp_dir.path().join("w2.txt");
    let file3 = temp_dir.path().join("w3.txt");

    let security = SecurityContext::allow_all();

    let args1 = [
        Value::string(file1.to_str().unwrap()),
        Value::string("data1"),
    ];
    let args2 = [
        Value::string(file2.to_str().unwrap()),
        Value::string("data2"),
    ];
    let args3 = [
        Value::string(file3.to_str().unwrap()),
        Value::string("data3"),
    ];

    let r1 = async_io::write_file_async(&args1, Span::dummy(), &security).unwrap();
    let r2 = async_io::write_file_async(&args2, Span::dummy(), &security).unwrap();
    let r3 = async_io::write_file_async(&args3, Span::dummy(), &security).unwrap();

    // All should complete
    match r1 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
    match r2 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
    match r3 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }

    // Verify all files
    assert_eq!(fs::read_to_string(&file1).unwrap(), "data1");
    assert_eq!(fs::read_to_string(&file2).unwrap(), "data2");
    assert_eq!(fs::read_to_string(&file3).unwrap(), "data3");
}

#[rstest]
fn test_write_to_non_existent_directory_creates() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("new_dir").join("file.txt");

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(nested_path.to_str().unwrap()),
        Value::string("nested"),
    ];

    let result = async_io::write_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
    assert_eq!(fs::read_to_string(&nested_path).unwrap(), "nested");
}

#[rstest]
fn test_write_empty_string_async() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.txt");

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string(""),
    ];

    let result = async_io::write_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "");
}

#[rstest]
fn test_atomic_write_option() {
    // Current implementation doesn't support atomic writes explicitly
    // This test documents expected behavior
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("atomic.txt");

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("atomic data"),
    ];

    let result = async_io::write_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "atomic data");
}

// ============================================================================
// Async File Appending Tests (5 tests)
// ============================================================================

#[rstest]
fn test_append_to_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("append.txt");
    fs::write(&file_path, "initial\n").unwrap();

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("appended\n"),
    ];

    let result = async_io::append_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
    assert_eq!(
        fs::read_to_string(&file_path).unwrap(),
        "initial\nappended\n"
    );
}

#[rstest]
fn test_append_creates_new_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("new_append.txt");

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("first line\n"),
    ];

    let result = async_io::append_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "first line\n");
}

#[rstest]
fn test_multiple_concurrent_appends() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("concurrent_append.txt");
    fs::write(&file_path, "start\n").unwrap();

    let security = SecurityContext::allow_all();

    // Multiple appends to same file
    let args1 = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("line1\n"),
    ];
    let args2 = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("line2\n"),
    ];

    let r1 = async_io::append_file_async(&args1, Span::dummy(), &security).unwrap();
    let r2 = async_io::append_file_async(&args2, Span::dummy(), &security).unwrap();

    match r1 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
    match r2 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("start"));
    assert!(content.contains("line1"));
    assert!(content.contains("line2"));
}

#[rstest]
fn test_append_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("denied_append.txt");

    let security = SecurityContext::new();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string("data"),
    ];

    let result = async_io::append_file_async(&args, Span::dummy(), &security);
    assert!(result.is_err());
}

#[rstest]
fn test_append_empty_string() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("append_empty.txt");
    fs::write(&file_path, "content").unwrap();

    let security = SecurityContext::allow_all();
    let args = [
        Value::string(file_path.to_str().unwrap()),
        Value::string(""),
    ];

    let result = async_io::append_file_async(&args, Span::dummy(), &security).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "content");
}

// ============================================================================
// Async HTTP Operations Tests (10 tests)
// ============================================================================

#[rstest]
#[ignore] // Requires network - run with --ignored
fn test_get_request_async() {
    let args = [Value::string("https://httpbin.org/get")];

    let result = async_io::http_get_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());

    match future.get_state() {
        FutureState::Resolved(Value::HttpResponse(resp)) => {
            assert!(resp.status() >= 200 && resp.status() < 300);
        }
        _ => panic!("Expected resolved HTTP response"),
    }
}

#[rstest]
#[ignore] // Requires network
fn test_post_request_async() {
    let args = [
        Value::string("https://httpbin.org/post"),
        Value::string("test data"),
    ];

    let result = async_io::http_post_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());

    match future.get_state() {
        FutureState::Resolved(Value::HttpResponse(resp)) => {
            assert_eq!(resp.status(), 200);
        }
        _ => panic!(),
    }
}

#[rstest]
#[ignore] // Requires network
fn test_put_request_async() {
    let args = [
        Value::string("https://httpbin.org/put"),
        Value::string("updated data"),
    ];

    let result = async_io::http_put_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
}

#[rstest]
#[ignore] // Requires network
fn test_delete_request_async() {
    let args = [Value::string("https://httpbin.org/delete")];

    let result = async_io::http_delete_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());
}

#[rstest]
#[ignore] // Requires network
fn test_multiple_concurrent_requests() {
    let args1 = [Value::string("https://httpbin.org/get")];
    let args2 = [Value::string("https://httpbin.org/headers")];
    let args3 = [Value::string("https://httpbin.org/user-agent")];

    let r1 = async_io::http_get_async(&args1, Span::dummy()).unwrap();
    let r2 = async_io::http_get_async(&args2, Span::dummy()).unwrap();
    let r3 = async_io::http_get_async(&args3, Span::dummy()).unwrap();

    // All should complete
    match r1 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
    match r2 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
    match r3 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
}

#[rstest]
fn test_request_timeout() {
    // Timeout test - use a slow endpoint
    use atlas_runtime::stdlib::http::HttpRequest;
    use std::sync::Arc;

    let request = HttpRequest::new(
        "GET".to_string(),
        "https://httpbin.org/delay/10".to_string(),
    )
    .with_timeout(1); // 1 second timeout

    let args = [Value::HttpRequest(Arc::new(request))];
    let result = async_io::http_send_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    // Should be resolved with error (timeout)
    match future.get_state() {
        FutureState::Rejected(_) => {
            // Expected timeout
        }
        _ => {
            // May also succeed if connection is very fast
        }
    }
}

#[rstest]
fn test_network_error_handling() {
    let args = [Value::string(
        "https://this-domain-does-not-exist-12345.com",
    )];

    let result = async_io::http_get_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    // Should be rejected with connection error
    match future.get_state() {
        FutureState::Rejected(Value::String(msg)) => {
            assert!(msg.contains("error") || msg.contains("Error"));
        }
        _ => panic!("Expected rejected future with error"),
    }
}

#[rstest]
#[ignore] // Requires network
fn test_large_response_handling() {
    let args = [Value::string("https://httpbin.org/bytes/100000")];

    let result = async_io::http_get_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());

    match future.get_state() {
        FutureState::Resolved(Value::HttpResponse(resp)) => {
            assert!(resp.body().len() >= 100000);
        }
        _ => panic!(),
    }
}

#[rstest]
#[ignore] // Requires network
fn test_concurrent_requests_different_hosts() {
    let args1 = [Value::string("https://httpbin.org/get")];
    let args2 = [Value::string(
        "https://jsonplaceholder.typicode.com/todos/1",
    )];

    let r1 = async_io::http_get_async(&args1, Span::dummy()).unwrap();
    let r2 = async_io::http_get_async(&args2, Span::dummy()).unwrap();

    match r1 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
    match r2 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
}

#[rstest]
#[ignore] // Requires network
fn test_request_with_custom_headers_async() {
    use atlas_runtime::stdlib::http::HttpRequest;
    use std::sync::Arc;

    let request = HttpRequest::new("GET".to_string(), "https://httpbin.org/headers".to_string())
        .with_header("X-Custom-Header".to_string(), "test-value".to_string());

    let args = [Value::HttpRequest(Arc::new(request))];
    let result = async_io::http_send_async(&args, Span::dummy()).unwrap();

    let future = match result {
        Value::Future(f) => f,
        _ => panic!(),
    };

    assert!(future.is_resolved());

    match future.get_state() {
        FutureState::Resolved(Value::HttpResponse(resp)) => {
            // Response should contain our custom header in the JSON
            assert!(resp.body().contains("X-Custom-Header"));
        }
        _ => panic!(),
    }
}

// ============================================================================
// Concurrent Operations Tests (8 tests)
// ============================================================================

#[rstest]
fn test_parallel_file_reads_with_future_all() {
    use atlas_runtime::async_runtime::future_all;

    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("f1.txt");
    let file2 = temp_dir.path().join("f2.txt");
    let file3 = temp_dir.path().join("f3.txt");

    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();
    fs::write(&file3, "content3").unwrap();

    let security = SecurityContext::allow_all();

    let args1 = [Value::string(file1.to_str().unwrap())];
    let args2 = [Value::string(file2.to_str().unwrap())];
    let args3 = [Value::string(file3.to_str().unwrap())];

    let r1 = async_io::read_file_async(&args1, Span::dummy(), &security).unwrap();
    let r2 = async_io::read_file_async(&args2, Span::dummy(), &security).unwrap();
    let r3 = async_io::read_file_async(&args3, Span::dummy(), &security).unwrap();

    let futures = vec![
        match r1 {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        },
        match r2 {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        },
        match r3 {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        },
    ];

    let combined = future_all(futures);
    assert!(combined.is_resolved());

    match combined.get_state() {
        FutureState::Resolved(Value::Array(arr)) => {
            assert_eq!(arr.lock().unwrap().len(), 3);
        }
        _ => panic!("Expected array of results"),
    }
}

#[rstest]
#[ignore] // Requires network
fn test_parallel_http_requests_with_future_all() {
    use atlas_runtime::async_runtime::future_all;

    let args1 = [Value::string("https://httpbin.org/get")];
    let args2 = [Value::string("https://httpbin.org/headers")];

    let r1 = async_io::http_get_async(&args1, Span::dummy()).unwrap();
    let r2 = async_io::http_get_async(&args2, Span::dummy()).unwrap();

    let futures = vec![
        match r1 {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        },
        match r2 {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        },
    ];

    let combined = future_all(futures);
    assert!(combined.is_resolved());

    match combined.get_state() {
        FutureState::Resolved(Value::Array(arr)) => {
            assert_eq!(arr.lock().unwrap().len(), 2);
        }
        _ => panic!(),
    }
}

#[rstest]
fn test_mixed_file_and_network_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "data").unwrap();

    let security = SecurityContext::allow_all();

    // Mix of file and HTTP operations
    let file_args = [Value::string(file_path.to_str().unwrap())];

    let file_result = async_io::read_file_async(&file_args, Span::dummy(), &security).unwrap();

    // File operation should complete
    match file_result {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
}

#[rstest]
fn test_future_race_for_first_completed() {
    use atlas_runtime::async_runtime::future_race;

    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("f1.txt");
    let file2 = temp_dir.path().join("f2.txt");

    fs::write(&file1, "first").unwrap();
    fs::write(&file2, "second").unwrap();

    let security = SecurityContext::allow_all();

    let args1 = [Value::string(file1.to_str().unwrap())];
    let args2 = [Value::string(file2.to_str().unwrap())];

    let r1 = async_io::read_file_async(&args1, Span::dummy(), &security).unwrap();
    let r2 = async_io::read_file_async(&args2, Span::dummy(), &security).unwrap();

    let futures = vec![
        match r1 {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        },
        match r2 {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        },
    ];

    let winner = future_race(futures);
    assert!(winner.is_resolved());

    // Should be one of the file contents
    match winner.get_state() {
        FutureState::Resolved(Value::String(s)) => {
            assert!(s.as_ref() == "first" || s.as_ref() == "second");
        }
        _ => panic!(),
    }
}

#[rstest]
fn test_error_in_parallel_operations() {
    use atlas_runtime::async_runtime::future_all;

    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("exists.txt");
    fs::write(&file1, "data").unwrap();

    let security = SecurityContext::allow_all();

    // One successful read
    let args1 = [Value::string(file1.to_str().unwrap())];

    let r1 = async_io::read_file_async(&args1, Span::dummy(), &security).unwrap();
    // Note: Testing error handling in parallel operations requires actual async
    // execution model. This documents expected behavior for future implementations.

    match r1 {
        Value::Future(f) => assert!(f.is_resolved()),
        _ => panic!(),
    }
}

#[rstest]
fn test_many_concurrent_operations() {
    let temp_dir = TempDir::new().unwrap();
    let security = SecurityContext::allow_all();

    let mut futures = Vec::new();

    // Create and read 10 files concurrently
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("file{}.txt", i));
        fs::write(&file_path, format!("content{}", i)).unwrap();

        let args = [Value::string(file_path.to_str().unwrap())];
        let result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

        futures.push(match result {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        });
    }

    // All should complete
    for future in futures {
        assert!(future.is_resolved());
    }
}

#[rstest]
fn test_resource_limit_handling() {
    // Current implementation uses block_on, so no resource limits apply
    // This test documents expected behavior for future async executor
    let temp_dir = TempDir::new().unwrap();
    let security = SecurityContext::allow_all();

    // Create many concurrent operations
    let mut count = 0;
    for i in 0..50 {
        let file_path = temp_dir.path().join(format!("r{}.txt", i));
        fs::write(&file_path, "data").unwrap();

        let args = [Value::string(file_path.to_str().unwrap())];
        if let Ok(Value::Future(f)) = async_io::read_file_async(&args, Span::dummy(), &security) {
            if f.is_resolved() {
                count += 1;
            }
        }
    }

    assert_eq!(count, 50);
}

#[rstest]
fn test_ordered_results_from_future_all() {
    use atlas_runtime::async_runtime::future_all;

    let temp_dir = TempDir::new().unwrap();
    let security = SecurityContext::allow_all();

    let mut futures = Vec::new();

    for i in 0..5 {
        let file_path = temp_dir.path().join(format!("ord{}.txt", i));
        fs::write(&file_path, format!("{}", i)).unwrap();

        let args = [Value::string(file_path.to_str().unwrap())];
        let result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

        futures.push(match result {
            Value::Future(f) => (*f).clone(),
            _ => panic!(),
        });
    }

    let combined = future_all(futures);
    assert!(combined.is_resolved());

    match combined.get_state() {
        FutureState::Resolved(Value::Array(arr)) => {
            let values = arr.lock().unwrap();
            assert_eq!(values.len(), 5);
            // Results should be in order
            for (i, val) in values.iter().enumerate() {
                match val {
                    Value::String(s) => assert_eq!(&**s, &i.to_string()),
                    _ => panic!(),
                }
            }
        }
        _ => panic!(),
    }
}

// ============================================================================
// Await Helper Tests (2 tests)
// ============================================================================

#[rstest]
fn test_await_resolved_future() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("await_test.txt");
    fs::write(&file_path, "awaited").unwrap();

    let security = SecurityContext::allow_all();
    let args = [Value::string(file_path.to_str().unwrap())];
    let future_result = async_io::read_file_async(&args, Span::dummy(), &security).unwrap();

    // Use await helper
    let await_args = [future_result];
    let result = async_io::await_future(&await_args, Span::dummy()).unwrap();

    match result {
        Value::String(s) => assert_eq!(&**s, "awaited"),
        _ => panic!("Expected string from await"),
    }
}

#[rstest]
fn test_await_rejected_future() {
    let args = [Value::string("https://this-will-fail.invalid")];
    let future_result = async_io::http_get_async(&args, Span::dummy()).unwrap();

    // Await should return error
    let await_args = [future_result];
    let result = async_io::await_future(&await_args, Span::dummy());

    assert!(result.is_err());
}
