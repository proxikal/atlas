//! corpus.rs — file-based test corpus harness
//!
//! Drives `.atlas` files through the Atlas runtime and compares output against
//! companion snapshot files:
//!
//! - `tests/corpus/pass/**/*.atlas`  → compared against `.stdout` companion (runs in both engines)
//! - `tests/corpus/fail/**/*.atlas`  → compared against `.stderr` companion
//! - `tests/corpus/warn/**/*.atlas`  → compared against `.stderr` companion (warnings only)
//!
//! # Snapshot generation
//!
//! Set `UPDATE_CORPUS=1` to write actual output to snapshot files instead of asserting.
//! This is the workflow for adding new corpus tests or updating expected output after
//! an intentional behavior change.
//!
//! ```
//! UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus
//! ```
//!
//! # Interpreter/VM parity
//!
//! Every `pass/` file runs in both the Interpreter and the VM. If the outputs differ,
//! the parity assertion fails — keeping both engines in sync automatically.

use atlas_runtime::api::{ExecutionMode, Runtime, RuntimeConfig};
use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::DiagnosticLevel;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::stdlib::OutputWriter;
use atlas_runtime::typechecker::TypeChecker;
use rstest::rstest;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ============================================================================
// CaptureWriter — an in-memory Write impl for capturing print() output
// ============================================================================

struct CaptureWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Create a capture buffer and the corresponding OutputWriter.
fn capture_output() -> (Arc<Mutex<Vec<u8>>>, OutputWriter) {
    let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let writer: OutputWriter = Arc::new(Mutex::new(Box::new(CaptureWriter { buf: buf.clone() })));
    (buf, writer)
}

/// Run an Atlas source file through one execution engine and return printed output.
///
/// On eval error, returns an empty string (the error is not the output).
/// Pass tests that fail at eval time will show an empty string vs expected content.
fn run_pass(source: &str, mode: ExecutionMode) -> Result<String, String> {
    let (buf, writer) = capture_output();
    let config = RuntimeConfig::new()
        .with_output(writer)
        .with_io_allowed(false)
        .with_network_allowed(false);
    let mut runtime = Runtime::with_config(mode, config);
    match runtime.eval(source) {
        Ok(_) => Ok(String::from_utf8(buf.lock().unwrap().clone()).unwrap()),
        Err(e) => Err(format!("{}", e)),
    }
}

/// Run an Atlas source file and capture the error message (for fail/ tests).
///
/// Returns the formatted error. If eval succeeds (test is broken), returns
/// a sentinel that will cause the assertion to fail.
fn run_fail(source: &str) -> String {
    let (_buf, writer) = capture_output();
    let config = RuntimeConfig::new()
        .with_output(writer)
        .with_io_allowed(false)
        .with_network_allowed(false);
    let mut runtime = Runtime::with_config(ExecutionMode::Interpreter, config);
    match runtime.eval(source) {
        Ok(_) => "(no error — expected failure did not occur)".to_string(),
        Err(e) => format!("{}", e),
    }
}

/// Run an Atlas file (with import support) through one execution engine.
///
/// Uses eval_file() which handles multi-file imports properly.
fn run_pass_file(path: &std::path::Path, mode: ExecutionMode) -> Result<String, String> {
    let (buf, writer) = capture_output();
    let config = RuntimeConfig::new()
        .with_output(writer)
        .with_io_allowed(true)
        .with_network_allowed(false);
    let mut runtime = Runtime::with_config(mode, config);
    match runtime.eval_file(path) {
        Ok(_) => Ok(String::from_utf8(buf.lock().unwrap().clone()).unwrap()),
        Err(e) => Err(format!("{}", e)),
    }
}

/// Run an Atlas file and capture the error message (for fail/modules/ tests).
fn run_fail_file(path: &std::path::Path) -> String {
    let (_buf, writer) = capture_output();
    let config = RuntimeConfig::new()
        .with_output(writer)
        .with_io_allowed(true)
        .with_network_allowed(false);
    let mut runtime = Runtime::with_config(ExecutionMode::Interpreter, config);
    match runtime.eval_file(path) {
        Ok(_) => "(no error — expected failure did not occur)".to_string(),
        Err(e) => format!("{}", e),
    }
}

/// Run an Atlas source through the full pipeline and collect only WARNING diagnostics.
///
/// Uses the lower-level API (Lexer → Parser → Binder → TypeChecker) to separate
/// warnings from errors. This avoids the issue where `Runtime::eval()` treats
/// warning-only diagnostic lists as failures.
///
/// Returns newline-separated warning lines, each formatted as:
/// `warning[CODE]: message`
fn run_warn(source: &str) -> String {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();

    // Collect lex-time errors (these are always errors, never warnings)
    let lex_errors: Vec<String> = lex_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .map(|d| format!("error[{}]: {}", d.code, d.message))
        .collect();
    if !lex_errors.is_empty() {
        return lex_errors.join("\n");
    }

    let mut parser = Parser::new(tokens);
    let (ast, parse_diags) = parser.parse();

    let parse_errors: Vec<String> = parse_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .map(|d| format!("error[{}]: {}", d.code, d.message))
        .collect();
    if !parse_errors.is_empty() {
        return parse_errors.join("\n");
    }

    let mut binder = Binder::new();
    let (mut symbol_table, _bind_diags) = binder.bind(&ast);

    let mut type_checker = TypeChecker::new(&mut symbol_table);
    let type_diags = type_checker.check(&ast);

    // Collect only WARNING-level diagnostics (not errors).
    // Sort for deterministic output — HashMap iteration order is not stable.
    let mut warnings: Vec<String> = type_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .map(|d| format!("warning[{}]: {}", d.code, d.message))
        .collect();
    warnings.sort();

    warnings.join("\n")
}

