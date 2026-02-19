//! Color-aware diagnostic formatter
//!
//! Formats diagnostics with source snippets, caret indicators, and optional
//! terminal colors. Respects NO_COLOR environment variable and auto-detects
//! terminal capabilities.

use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Color mode for diagnostic output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Always use colors
    Always,
    /// Never use colors
    Never,
    /// Auto-detect terminal capabilities
    Auto,
}

impl ColorMode {
    /// Resolve to a termcolor ColorChoice
    pub fn to_color_choice(self) -> ColorChoice {
        // Always respect NO_COLOR (https://no-color.org)
        if std::env::var("NO_COLOR").is_ok() {
            return ColorChoice::Never;
        }
        match self {
            ColorMode::Always => ColorChoice::Always,
            ColorMode::Never => ColorChoice::Never,
            ColorMode::Auto => ColorChoice::Auto,
        }
    }
}

/// Diagnostic formatter with color support
pub struct DiagnosticFormatter {
    color_mode: ColorMode,
}

impl DiagnosticFormatter {
    /// Create a new formatter with the given color mode
    pub fn new(color_mode: ColorMode) -> Self {
        Self { color_mode }
    }

    /// Create a formatter that auto-detects color support
    pub fn auto() -> Self {
        Self::new(ColorMode::Auto)
    }

    /// Create a plain (no color) formatter
    pub fn plain() -> Self {
        Self::new(ColorMode::Never)
    }

    /// Format a diagnostic to a string (without colors)
    pub fn format_to_string(&self, diag: &Diagnostic) -> String {
        diag.to_human_string()
    }

    /// Format a diagnostic with colors to stderr
    pub fn emit(&self, diag: &Diagnostic) {
        let mut stream = StandardStream::stderr(self.color_mode.to_color_choice());
        let _ = self.write_diagnostic(&mut stream, diag);
    }

    /// Format a diagnostic with colors to a WriteColor sink
    pub fn write_diagnostic(
        &self,
        w: &mut impl WriteColor,
        diag: &Diagnostic,
    ) -> std::io::Result<()> {
        // Header: error[AT0001]: message
        self.write_header(w, diag)?;

        // Location: --> file:line:column
        self.write_location(w, diag)?;

        // Snippet with carets
        if !diag.snippet.is_empty() {
            self.write_snippet(w, diag)?;
        }

        // Notes
        for note in &diag.notes {
            self.write_note(w, note)?;
        }

        // Related locations
        for related in &diag.related {
            self.write_note(
                w,
                &format!(
                    "related location at {}:{}:{}: {}",
                    related.file, related.line, related.column, related.message
                ),
            )?;
        }

        // Help
        if let Some(help) = &diag.help {
            self.write_help(w, help)?;
        }

        writeln!(w)?;
        Ok(())
    }

    fn write_header(&self, w: &mut impl WriteColor, diag: &Diagnostic) -> std::io::Result<()> {
        let (color, label) = match diag.level {
            DiagnosticLevel::Error => (Color::Red, "error"),
            DiagnosticLevel::Warning => (Color::Yellow, "warning"),
        };

        w.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
        write!(w, "{}[{}]", label, diag.code)?;
        w.reset()?;

        w.set_color(ColorSpec::new().set_bold(true))?;
        write!(w, ": {}", diag.message)?;
        w.reset()?;
        writeln!(w)?;
        Ok(())
    }

