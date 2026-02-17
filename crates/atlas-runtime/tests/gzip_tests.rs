//! Gzip compression tests
//!
//! Comprehensive tests for gzip compression and decompression

use atlas_runtime::span::Span;
use atlas_runtime::stdlib::compression::gzip;
use atlas_runtime::value::Value;

// ============================================================================
// Helper Functions
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn bytes_to_atlas_array(bytes: &[u8]) -> Value {
    let values: Vec<Value> = bytes.iter().map(|&b| Value::Number(b as f64)).collect();
    Value::array(values)
}

fn atlas_array_to_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            arr_guard
                .iter()
                .map(|v| match v {
                    Value::Number(n) => *n as u8,
                    _ => panic!("Expected number in array"),
                })
                .collect()
        }
        _ => panic!("Expected array"),
    }
}

fn extract_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        _ => panic!("Expected bool value"),
    }
}

fn extract_number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => *n,
        _ => panic!("Expected number value"),
    }
}

// ============================================================================
// Compression Tests
// ============================================================================

#[test]
fn test_compress_byte_array() {
    let data = b"Hello, World!";
    let atlas_data = bytes_to_atlas_array(data);

    let result = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Verify gzip magic header
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);

    // Compressed should be different from original
    assert_ne!(compressed_bytes, data);
}

#[test]
fn test_compress_string() {
    let text = "Hello, World!";
    let data = Value::string(text.to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Verify gzip magic header
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);
}

#[test]
fn test_compression_level_0() {
    let data = Value::string("Test data for compression".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(0.0)), span());
    assert!(result.is_ok());

    // Level 0 should still produce valid gzip
    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);
}

#[test]
fn test_compression_level_6() {
    let data = Value::string("Test data for compression".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());
}

#[test]
fn test_compression_level_9() {
    let data = Value::string("Test data for compression".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span());
    assert!(result.is_ok());

    // Level 9 should produce smaller output than level 0
    let result0 = gzip::gzip_compress(&data, Some(&Value::Number(0.0)), span()).unwrap();

    let compressed9 = atlas_array_to_bytes(&result.unwrap());
    let compressed0 = atlas_array_to_bytes(&result0);

    assert!(compressed9.len() <= compressed0.len());
}

#[test]
fn test_compress_large_data() {
    // Create large repeating data (compresses well)
    let large_text = "A".repeat(10000);
    let data = Value::string(large_text);

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Large repeating data should compress significantly
    assert!(compressed_bytes.len() < 10000);
}

#[test]
fn test_compress_empty_data() {
    let data = Value::string("".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_ok());

    let compressed = result.unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Even empty data produces gzip header
    assert!(compressed_bytes.len() > 10);
    assert_eq!(compressed_bytes[0], 0x1f);
    assert_eq!(compressed_bytes[1], 0x8b);
}

#[test]
fn test_compress_invalid_level() {
    let data = Value::string("test".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::Number(10.0)), span());
    assert!(result.is_err());
}

// ============================================================================
// Decompression Tests
// ============================================================================

#[test]
fn test_decompress_to_bytes() {
    let original = b"Hello, World!";
    let atlas_data = bytes_to_atlas_array(original);

    // Compress
    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress(&compressed, span());
    assert!(result.is_ok());

    let decompressed = result.unwrap();
    let decompressed_bytes = atlas_array_to_bytes(&decompressed);

    assert_eq!(decompressed_bytes, original);
}

#[test]
fn test_decompress_to_string() {
    let original = "Hello, World!";
    let data = Value::string(original.to_string());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress to string
    let result = gzip::gzip_decompress_string(&compressed, span());
    assert!(result.is_ok());

    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), original),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_decompress_corrupt_data() {
    let bad_data = vec![Value::Number(0.0), Value::Number(1.0), Value::Number(2.0)];
    let atlas_data = Value::array(bad_data);

    let result = gzip::gzip_decompress(&atlas_data, span());
    assert!(result.is_err());
}

