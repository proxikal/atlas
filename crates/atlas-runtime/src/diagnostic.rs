//! Diagnostic system for errors and warnings
//!
//! All errors and warnings flow through the unified Diagnostic type,
//! ensuring consistent formatting across compiler, interpreter, and VM.

pub mod normalizer;

use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Diagnostic schema version
pub const DIAG_VERSION: u32 = 1;

/// Severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    /// Fatal error that prevents compilation
    #[serde(rename = "error")]
    Error,
    /// Warning that doesn't prevent compilation
    #[serde(rename = "warning")]
    Warning,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
        }
    }
}

/// Secondary location for related diagnostic information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedLocation {
    /// File path
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Length of the span
    pub length: usize,
    /// Description of this location
    pub message: String,
}

/// A diagnostic message (error or warning)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Diagnostic schema version
    pub diag_version: u32,
    /// Severity level
    pub level: DiagnosticLevel,
    /// Error code (e.g., "AT0001")
    pub code: String,
    /// Main diagnostic message
    pub message: String,
    /// File path
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Length of error span
    pub length: usize,
    /// Source line string
    pub snippet: String,
    /// Short label for caret range
    pub label: String,
    /// Additional notes (optional)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<String>,
    /// Related locations (optional)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related: Vec<RelatedLocation>,
    /// Suggested fix (optional)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub help: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic with code
    pub fn error_with_code(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            diag_version: DIAG_VERSION,
            level: DiagnosticLevel::Error,
            code: code.into(),
            message: message.into(),
            file: "<unknown>".to_string(),
            line: 1,
            column: span.start + 1,
            length: span.end.saturating_sub(span.start),
            snippet: "".to_string(),
            label: "".to_string(),
            notes: Vec::new(),
            related: Vec::new(),
            help: None,
        }
    }

    /// Create a new warning diagnostic with code
    pub fn warning_with_code(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            diag_version: DIAG_VERSION,
            level: DiagnosticLevel::Warning,
            code: code.into(),
            message: message.into(),
            file: "<unknown>".to_string(),
            line: 1,
            column: span.start + 1,
            length: span.end.saturating_sub(span.start),
            snippet: String::new(),
            label: String::new(),
            notes: Vec::new(),
            related: Vec::new(),
            help: None,
        }
    }

    /// Create a new error diagnostic (uses generic error code)
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::error_with_code("AT9999", message, span)
    }

    /// Create a new warning diagnostic (uses generic warning code)
    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self::warning_with_code("AW9999", message, span)
    }

    /// Set the file path
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = file.into();
        self
    }

    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = line;
        self
    }

    /// Set the snippet (source line)
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = snippet.into();
        self
    }

    /// Set the label (caret description)
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Add a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a help message
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add a related location
    pub fn with_related_location(mut self, location: RelatedLocation) -> Self {
        self.related.push(location);
        self
    }

    /// Format as human-readable string
    pub fn to_human_string(&self) -> String {
        let mut output = String::new();

        // Header: error[AT0001]: Type mismatch
        output.push_str(&format!(
            "{}[{}]: {}\n",
            self.level, self.code, self.message
        ));

        // Location: --> path/to/file.atl:12:9
        output.push_str(&format!(
            "  --> {}:{}:{}\n",
            self.file, self.line, self.column
        ));

        // Snippet with caret
        if !self.snippet.is_empty() {
            output.push_str("   |\n");
            output.push_str(&format!("{:>2} | {}\n", self.line, self.snippet));

            // Caret line
            if self.length > 0 {
                let padding = " ".repeat(self.column - 1);
                let carets = "^".repeat(self.length);
                output.push_str(&format!("   | {}{}", padding, carets));

                if !self.label.is_empty() {
                    output.push_str(&format!(" {}", self.label));
                }
                output.push('\n');
            }
        }

        // Notes
        for note in &self.notes {
            output.push_str(&format!("   = note: {}\n", note));
        }

        // Related locations
        for related in &self.related {
            output.push_str(&format!(
                "   = note: related location at {}:{}:{}: {}\n",
                related.file, related.line, related.column, related.message
            ));
        }

        // Help
        if let Some(help) = &self.help {
            output.push_str(&format!("   = help: {}\n", help));
        }

        output
    }

    /// Format as JSON string
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Format as compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Sort diagnostics by level (errors first), then by location
pub fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|a, b| {
        // Errors before warnings
        match (a.level, b.level) {
            (DiagnosticLevel::Error, DiagnosticLevel::Warning) => std::cmp::Ordering::Less,
            (DiagnosticLevel::Warning, DiagnosticLevel::Error) => std::cmp::Ordering::Greater,
            _ => {
                // Same level: sort by file, line, column
                a.file
                    .cmp(&b.file)
                    .then(a.line.cmp(&b.line))
                    .then(a.column.cmp(&b.column))
            }
        }
    });
}

