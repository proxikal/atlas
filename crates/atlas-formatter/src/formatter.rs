//! Formatter core - configuration and entry point

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use serde::{Deserialize, Serialize};

use crate::comments::CommentCollector;
use crate::visitor::FormatVisitor;

/// Formatter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Number of spaces per indentation level (default: 4)
    pub indent_size: usize,
    /// Maximum line width before breaking (default: 100)
    pub max_width: usize,
    /// Whether to add trailing commas in multi-line constructs (default: true)
    pub trailing_commas: bool,
    /// Semicolon style: "always" (default)
    pub semicolon_style: SemicolonStyle,
}

/// Semicolon insertion style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemicolonStyle {
    /// Always include semicolons
    Always,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_width: 100,
            trailing_commas: true,
            semicolon_style: SemicolonStyle::Always,
        }
    }
}

impl FormatConfig {
    /// Create config with custom indent size
    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    /// Create config with custom max width
    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    /// Create config with trailing commas setting
    pub fn with_trailing_commas(mut self, enabled: bool) -> Self {
        self.trailing_commas = enabled;
        self
    }
}

/// Result of formatting
#[derive(Debug, Clone, PartialEq)]
pub enum FormatResult {
    /// Successfully formatted code
    Ok(String),
    /// Source code has parse errors
    ParseError(Vec<String>),
}

/// The main formatter
pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        Self { config }
    }

    /// Format source code, returning the formatted string or parse errors
    pub fn format(&mut self, source: &str) -> FormatResult {
        // Step 1: Tokenize with comments
        let mut lexer = Lexer::new(source);
        let (tokens, _lex_diags) = lexer.tokenize_with_comments();

        // Step 2: Collect comments from token stream
        let mut collector = CommentCollector::new();
        collector.collect_from_tokens(&tokens, source);
        let comments = collector.into_comments();

        // Step 3: Parse source into AST (using a fresh lexer, since parser expects no comments)
        let mut parse_lexer = Lexer::new(source);
        let (parse_tokens, _) = parse_lexer.tokenize();
        let mut parser = Parser::new(parse_tokens);
        let (program, parse_diags) = parser.parse();

        if !parse_diags.is_empty() {
            let errors: Vec<String> = parse_diags.iter().map(|d| d.message.clone()).collect();
            return FormatResult::ParseError(errors);
        }

        // Step 4: Visit AST and produce formatted output
        let mut visitor = FormatVisitor::new(self.config.clone(), comments, source.to_string());
        visitor.visit_program(&program);

        FormatResult::Ok(visitor.into_output())
    }
}
