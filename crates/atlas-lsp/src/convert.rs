//! Type conversions between Atlas and LSP types

use atlas_runtime::{Diagnostic, DiagnosticLevel};
use tower_lsp::lsp_types;

/// Convert an Atlas diagnostic to an LSP diagnostic
pub fn diagnostic_to_lsp(diag: &Diagnostic) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range: lsp_types::Range {
            start: lsp_types::Position {
                line: (diag.line.saturating_sub(1)) as u32,
                character: (diag.column.saturating_sub(1)) as u32,
            },
            end: lsp_types::Position {
                line: (diag.line.saturating_sub(1)) as u32,
                character: (diag.column.saturating_sub(1) + diag.length) as u32,
            },
        },
        severity: Some(match diag.level {
            DiagnosticLevel::Error => lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticLevel::Warning => lsp_types::DiagnosticSeverity::WARNING,
        }),
        code: Some(lsp_types::NumberOrString::String(diag.code.clone())),
        source: Some("atlas".to_string()),
        message: diag.message.clone(),
        ..Default::default()
    }
}
