//! Compression utilities
//!
//! Provides gzip compression and decompression for data and file operations.

pub mod gzip;

// Re-export main functions
pub use gzip::{compress_bytes, compress_string, decompress_bytes, decompress_string, is_gzip};