/// Error code registry
pub mod error_codes {
    // AT0xxx - Type and Runtime Errors
    pub const TYPE_MISMATCH: &str = "AT0001";
    pub const UNDEFINED_SYMBOL: &str = "AT0002";
    pub const DIVIDE_BY_ZERO: &str = "AT0005";
    pub const ARRAY_OUT_OF_BOUNDS: &str = "AT0006";
    pub const INVALID_NUMERIC_RESULT: &str = "AT0007";
    pub const STDLIB_ARG_ERROR: &str = "AT0102";
    pub const STDLIB_VALUE_ERROR: &str = "AT0103";

    // AT03xx - Permission Errors
    pub const FILESYSTEM_PERMISSION_DENIED: &str = "AT0300";
    pub const NETWORK_PERMISSION_DENIED: &str = "AT0301";
    pub const PROCESS_PERMISSION_DENIED: &str = "AT0302";
    pub const ENVIRONMENT_PERMISSION_DENIED: &str = "AT0303";

    // AT1xxx - Syntax Errors
    pub const SYNTAX_ERROR: &str = "AT1000";
    pub const UNEXPECTED_TOKEN: &str = "AT1001";
    pub const UNTERMINATED_STRING: &str = "AT1002";
    pub const INVALID_ESCAPE: &str = "AT1003";
    pub const UNTERMINATED_COMMENT: &str = "AT1004";
    pub const SHADOWING_PRELUDE: &str = "AT1012";

    // AT2xxx - Warnings
    pub const UNUSED_VARIABLE: &str = "AT2001";
    pub const UNREACHABLE_CODE: &str = "AT2002";
    pub const DUPLICATE_DECLARATION: &str = "AT2003";

    // AT3xxx - Semantic and Type Checking Errors
    pub const TYPE_ERROR: &str = "AT3001";
    pub const BINARY_OP_TYPE_ERROR: &str = "AT3002";
    pub const IMMUTABLE_ASSIGNMENT: &str = "AT3003";
    pub const MISSING_RETURN: &str = "AT3004";
    pub const ARITY_MISMATCH: &str = "AT3005";
    pub const NOT_CALLABLE: &str = "AT3006";
    pub const INVALID_INDEX_TYPE: &str = "AT3010";
    pub const NOT_INDEXABLE: &str = "AT3011";
    pub const MATCH_EMPTY: &str = "AT3020";
    pub const MATCH_ARM_TYPE_MISMATCH: &str = "AT3021";
    pub const PATTERN_TYPE_MISMATCH: &str = "AT3022";
    pub const CONSTRUCTOR_ARITY: &str = "AT3023";
    pub const UNKNOWN_CONSTRUCTOR: &str = "AT3024";
    pub const UNSUPPORTED_PATTERN_TYPE: &str = "AT3025";
    pub const ARRAY_PATTERN_TYPE_MISMATCH: &str = "AT3026";
    pub const NON_EXHAUSTIVE_MATCH: &str = "AT3027";

    // AT5xxx - Module System Errors
    pub const INVALID_MODULE_PATH: &str = "AT5001";
    pub const MODULE_NOT_FOUND: &str = "AT5002";
    pub const CIRCULAR_DEPENDENCY: &str = "AT5003";
    pub const EXPORT_NOT_FOUND: &str = "AT5004";
    pub const IMPORT_RESOLUTION_FAILED: &str = "AT5005";
    pub const MODULE_NOT_EXPORTED: &str = "AT5006";
    pub const NAMESPACE_IMPORT_UNSUPPORTED: &str = "AT5007";
    pub const DUPLICATE_EXPORT: &str = "AT5008";

