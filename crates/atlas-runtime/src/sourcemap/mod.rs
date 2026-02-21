//! Source Map v3 generation and consumption.
//!
//! Maps compiled bytecode positions back to original Atlas source code.
//! Implements the [Source Map v3 specification](https://sourcemaps.info/spec.html).
//!
//! # Architecture
//!
//! - `vlq` — Base64-VLQ encoding/decoding for compact position representation
//! - `encoder` — Source Map v3 JSON builder, serializer, and parser
//! - This module — Integration with the Atlas compiler and debugger

pub mod encoder;
pub mod vlq;

pub use encoder::{decode_mappings, MappingEntry, OriginalLocation, SourceMapBuilder, SourceMapV3};

use crate::bytecode::{Bytecode, DebugSpan};
use crate::debugger::source_map::compute_line_offsets;

/// Options for source map generation.
#[derive(Debug, Clone, Default)]
pub struct SourceMapOptions {
    /// Output file name (the .atlas.map reference).
    pub file: Option<String>,
    /// Root path prefix for source files.
    pub source_root: Option<String>,
    /// Whether to include source content inline.
    pub include_sources: bool,
}

impl SourceMapOptions {
    /// Create default options.
    pub fn new() -> Self {
        Self {
            file: None,
            source_root: None,
            include_sources: false,
        }
    }
}

/// Generate a Source Map v3 from compiled bytecode.
///
/// Converts the bytecode's `debug_info` spans into standard v3 mappings.
/// Each bytecode instruction offset is treated as a "generated position" on
/// a single line (the bytecode stream), and the original positions come from
/// the AST spans embedded during compilation.
///
/// # Arguments
/// - `bytecode` — compiled bytecode with debug_info
/// - `source_file` — name of the original source file
/// - `source_text` — optional source code for line/column computation and inlining
/// - `options` — source map generation options
pub fn generate_source_map(
    bytecode: &Bytecode,
    source_file: &str,
    source_text: Option<&str>,
    options: &SourceMapOptions,
) -> SourceMapV3 {
    let mut builder = SourceMapBuilder::new();

    if let Some(ref file) = options.file {
        builder.set_file(file);
    }
    if let Some(ref root) = options.source_root {
        builder.set_source_root(root);
    }

    let content = if options.include_sources {
        source_text.map(|s| s.to_string())
    } else {
        None
    };
    let source_idx = builder.add_source(source_file, content);

    let line_offsets = source_text
        .map(compute_line_offsets)
        .unwrap_or_else(|| vec![0]);

    // Convert debug spans to source map entries.
    // Each instruction offset → generated position (line 0, column = offset).
    // Original positions come from the span's byte offset → line/column.
    let mut entries: Vec<(usize, u32, u32)> = Vec::new();
    for debug_span in &bytecode.debug_info {
        if debug_span.span.start == 0 && debug_span.span.end == 0 {
            continue; // Skip dummy spans
        }
        let (orig_line, orig_col) = byte_offset_to_zero_based(debug_span.span.start, &line_offsets);
        entries.push((debug_span.instruction_offset, orig_line, orig_col));
    }

    // Remove redundant entries (same original position as previous)
    entries.dedup_by(|b, a| a.1 == b.1 && a.2 == b.2);

    for (offset, orig_line, orig_col) in &entries {
        builder.add_mapping(
            0,              // generated line (bytecode is a flat stream)
            *offset as u32, // generated column = instruction offset
            source_idx,
            *orig_line,
            *orig_col,
            None,
        );
    }

    builder.build()
}

/// Generate a source map from debug spans directly (for use without full Bytecode).
pub fn generate_from_debug_spans(
    spans: &[DebugSpan],
    source_file: &str,
    source_text: Option<&str>,
    options: &SourceMapOptions,
) -> SourceMapV3 {
    let bytecode = Bytecode {
        instructions: Vec::new(),
        constants: Vec::new(),
        debug_info: spans.to_vec(),
        top_level_local_count: 0,
    };
    generate_source_map(&bytecode, source_file, source_text, options)
}

/// Generate an inline source map comment (data URL).
///
/// Returns `//# sourceMappingURL=data:application/json;base64,...`
pub fn generate_inline_source_map(source_map: &SourceMapV3) -> Option<String> {
    let json = source_map.to_json().ok()?;
    let b64 = base64_encode(&json);
    Some(format!(
        "//# sourceMappingURL=data:application/json;base64,{}",
        b64
    ))
}

/// Base64 encode (no external dependency — simple implementation).
fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Convert a byte offset to 0-based (line, column) using pre-computed line offsets.
fn byte_offset_to_zero_based(offset: usize, line_offsets: &[usize]) -> (u32, u32) {
    let line_index = match line_offsets.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    let line_start = line_offsets[line_index];
    let column = offset.saturating_sub(line_start);
    (line_index as u32, column as u32)
}
