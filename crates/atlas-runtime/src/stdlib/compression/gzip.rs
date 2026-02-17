//! Gzip compression and decompression
//!
//! Provides gzip utilities using the flate2 library for data and file compression.

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::Read;

// ============================================================================
// Constants
// ============================================================================

/// Default compression level (6 = good balance of speed vs ratio)
const DEFAULT_COMPRESSION_LEVEL: u32 = 6;

/// Gzip magic number (first two bytes)
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

// ============================================================================
// Compression Functions
// ============================================================================

/// Compress bytes using gzip
///
/// Takes a byte array and compression level (0-9), returns compressed bytes.
/// Level 0 = no compression (store only), 9 = maximum compression.
pub fn compress_bytes(data: &[u8], level: u32, span: Span) -> Result<Vec<u8>, RuntimeError> {
    // Validate compression level
    if level > 9 {
        return Err(RuntimeError::IoError {
            message: format!("Invalid compression level {}: must be 0-9", level),
            span,
        });
    }

    let compression = Compression::new(level);
    let mut encoder = GzEncoder::new(data, compression);
    let mut compressed = Vec::new();

    encoder
        .read_to_end(&mut compressed)
        .map_err(|e| RuntimeError::IoError {
            message: format!("Gzip compression failed: {}", e),
            span,
        })?;

    Ok(compressed)
}

/// Compress string using gzip (convenience function)
///
/// Takes a string and compression level, returns compressed bytes.
pub fn compress_string(text: &str, level: u32, span: Span) -> Result<Vec<u8>, RuntimeError> {
    compress_bytes(text.as_bytes(), level, span)
}

// ============================================================================
// Decompression Functions
// ============================================================================

/// Decompress gzip bytes
///
/// Takes compressed bytes, returns original uncompressed data.
pub fn decompress_bytes(compressed: &[u8], span: Span) -> Result<Vec<u8>, RuntimeError> {
    // Validate gzip magic header
    if compressed.len() < 2 || compressed[0] != GZIP_MAGIC[0] || compressed[1] != GZIP_MAGIC[1] {
        return Err(RuntimeError::IoError {
            message: "Invalid gzip format: missing magic header".to_string(),
            span,
        });
    }

    let mut decoder = GzDecoder::new(compressed);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| RuntimeError::IoError {
            message: format!("Gzip decompression failed: {}", e),
            span,
        })?;

    Ok(decompressed)
}

/// Decompress gzip bytes to string
///
/// Takes compressed bytes, returns UTF-8 string.
pub fn decompress_string(compressed: &[u8], span: Span) -> Result<String, RuntimeError> {
    let bytes = decompress_bytes(compressed, span)?;

    String::from_utf8(bytes).map_err(|e| RuntimeError::IoError {
        message: format!("Decompressed data is not valid UTF-8: {}", e),
        span,
    })
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Check if bytes are gzip-compressed (by magic header)
pub fn is_gzip(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == GZIP_MAGIC[0] && data[1] == GZIP_MAGIC[1]
}

/// Calculate compression ratio (original size / compressed size)
pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
    if compressed_size == 0 {
        return 0.0;
    }
    original_size as f64 / compressed_size as f64
}

// ============================================================================
// Atlas Stdlib API Functions
// ============================================================================

/// gzipCompress(data: string | array<number>, level?: number) -> array<number>
///
/// Compress data using gzip. Level 0-9 (default 6).
pub fn gzip_compress(
    data: &Value,
    level_opt: Option<&Value>,
    span: Span,
) -> Result<Value, RuntimeError> {
    // Extract compression level (default 6)
    let level = if let Some(level_val) = level_opt {
        match level_val {
            Value::Number(n) => {
                let l = *n as u32;
                if l > 9 {
                    return Err(RuntimeError::IoError {
                        message: format!("Compression level must be 0-9, got {}", l),
                        span,
                    });
                }
                l
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Compression level must be a number".to_string(),
                    span,
                });
            }
        }
    } else {
        DEFAULT_COMPRESSION_LEVEL
    };

    // Get bytes to compress
    let bytes = match data {
        Value::String(s) => s.as_ref().as_bytes().to_vec(),
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            let mut bytes = Vec::with_capacity(arr_guard.len());
            for val in arr_guard.iter() {
                match val {
                    Value::Number(n) => {
                        let byte = *n as i32;
                        if !(0..=255).contains(&byte) {
                            return Err(RuntimeError::IoError {
                                message: format!("Byte value out of range: {}", byte),
                                span,
                            });
                        }
                        bytes.push(byte as u8);
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Array must contain only numbers (0-255)".to_string(),
                            span,
                        });
                    }
                }
            }
            bytes
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Data must be string or byte array".to_string(),
                span,
            });
        }
    };

    // Compress
    let compressed = compress_bytes(&bytes, level, span)?;

    // Convert to Atlas byte array (array of numbers)
    let result: Vec<Value> = compressed
        .iter()
        .map(|&b| Value::Number(b as f64))
        .collect();
    Ok(Value::array(result))
}

