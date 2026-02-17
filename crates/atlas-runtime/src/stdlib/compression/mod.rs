//! Compression utilities
//!
//! Provides gzip compression/decompression, tar archive management, and zip archive management.

pub mod gzip;
pub mod tar;
pub mod zip;

// Re-export main functions
pub use gzip::{compress_bytes, compress_string, decompress_bytes, decompress_string, is_gzip};