#[test]
fn test_decompress_invalid_format() {
    // Create data without gzip magic header
    let bad_data: Vec<Value> = (0..20).map(|i| Value::Number(i as f64)).collect();
    let atlas_data = Value::array(bad_data);

    let result = gzip::gzip_decompress(&atlas_data, span());
    assert!(result.is_err());
}

#[test]
fn test_decompress_large_file() {
    let large_text = "B".repeat(50000);
    let data = Value::string(large_text.clone());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span());
    assert!(result.is_ok());

    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), &large_text),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_decompress_empty_compressed_data() {
    let empty_data = Value::string("".to_string());

    // Compress empty data
    let compressed = gzip::gzip_compress(&empty_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span());
    assert!(result.is_ok());

    match result.unwrap() {
        Value::String(s) => assert_eq!(s.as_ref(), ""),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_decompress_non_utf8() {
    // Create binary data that isn't valid UTF-8
    let binary: Vec<u8> = vec![0xff, 0xfe, 0xfd, 0xfc];
    let atlas_data = bytes_to_atlas_array(&binary);

    // Compress binary data
    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress to bytes should work
    let result_bytes = gzip::gzip_decompress(&compressed, span());
    assert!(result_bytes.is_ok());

    // Decompress to string should fail
    let result_string = gzip::gzip_decompress_string(&compressed, span());
    assert!(result_string.is_err());
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[test]
fn test_round_trip_bytes() {
    let original = b"The quick brown fox jumps over the lazy dog";
    let atlas_data = bytes_to_atlas_array(original);

    // Compress
    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let decompressed = gzip::gzip_decompress(&compressed, span()).unwrap();
    let decompressed_bytes = atlas_array_to_bytes(&decompressed);

    assert_eq!(decompressed_bytes, original);
}

#[test]
fn test_round_trip_string() {
    let original = "The quick brown fox jumps over the lazy dog";
    let data = Value::string(original.to_string());

    // Compress
    let compressed = gzip::gzip_compress(&data, None, span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match result {
        Value::String(s) => assert_eq!(s.as_ref(), original),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_round_trip_large_data() {
    let original = "Lorem ipsum dolor sit amet. ".repeat(1000);
    let data = Value::string(original.clone());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match result {
        Value::String(s) => assert_eq!(s.as_ref(), &original),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_round_trip_different_levels() {
    let original = "Test data for different compression levels";

    for level in 0..=9 {
        let data = Value::string(original.to_string());

        // Compress with specific level
        let compressed =
            gzip::gzip_compress(&data, Some(&Value::Number(level as f64)), span()).unwrap();

        // Decompress
        let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

        match result {
            Value::String(s) => assert_eq!(s.as_ref(), original, "Failed at level {}", level),
            _ => panic!("Expected string at level {}", level),
        }
    }
}

#[test]
fn test_round_trip_utf8_preservation() {
    let original = "Hello 世界! 🎉 Ça marche!";
    let data = Value::string(original.to_string());

    // Compress
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    // Decompress
    let result = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match result {
        Value::String(s) => assert_eq!(s.as_ref(), original),
        _ => panic!("Expected string"),
    }
}

// ============================================================================
// Utility Tests
// ============================================================================

#[test]
fn test_is_gzip_true() {
    let data = Value::string("test".to_string());
    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();

    let result = gzip::gzip_is_gzip(&compressed, span()).unwrap();
    assert!(extract_bool(&result));
}

#[test]
fn test_is_gzip_false() {
    let data: Vec<Value> = vec![Value::Number(0.0), Value::Number(1.0)];
    let atlas_data = Value::array(data);

    let result = gzip::gzip_is_gzip(&atlas_data, span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_compression_ratio() {
    let original_size = Value::Number(1000.0);
    let compressed_size = Value::Number(500.0);

    let result = gzip::gzip_compression_ratio(&original_size, &compressed_size, span()).unwrap();
    let ratio = extract_number(&result);

    assert_eq!(ratio, 2.0); // 1000 / 500 = 2.0
}

#[test]
fn test_compression_ratio_no_compression() {
    let original_size = Value::Number(1000.0);
    let compressed_size = Value::Number(1000.0);

    let result = gzip::gzip_compression_ratio(&original_size, &compressed_size, span()).unwrap();
    let ratio = extract_number(&result);

    assert_eq!(ratio, 1.0);
}

#[test]
fn test_compression_ratio_expansion() {
    let original_size = Value::Number(100.0);
    let compressed_size = Value::Number(150.0);

    let result = gzip::gzip_compression_ratio(&original_size, &compressed_size, span()).unwrap();
    let ratio = extract_number(&result);

    assert!((ratio - 0.666).abs() < 0.01);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_compress_decompress_json_data() {
    let json_text = r#"{"name":"Alice","age":30,"active":true}"#;
    let data = Value::string(json_text.to_string());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();
    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), json_text),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_compress_decompress_code() {
    let code = r#"
fn main() {
    let x = 42;
    println!("Hello, world!");
    return x;
}
"#;
    let data = Value::string(code.to_string());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span()).unwrap();
    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), code),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_real_world_log_data() {
    let log = "[INFO] 2024-01-01 12:00:00 - Application started\n".repeat(100);
    let data = Value::string(log.clone());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span()).unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Repeating log data should compress well
    assert!(compressed_bytes.len() < log.len() / 2);

    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();

    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), &log),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_memory_efficiency_large_repeated_data() {
    // Create data with lots of repetition (should compress extremely well)
    let repeated = "AAAABBBBCCCCDDDD".repeat(1000);
    let data = Value::string(repeated.clone());

    let compressed = gzip::gzip_compress(&data, Some(&Value::Number(9.0)), span()).unwrap();
    let compressed_bytes = atlas_array_to_bytes(&compressed);

    // Should achieve >10x compression
    let ratio = repeated.len() as f64 / compressed_bytes.len() as f64;
    assert!(ratio > 10.0);

    // Verify decompression works
    let decompressed = gzip::gzip_decompress_string(&compressed, span()).unwrap();
    match decompressed {
        Value::String(s) => assert_eq!(s.as_ref(), &repeated),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_binary_data_round_trip() {
    // Test binary data (all byte values)
    let binary: Vec<u8> = (0..=255).collect();
    let atlas_data = bytes_to_atlas_array(&binary);

    let compressed = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span()).unwrap();
    let decompressed = gzip::gzip_decompress(&compressed, span()).unwrap();
    let decompressed_bytes = atlas_array_to_bytes(&decompressed);

    assert_eq!(decompressed_bytes, binary);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_compress_wrong_type() {
    let data = Value::Number(42.0);

    let result = gzip::gzip_compress(&data, Some(&Value::Number(6.0)), span());
    assert!(result.is_err());
}

#[test]
fn test_decompress_wrong_type() {
    let data = Value::string("not an array".to_string());

    let result = gzip::gzip_decompress(&data, span());
    assert!(result.is_err());
}

#[test]
fn test_compress_level_wrong_type() {
    let data = Value::string("test".to_string());

    let result = gzip::gzip_compress(&data, Some(&Value::string("six".to_string())), span());
    assert!(result.is_err());
}

#[test]
fn test_byte_array_out_of_range() {
    let data: Vec<Value> = vec![Value::Number(256.0)]; // Out of byte range
    let atlas_data = Value::array(data);

    let result = gzip::gzip_compress(&atlas_data, Some(&Value::Number(6.0)), span());
    assert!(result.is_err());
}

#[test]
fn test_default_compression_level() {
    let data = Value::string("test data".to_string());

    // No level specified - should use default (6)
    let result = gzip::gzip_compress(&data, None, span());
    assert!(result.is_ok());
}