/// gzipDecompress(compressed: array<number>) -> array<number>
///
/// Decompress gzip data to byte array.
pub fn gzip_decompress(compressed: &Value, span: Span) -> Result<Value, RuntimeError> {
    // Extract byte array
    let bytes = match compressed {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            let mut bytes = Vec::with_capacity(arr_guard.len());
            for val in arr_guard.iter() {
                match val {
                    Value::Number(n) => {
                        let byte = *n as i32;
                        if !(0..=255).contains(&byte) {
                            return Err(RuntimeError::IoError {
                                message: format!("Byte value out of range: {}", byte),
                                span,
                            });
                        }
                        bytes.push(byte as u8);
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Array must contain only numbers (0-255)".to_string(),
                            span,
                        });
                    }
                }
            }
            bytes
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Compressed data must be a byte array".to_string(),
                span,
            });
        }
    };

    // Decompress
    let decompressed = decompress_bytes(&bytes, span)?;

    // Convert to Atlas byte array
    let result: Vec<Value> = decompressed
        .iter()
        .map(|&b| Value::Number(b as f64))
        .collect();
    Ok(Value::array(result))
}

/// gzipDecompressString(compressed: array<number>) -> string
///
/// Decompress gzip data to UTF-8 string.
pub fn gzip_decompress_string(compressed: &Value, span: Span) -> Result<Value, RuntimeError> {
    // Extract byte array
    let bytes = match compressed {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            let mut bytes = Vec::with_capacity(arr_guard.len());
            for val in arr_guard.iter() {
                match val {
                    Value::Number(n) => {
                        let byte = *n as i32;
                        if !(0..=255).contains(&byte) {
                            return Err(RuntimeError::IoError {
                                message: format!("Byte value out of range: {}", byte),
                                span,
                            });
                        }
                        bytes.push(byte as u8);
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Array must contain only numbers (0-255)".to_string(),
                            span,
                        });
                    }
                }
            }
            bytes
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Compressed data must be a byte array".to_string(),
                span,
            });
        }
    };

    // Decompress to string
    let text = decompress_string(&bytes, span)?;
    Ok(Value::string(text))
}

/// gzipIsGzip(data: array<number>) -> bool
///
/// Check if data is gzip-compressed (checks magic header).
pub fn gzip_is_gzip(data: &Value, span: Span) -> Result<Value, RuntimeError> {
    // Extract byte array
    let bytes = match data {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            let mut bytes = Vec::with_capacity(arr_guard.len().min(2));
            for val in arr_guard.iter().take(2) {
                match val {
                    Value::Number(n) => {
                        let byte = *n as i32;
                        if !(0..=255).contains(&byte) {
                            return Err(RuntimeError::IoError {
                                message: format!("Byte value out of range: {}", byte),
                                span,
                            });
                        }
                        bytes.push(byte as u8);
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Array must contain only numbers (0-255)".to_string(),
                            span,
                        });
                    }
                }
            }
            bytes
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Data must be a byte array".to_string(),
                span,
            });
        }
    };

    Ok(Value::Bool(is_gzip(&bytes)))
}

/// gzipCompressionRatio(original_size: number, compressed_size: number) -> number
///
/// Calculate compression ratio (original / compressed).
pub fn gzip_compression_ratio(
    original: &Value,
    compressed: &Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    let orig_size = match original {
        Value::Number(n) => *n as usize,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Original size must be a number".to_string(),
                span,
            });
        }
    };

    let comp_size = match compressed {
        Value::Number(n) => *n as usize,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Compressed size must be a number".to_string(),
                span,
            });
        }
    };

    let ratio = compression_ratio(orig_size, comp_size);
    Ok(Value::Number(ratio))
}