    fn write_location(&self, w: &mut impl WriteColor, diag: &Diagnostic) -> std::io::Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(w, "  --> ")?;
        w.reset()?;
        writeln!(w, "{}:{}:{}", diag.file, diag.line, diag.column)?;
        Ok(())
    }

    fn write_snippet(&self, w: &mut impl WriteColor, diag: &Diagnostic) -> std::io::Result<()> {
        let line_num_str = format!("{}", diag.line);
        let gutter_width = line_num_str.len() + 1;

        // Empty gutter line
        w.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(w, "{:>width$}|", "", width = gutter_width)?;
        w.reset()?;
        writeln!(w)?;

        // Source line
        w.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(w, "{:>width$}| ", diag.line, width = gutter_width)?;
        w.reset()?;
        writeln!(w, "{}", diag.snippet)?;

        // Caret line
        if diag.length > 0 {
            w.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
            write!(w, "{:>width$}| ", "", width = gutter_width)?;
            w.reset()?;

            // Compute caret position accounting for Unicode
            let padding = compute_display_width(&diag.snippet, diag.column.saturating_sub(1));
            write!(w, "{}", " ".repeat(padding))?;

            let col = diag.column.saturating_sub(1);
            let caret_len = diag
                .length
                .min(diag.snippet.len().saturating_sub(col).max(1));

            let color = match diag.level {
                DiagnosticLevel::Error => Color::Red,
                DiagnosticLevel::Warning => Color::Yellow,
            };
            w.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
            write!(w, "{}", "^".repeat(caret_len))?;

            if !diag.label.is_empty() {
                write!(w, " {}", diag.label)?;
            }
            w.reset()?;
            writeln!(w)?;
        }

        Ok(())
    }

    fn write_note(&self, w: &mut impl WriteColor, note: &str) -> std::io::Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(w, "   = ")?;
        w.reset()?;

        w.set_color(ColorSpec::new().set_bold(true))?;
        write!(w, "note")?;
        w.reset()?;

        writeln!(w, ": {}", note)?;
        Ok(())
    }

    fn write_help(&self, w: &mut impl WriteColor, help: &str) -> std::io::Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(w, "   = ")?;
        w.reset()?;

        w.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
        write!(w, "help")?;
        w.reset()?;

        writeln!(w, ": {}", help)?;
        Ok(())
    }

    /// Format multiple diagnostics
    pub fn emit_all(&self, diagnostics: &[Diagnostic]) {
        for diag in diagnostics {
            self.emit(diag);
        }
    }

    /// Format a diagnostic to a buffer (for testing)
    pub fn format_to_buffer(&self, diag: &Diagnostic) -> Vec<u8> {
        let mut buf = termcolor::Buffer::no_color();
        let _ = self.write_diagnostic(&mut buf, diag);
        buf.into_inner()
    }
}

/// Compute display width for the first `n` characters of a string,
/// handling Unicode characters that may have different byte widths
fn compute_display_width(s: &str, n: usize) -> usize {
    // For now, count characters (not bytes) up to position n
    // This handles basic Unicode correctly
    s.chars().take(n).count()
}

/// Format a source snippet from full source text given a span
pub fn extract_snippet(source: &str, line: usize) -> Option<String> {
    source.lines().nth(line.saturating_sub(1)).map(String::from)
}

/// Compute line and column from byte offset in source
pub fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Enrich a diagnostic with source information (line, column, snippet)
pub fn enrich_diagnostic(diag: Diagnostic, source: &str) -> Diagnostic {
    let span_start = diag.column.saturating_sub(1); // column is 1-based from span.start+1
    let (line, _col) = offset_to_line_col(source, span_start);
    let snippet = extract_snippet(source, line).unwrap_or_default();
    diag.with_line(line).with_snippet(snippet)
}

