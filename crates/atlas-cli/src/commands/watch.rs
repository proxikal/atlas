//! Watch mode for Atlas - automatic recompilation on file changes

use anyhow::{Context, Result};
use notify::{RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use atlas_runtime::{Atlas, SecurityContext, Value};

/// Debounce delay in milliseconds (spec: detect changes within 500ms)
const DEBOUNCE_MS: u64 = 300;

/// Watch mode configuration
pub struct WatchConfig {
    /// Clear terminal before each recompilation
    pub clear_screen: bool,
    /// Continue watching after errors
    pub continue_on_error: bool,
    /// Use JSON output for diagnostics
    pub json_output: bool,
    /// Show verbose timing information
    pub verbose: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            clear_screen: true,
            continue_on_error: true,
            json_output: false,
            verbose: false,
        }
    }
}

/// Run a file in watch mode, automatically recompiling on changes
pub fn run_watch(file_path: &str, config: WatchConfig) -> Result<()> {
    let path = PathBuf::from(file_path);

    // Verify file exists
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }

    // Get the parent directory to watch
    let watch_dir = path
        .parent()
        .map(|p| {
            if p.as_os_str().is_empty() {
                Path::new(".")
            } else {
                p
            }
        })
        .unwrap_or(Path::new("."));

    // Get canonical path for comparison
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());

    // Create channel for receiving file events
    let (tx, rx) = channel();

    // Create watcher
    let mut watcher = notify::recommended_watcher(tx).context("Failed to create file watcher")?;

    // Watch the directory containing the file
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .context("Failed to start watching directory")?;

    println!("Watching {} for changes...", file_path);
    println!("Press Ctrl+C to stop\n");

    // Initial run
    run_once(&path, &config);

    // Debounce state
    let mut last_run = Instant::now();
    let debounce_duration = Duration::from_millis(DEBOUNCE_MS);

    // Watch loop
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // Check if any path is relevant
                let should_rerun = event
                    .paths
                    .iter()
                    .any(|p| is_relevant_change(p, &canonical_path));

                if should_rerun {
                    // Debounce: skip if we ran too recently
                    let now = Instant::now();
                    if now.duration_since(last_run) < debounce_duration {
                        continue;
                    }
                    last_run = now;

                    if config.clear_screen {
                        clear_terminal();
                    }

                    if config.verbose {
                        eprintln!("[watch] Change detected, recompiling...\n");
                    }

                    run_once(&path, &config);
                }
            }
            Ok(Err(e)) => {
                eprintln!("[watch] Error: {:?}", e);
                if !config.continue_on_error {
                    break;
                }
            }
            Err(e) => {
                eprintln!("[watch] Channel error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Check if a change is relevant to trigger recompilation
fn is_relevant_change(changed_path: &Path, watched_path: &Path) -> bool {
    // Direct match
    if changed_path == watched_path {
        return true;
    }

    // Check if it's an Atlas file in the same directory
    if let Some(ext) = changed_path.extension() {
        if ext == "at" || ext == "atlas" {
            return true;
        }
    }

    false
}

/// Run the file once and display results
fn run_once(path: &Path, config: &WatchConfig) {
    let start = std::time::Instant::now();

    let runtime = Atlas::new_with_security(SecurityContext::allow_all());

    match runtime.eval_file(path.to_str().unwrap_or("")) {
        Ok(value) => {
            // Print the result value if it's not null
            if !matches!(value, Value::Null) {
                println!("{}", value);
            }

            if config.verbose {
                let elapsed = start.elapsed();
                eprintln!(
                    "\n[watch] Completed in {:.2}ms",
                    elapsed.as_secs_f64() * 1000.0
                );
            }

            println!();
            println!("Watching for changes...");
        }
        Err(diagnostics) => {
            // Print all diagnostics
            if config.json_output {
                for diag in &diagnostics {
                    if let Ok(json) = diag.to_json_string() {
                        println!("{}", json);
                    }
                }
            } else {
                eprintln!("Errors:");
                for diag in &diagnostics {
                    eprintln!("{}", format_diagnostic(diag));
                }
            }

            if config.verbose {
                let elapsed = start.elapsed();
                eprintln!(
                    "\n[watch] Failed in {:.2}ms",
                    elapsed.as_secs_f64() * 1000.0
                );
            }

            println!();
            println!("Watching for changes... (fix errors and save)");
        }
    }
}

/// Clear the terminal screen
fn clear_terminal() {
    // ANSI escape codes work on most terminals
    print!("\x1B[2J\x1B[1;1H");
    // Flush to ensure it takes effect
    use std::io::Write;
    let _ = std::io::stdout().flush();
}

/// Format a diagnostic for display
fn format_diagnostic(diag: &atlas_runtime::Diagnostic) -> String {
    use atlas_runtime::DiagnosticLevel;

    let level_str = match diag.level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warning => "warning",
    };

    format!(
        "{}:{}: {}: {}",
        diag.line, diag.column, level_str, diag.message
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert!(config.clear_screen);
        assert!(config.continue_on_error);
        assert!(!config.json_output);
        assert!(!config.verbose);
    }

    #[test]
    fn test_is_relevant_change_same_file() {
        let watched = Path::new("/test/file.at");
        let changed = Path::new("/test/file.at");
        assert!(is_relevant_change(changed, watched));
    }

    #[test]
    fn test_is_relevant_change_atlas_file() {
        let watched = Path::new("/test/main.at");
        let changed = Path::new("/test/other.at");
        assert!(is_relevant_change(changed, watched));
    }

    #[test]
    fn test_is_relevant_change_atlas_extension() {
        let watched = Path::new("/test/main.at");
        let changed = Path::new("/test/module.atlas");
        assert!(is_relevant_change(changed, watched));
    }

    #[test]
    fn test_is_relevant_change_non_atlas_file() {
        let watched = Path::new("/test/main.at");
        let changed = Path::new("/test/readme.md");
        assert!(!is_relevant_change(changed, watched));
    }

    #[test]
    fn test_format_diagnostic_error() {
        use atlas_runtime::{Diagnostic, Span};
        let diag = Diagnostic::error("test error".to_string(), Span::new(0, 5));
        let formatted = format_diagnostic(&diag);
        assert!(formatted.contains("error"));
        assert!(formatted.contains("test error"));
    }

    #[test]
    fn test_run_watch_nonexistent_file() {
        let result = run_watch("nonexistent_file.at", WatchConfig::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_run_once_valid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1 + 2;").unwrap();

        let config = WatchConfig {
            clear_screen: false,
            verbose: false,
            ..Default::default()
        };

        // run_once doesn't return a value, just verify it doesn't panic
        run_once(temp_file.path(), &config);
    }

    #[test]
    fn test_run_once_invalid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = \"invalid\";").unwrap();

        let config = WatchConfig {
            clear_screen: false,
            verbose: false,
            ..Default::default()
        };

        // run_once doesn't panic on errors, just displays them
        run_once(temp_file.path(), &config);
    }

    #[test]
    fn test_run_once_verbose() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "42;").unwrap();

        let config = WatchConfig {
            clear_screen: false,
            verbose: true,
            ..Default::default()
        };

        run_once(temp_file.path(), &config);
    }
}
