//! Diagnostic normalization for stable golden tests
//!
//! Normalizes diagnostics by stripping non-deterministic data like
//! absolute paths, timestamps, and machine-specific information.

use crate::diagnostic::{Diagnostic, RelatedLocation};
use std::path::Path;

/// Normalize a diagnostic for golden testing
pub fn normalize_diagnostic_for_testing(diag: &Diagnostic) -> Diagnostic {
    let mut normalized = diag.clone();

    // Normalize file path to relative or placeholder
    normalized.file = normalize_path(&diag.file);

    // Normalize related locations
    normalized.related = diag
        .related
        .iter()
        .map(|rel| RelatedLocation {
            file: normalize_path(&rel.file),
            line: rel.line,
            column: rel.column,
            length: rel.length,
            message: rel.message.clone(),
        })
        .collect();

    normalized
}

/// Normalize a file path for testing
///
/// Converts absolute paths to just the filename, or uses placeholders
/// for common test paths like "<input>", "<unknown>", etc.
fn normalize_path(path: &str) -> String {
    // Keep special paths as-is
    if path.starts_with('<') && path.ends_with('>') {
        return path.to_string();
    }

    // For absolute paths, try to make them relative to current dir
    if Path::new(path).is_absolute() {
        // First try to strip current directory prefix
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(relative) = Path::new(path).strip_prefix(&current_dir) {
                return relative.display().to_string();
            }
        }

        // If that fails, just use the filename
        if let Some(filename) = Path::new(path).file_name() {
            return filename.to_string_lossy().to_string();
        }
    }

    // Return as-is if already relative or can't be normalized
    path.to_string()
}

/// Normalize a collection of diagnostics
pub fn normalize_diagnostics_for_testing(diags: &[Diagnostic]) -> Vec<Diagnostic> {
    diags.iter().map(normalize_diagnostic_for_testing).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Diagnostic;
    use crate::span::Span;

    /// Get an absolute path that works on both Unix and Windows
    #[cfg(unix)]
    fn absolute_test_path(filename: &str) -> String {
        format!("/absolute/path/to/{}", filename)
    }

    #[cfg(windows)]
    fn absolute_test_path(filename: &str) -> String {
        format!("C:\\absolute\\path\\to\\{}", filename)
    }

    /// Check if a path looks absolute (cross-platform check for test assertions)
    fn looks_like_absolute_path(path: &str) -> bool {
        #[cfg(unix)]
        {
            path.starts_with('/')
        }
        #[cfg(windows)]
        {
            // Windows absolute paths typically start with drive letter
            path.len() >= 2 && path.chars().nth(1) == Some(':')
        }
    }

    #[test]
    fn test_normalize_special_paths() {
        assert_eq!(normalize_path("<input>"), "<input>");
        assert_eq!(normalize_path("<unknown>"), "<unknown>");
        assert_eq!(normalize_path("<stdin>"), "<stdin>");
    }

    #[test]
    fn test_normalize_relative_paths() {
        assert_eq!(normalize_path("test.atlas"), "test.atlas");
        assert_eq!(normalize_path("src/main.atlas"), "src/main.atlas");
    }

    #[test]
    fn test_normalize_diagnostic() {
        let abs_path = absolute_test_path("test.atlas");
        let diag = Diagnostic::error("test error", Span::new(0, 5))
            .with_file(&abs_path)
            .with_line(10);

        let normalized = normalize_diagnostic_for_testing(&diag);

        // Path should be normalized (not the original absolute path)
        assert_ne!(normalized.file, abs_path);

        // Other fields should be preserved
        assert_eq!(normalized.message, "test error");
        assert_eq!(normalized.line, 10);
        assert_eq!(normalized.diag_version, diag.diag_version);
    }

    #[test]
    fn test_normalize_diagnostic_with_special_path() {
        let diag = Diagnostic::error("test error", Span::new(0, 5))
            .with_file("<input>")
            .with_line(5);

        let normalized = normalize_diagnostic_for_testing(&diag);

        // Special paths should remain unchanged
        assert_eq!(normalized.file, "<input>");
    }

    #[test]
    fn test_normalize_related_locations() {
        let abs_path = absolute_test_path("other.atlas");
        let diag =
            Diagnostic::error("test", Span::new(0, 1)).with_related_location(RelatedLocation {
                file: abs_path.clone(),
                line: 5,
                column: 10,
                length: 3,
                message: "defined here".to_string(),
            });

        let normalized = normalize_diagnostic_for_testing(&diag);

        assert_eq!(normalized.related.len(), 1);
        // Related path should also be normalized
        assert_ne!(normalized.related[0].file, abs_path);
    }

    #[test]
    fn test_normalize_diagnostics_collection() {
        let path_a = absolute_test_path("a.atlas");
        let path_b = absolute_test_path("b.atlas");
        let diags = vec![
            Diagnostic::error("error 1", Span::new(0, 1)).with_file(&path_a),
            Diagnostic::error("error 2", Span::new(0, 1)).with_file(&path_b),
        ];

        let normalized = normalize_diagnostics_for_testing(&diags);

        assert_eq!(normalized.len(), 2);
        // All paths should be normalized (no longer absolute)
        for norm_diag in &normalized {
            assert!(
                !looks_like_absolute_path(&norm_diag.file),
                "Path should be normalized but got: {}",
                norm_diag.file
            );
        }
    }
}