impl Default for DiagnosticFormatter {
    fn default() -> Self {
        Self::auto()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_format_error_to_buffer() {
        let formatter = DiagnosticFormatter::plain();
        let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(8, 13))
            .with_file("test.atlas")
            .with_line(5)
            .with_snippet("let x: number = \"hello\";")
            .with_label("expected number, found string")
            .with_help("convert the value to number");

        let buf = formatter.format_to_buffer(&diag);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("error[AT0001]"));
        assert!(output.contains("Type mismatch"));
        assert!(output.contains("test.atlas:5:9"));
        assert!(output.contains("^^^^^"));
        assert!(output.contains("help:"));
    }

    #[test]
    fn test_format_warning_to_buffer() {
        let formatter = DiagnosticFormatter::plain();
        let diag = Diagnostic::warning_with_code("AT2001", "Unused variable 'x'", Span::new(4, 5))
            .with_file("test.atlas")
            .with_line(1)
            .with_snippet("let x: number = 42;")
            .with_label("declared here but never used");

        let buf = formatter.format_to_buffer(&diag);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("warning[AT2001]"));
        assert!(output.contains("Unused variable"));
    }

    #[test]
    fn test_no_snippet() {
        let formatter = DiagnosticFormatter::plain();
        let diag = Diagnostic::error("some error", Span::new(0, 1))
            .with_file("test.atlas")
            .with_line(1);

        let buf = formatter.format_to_buffer(&diag);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("error[AT9999]"));
        assert!(!output.contains("^")); // No carets without snippet
    }

    #[test]
    fn test_with_notes() {
        let formatter = DiagnosticFormatter::plain();
        let diag = Diagnostic::error("test", Span::new(0, 1))
            .with_file("test.atlas")
            .with_line(1)
            .with_note("this is a note")
            .with_note("this is another note");

        let buf = formatter.format_to_buffer(&diag);
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("note: this is a note"));
        assert!(output.contains("note: this is another note"));
    }

    #[test]
    fn test_offset_to_line_col() {
        let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
        assert_eq!(offset_to_line_col(source, 0), (1, 1));
        assert_eq!(offset_to_line_col(source, 4), (1, 5));
        assert_eq!(offset_to_line_col(source, 11), (2, 1));
        assert_eq!(offset_to_line_col(source, 22), (3, 1));
    }

    #[test]
    fn test_extract_snippet() {
        let source = "line one\nline two\nline three";
        assert_eq!(extract_snippet(source, 1).unwrap(), "line one");
        assert_eq!(extract_snippet(source, 2).unwrap(), "line two");
        assert_eq!(extract_snippet(source, 3).unwrap(), "line three");
        assert!(extract_snippet(source, 4).is_none());
    }

    #[test]
    fn test_compute_display_width_ascii() {
        assert_eq!(compute_display_width("hello", 3), 3);
        assert_eq!(compute_display_width("hello", 0), 0);
    }

    #[test]
    fn test_compute_display_width_unicode() {
        let s = "héllo";
        assert_eq!(compute_display_width(s, 2), 2);
    }

    #[test]
    fn test_color_mode_no_color() {
        // We can't easily test NO_COLOR env var mutation in parallel tests,
        // but we can test the enum variants
        let no_color = std::env::var("NO_COLOR").is_ok();
        let expected_always = if no_color {
            ColorChoice::Never
        } else {
            ColorChoice::Always
        };
        assert_eq!(ColorMode::Always.to_color_choice(), expected_always);
        assert_eq!(ColorMode::Never.to_color_choice(), ColorChoice::Never);
    }

    #[test]
    fn test_enrich_diagnostic() {
        let source = "let x = 1;\nlet y = \"hello\";\nlet z = 3;";
        let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(11, 14))
            .with_file("test.atlas");
        // column is set to span.start+1 = 12 by error_with_code
        // enrich uses column-1 as offset, so offset = 11
        let enriched = enrich_diagnostic(diag, source);
        assert_eq!(enriched.line, 2);
        assert!(!enriched.snippet.is_empty());
    }

    #[test]
    fn test_format_to_string_matches_human_string() {
        let formatter = DiagnosticFormatter::plain();
        let diag = Diagnostic::error("test error", Span::new(0, 5))
            .with_file("test.atlas")
            .with_line(1)
            .with_snippet("hello world")
            .with_label("here");

        let formatted = formatter.format_to_string(&diag);
        let human = diag.to_human_string();
        assert_eq!(formatted, human);
    }

    #[test]
    fn test_empty_file_snippet() {
        let source = "";
        assert!(extract_snippet(source, 1).is_none());
    }

    #[test]
    fn test_first_line_snippet() {
        let source = "hello world";
        assert_eq!(extract_snippet(source, 1).unwrap(), "hello world");
    }

    #[test]
    fn test_last_line_snippet() {
        let source = "a\nb\nc";
        assert_eq!(extract_snippet(source, 3).unwrap(), "c");
    }
}
