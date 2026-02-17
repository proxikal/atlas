//! Atlas Code Formatter
//!
//! Automatic code formatting with comment preservation for Atlas source code.

pub mod comments;
mod formatter;
mod visitor;

pub use comments::{Comment, CommentKind, CommentPosition};
pub use formatter::{FormatConfig, FormatResult, Formatter};

/// Format Atlas source code with default configuration
pub fn format_source(source: &str) -> FormatResult {
    let config = FormatConfig::default();
    format_source_with_config(source, &config)
}

/// Format Atlas source code with custom configuration
pub fn format_source_with_config(source: &str, config: &FormatConfig) -> FormatResult {
    let mut formatter = Formatter::new(config.clone());
    formatter.format(source)
}

/// Check if source code is already formatted (without modifying)
pub fn check_formatted(source: &str) -> bool {
    check_formatted_with_config(source, &FormatConfig::default())
}

/// Check if source code is already formatted with custom configuration
pub fn check_formatted_with_config(source: &str, config: &FormatConfig) -> bool {
    match format_source_with_config(source, config) {
        FormatResult::Ok(formatted) => formatted == source,
        FormatResult::ParseError(_) => false,
    }
}