// ============================================================================
// Snapshot assertion / update
// ============================================================================

/// Read the expected snapshot from disk, or write it if UPDATE_CORPUS=1 is set.
///
/// On mismatch, panics with a clear diff showing expected vs actual.
fn assert_snapshot(snapshot_path: &PathBuf, actual: &str) {
    let update = std::env::var("UPDATE_CORPUS").is_ok();

    if update {
        if let Some(parent) = snapshot_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create snapshot dir");
        }
        std::fs::write(snapshot_path, actual).expect("Failed to write snapshot");
        return;
    }

    match std::fs::read_to_string(snapshot_path) {
        Ok(expected) => {
            assert_eq!(
                actual,
                expected.as_str(),
                "\nCorpus snapshot mismatch: {}\n\n--- expected\n+++ actual",
                snapshot_path.display()
            );
        }
        Err(_) => {
            panic!(
                "Missing snapshot: {}\n\
                 Run with UPDATE_CORPUS=1 to generate it.\n\
                 Actual output:\n---\n{}\n---",
                snapshot_path.display(),
                actual
            );
        }
    }
}

// ============================================================================
// Pass tests — interpreter and VM, one named test per file via rstest #[files]
// ============================================================================

/// Run a pass/ corpus file in the Interpreter engine and compare against .stdout snapshot.
#[rstest]
fn pass_interpreter(#[files("tests/corpus/pass/**/*.atlas")] path: PathBuf) {
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    let output = run_pass(&source, ExecutionMode::Interpreter).unwrap_or_else(|e| {
        panic!(
            "Pass test failed in interpreter: {}\nFile: {}",
            e,
            path.display()
        )
    });

    let snapshot = path.with_extension("stdout");
    assert_snapshot(&snapshot, &output);
}

/// Run a pass/ corpus file in the VM engine and compare against .stdout snapshot.
///
/// Both engines must produce identical output (parity enforced via shared snapshot).
#[rstest]
fn pass_vm(#[files("tests/corpus/pass/**/*.atlas")] path: PathBuf) {
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    let output = run_pass(&source, ExecutionMode::VM)
        .unwrap_or_else(|e| panic!("Pass test failed in VM: {}\nFile: {}", e, path.display()));

    // The VM compares against the same .stdout snapshot as the interpreter.
    // If they disagree, the snapshot written by pass_interpreter will not match.
    let snapshot = path.with_extension("stdout");
    assert_snapshot(&snapshot, &output);
}

// ============================================================================
// Fail tests — must produce an error; compared against .stderr snapshot
// ============================================================================

/// Run a fail/ corpus file and compare the error message against .stderr snapshot.
#[rstest]
fn fail_corpus(#[files("tests/corpus/fail/**/*.atlas")] path: PathBuf) {
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    let error = run_fail(&source);

    let snapshot = path.with_extension("stderr");
    assert_snapshot(&snapshot, &error);
}

// ============================================================================
// Warn tests — must produce warnings; compared against .stderr snapshot
// ============================================================================

/// Run a warn/ corpus file, collect warning diagnostics, compare against .stderr snapshot.
#[rstest]
fn warn_corpus(#[files("tests/corpus/warn/**/*.atlas")] path: PathBuf) {
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    let warnings = run_warn(&source);

    // A warn/ test must produce at least one warning
    let snapshot = path.with_extension("stderr");

    if std::env::var("UPDATE_CORPUS").is_ok() {
        // In update mode, always write (even if empty)
        assert_snapshot(&snapshot, &warnings);
    } else {
        // Verify at least one warning was produced before comparing
        assert!(
            !warnings.is_empty(),
            "warn/ corpus test produced no warnings: {}\n\
             Check that the program actually triggers a warning diagnostic.",
            path.display()
        );
        assert_snapshot(&snapshot, &warnings);
    }
}

// ============================================================================
// Module tests — file-based with import support, parity between engines
// ============================================================================

/// Run a pass/modules/ corpus file in the Interpreter engine.
///
/// Module tests use eval_file() which properly handles imports.
/// Only files named main.atl are entry points (other .atl files are dependencies).
/// Note: Module corpus uses .atl extension (not .atlas) for import resolution.
#[rstest]
fn modules_pass_interpreter(#[files("tests/corpus/pass/modules/**/main.atl")] path: PathBuf) {
    let output = run_pass_file(&path, ExecutionMode::Interpreter).unwrap_or_else(|e| {
        panic!(
            "Module test failed in interpreter: {}\nFile: {}",
            e,
            path.display()
        )
    });

    let snapshot = path.with_extension("stdout");
    assert_snapshot(&snapshot, &output);
}

/// Run a pass/modules/ corpus file in the VM engine.
#[rstest]
fn modules_pass_vm(#[files("tests/corpus/pass/modules/**/main.atl")] path: PathBuf) {
    let output = run_pass_file(&path, ExecutionMode::VM)
        .unwrap_or_else(|e| panic!("Module test failed in VM: {}\nFile: {}", e, path.display()));

    let snapshot = path.with_extension("stdout");
    assert_snapshot(&snapshot, &output);
}

/// Run a fail/modules/ corpus file and compare error against .stderr snapshot.
#[rstest]
fn modules_fail_corpus(#[files("tests/corpus/fail/modules/**/main.atl")] path: PathBuf) {
    let error = run_fail_file(&path);

    let snapshot = path.with_extension("stderr");
    assert_snapshot(&snapshot, &error);
}