    // AT9xxx - Internal Errors
    pub const INTERNAL_ERROR: &str = "AT9995";
    pub const STACK_UNDERFLOW: &str = "AT9997";
    pub const UNKNOWN_OPCODE: &str = "AT9998";
    pub const GENERIC_ERROR: &str = "AT9999";
    pub const GENERIC_WARNING: &str = "AW9999";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("test error", Span::new(0, 5));
        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert_eq!(diag.message, "test error");
        assert_eq!(diag.diag_version, DIAG_VERSION);
    }

    #[test]
    fn test_diagnostic_with_code() {
        let diag = Diagnostic::error_with_code("AT0001", "type error", Span::new(5, 10));
        assert_eq!(diag.code, "AT0001");
        assert_eq!(diag.level, DiagnosticLevel::Error);
    }

    #[test]
    fn test_warning_creation() {
        let diag = Diagnostic::warning("test warning", Span::new(0, 3));
        assert_eq!(diag.level, DiagnosticLevel::Warning);
    }

    #[test]
    fn test_builder_pattern() {
        let diag = Diagnostic::error("test", Span::new(0, 4))
            .with_file("test.atlas")
            .with_line(10)
            .with_snippet("let x = y;")
            .with_label("undefined variable")
            .with_note("y is not defined in this scope")
            .with_help("define y before using it");

        assert_eq!(diag.file, "test.atlas");
        assert_eq!(diag.line, 10);
        assert_eq!(diag.snippet, "let x = y;");
        assert_eq!(diag.label, "undefined variable");
        assert_eq!(diag.notes.len(), 1);
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_human_format() {
        let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(8, 13))
            .with_file("test.atlas")
            .with_line(12)
            .with_snippet("let x: number = \"hello\";")
            .with_label("expected number, found string")
            .with_help("convert the value to number");

        let output = diag.to_human_string();
        assert!(output.contains("error[AT0001]"));
        assert!(output.contains("Type mismatch"));
        assert!(output.contains("test.atlas:12"));
        assert!(output.contains("^^^^^"));
    }

    #[test]
    fn test_json_format() {
        let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(0, 5))
            .with_file("test.atlas")
            .with_line(1)
            .with_snippet("test")
            .with_label("error here");

        let json = diag.to_json_string().unwrap();
        assert!(json.contains("\"diag_version\": 1"));
        assert!(json.contains("\"level\": \"error\""));
        assert!(json.contains("\"code\": \"AT0001\""));
        assert!(json.contains("\"message\": \"Type mismatch\""));
    }

    #[test]
    fn test_json_stable_ordering() {
        let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 1))
            .with_file("test.atlas")
            .with_line(1);

        let json1 = diag.to_json_compact().unwrap();
        let json2 = diag.clone().to_json_compact().unwrap();

        // Same diagnostic should produce identical JSON
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_sort_diagnostics() {
        let mut diagnostics = vec![
            Diagnostic::warning("warn1", Span::new(0, 1))
                .with_file("a.atlas")
                .with_line(5),
            Diagnostic::error("err1", Span::new(0, 1))
                .with_file("b.atlas")
                .with_line(1),
            Diagnostic::error("err2", Span::new(0, 1))
                .with_file("a.atlas")
                .with_line(10),
            Diagnostic::warning("warn2", Span::new(0, 1))
                .with_file("a.atlas")
                .with_line(1),
        ];

        sort_diagnostics(&mut diagnostics);

        // Should be: errors first, then by file/line
        assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
        assert_eq!(diagnostics[0].file, "a.atlas");
        assert_eq!(diagnostics[1].level, DiagnosticLevel::Error);
        assert_eq!(diagnostics[1].file, "b.atlas");
        assert_eq!(diagnostics[2].level, DiagnosticLevel::Warning);
        assert_eq!(diagnostics[3].level, DiagnosticLevel::Warning);
    }

    #[test]
    fn test_related_locations() {
        let diag =
            Diagnostic::error("test", Span::new(0, 1)).with_related_location(RelatedLocation {
                file: "other.atlas".to_string(),
                line: 5,
                column: 10,
                length: 3,
                message: "defined here".to_string(),
            });

        assert_eq!(diag.related.len(), 1);
        assert_eq!(diag.related[0].file, "other.atlas");
    }

    #[test]
    fn test_diagnostic_level_display() {
        assert_eq!(DiagnosticLevel::Error.to_string(), "error");
        assert_eq!(DiagnosticLevel::Warning.to_string(), "warning");
    }

    #[test]
    fn test_diagnostic_version_always_present() {
        // Test that version is always set in all diagnostic constructors
        let error = Diagnostic::error("test", Span::new(0, 1));
        assert_eq!(error.diag_version, DIAG_VERSION);

        let error_with_code = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 1));
        assert_eq!(error_with_code.diag_version, DIAG_VERSION);

        let warning = Diagnostic::warning("test", Span::new(0, 1));
        assert_eq!(warning.diag_version, DIAG_VERSION);

        let warning_with_code = Diagnostic::warning_with_code("AW0001", "test", Span::new(0, 1));
        assert_eq!(warning_with_code.diag_version, DIAG_VERSION);
    }

    #[test]
    fn test_diagnostic_version_in_json() {
        // Test that version appears in JSON output
        let diag = Diagnostic::error("test", Span::new(0, 1));
        let json = diag.to_json_string().unwrap();

        assert!(
            json.contains(&format!("\"diag_version\": {}", DIAG_VERSION)),
            "JSON output should contain diag_version field: {}",
            json
        );
    }

    #[test]
    fn test_diagnostic_version_deserialization() {
        // Test that version can be round-tripped through JSON
        let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 1))
            .with_file("test.atlas")
            .with_line(1);

        let json = diag.to_json_string().unwrap();
        let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.diag_version, DIAG_VERSION);
        assert_eq!(deserialized, diag);
    }
}
