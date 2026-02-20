//! Atlas code formatter CLI command

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use atlas_formatter::{FormatConfig, FormatResult};

/// Verbosity level for formatter output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Suppress all non-error output
    Quiet,
    /// Normal output (default)
    #[default]
    Normal,
    /// Detailed output with timing and file info
    Verbose,
}

/// Arguments for the fmt command
pub struct FmtArgs {
    pub files: Vec<String>,
    pub check: bool,
    pub write: bool,
    pub config_path: Option<PathBuf>,
    pub indent_size: Option<usize>,
    pub max_width: Option<usize>,
    pub trailing_commas: Option<bool>,
    pub verbosity: Verbosity,
}

/// Run the fmt command
pub fn run(args: FmtArgs) -> Result<()> {
    let start_time = std::time::Instant::now();

    // Load config from file if specified, then apply CLI overrides
    let mut config = load_config(&args.config_path)?;

    // CLI arguments override config file settings
    if let Some(size) = args.indent_size {
        config.indent_size = size;
    }
    if let Some(width) = args.max_width {
        config.max_width = width;
    }
    if let Some(tc) = args.trailing_commas {
        config.trailing_commas = tc;
    }

    // Collect all .at files from arguments
    let files = collect_files(&args.files)?;

    if files.is_empty() {
        if args.verbosity != Verbosity::Quiet {
            eprintln!("No Atlas files found");
        }
        return Ok(());
    }

    // Verbose: show config and file count
    if args.verbosity == Verbosity::Verbose {
        eprintln!("Configuration:");
        eprintln!("  indent_size: {}", config.indent_size);
        eprintln!("  max_width: {}", config.max_width);
        eprintln!("  trailing_commas: {}", config.trailing_commas);
        if let Some(ref path) = args.config_path {
            eprintln!("  config_file: {}", path.display());
        }
        eprintln!("Processing {} file(s)...", files.len());
        eprintln!();
    }

    let mut had_errors = false;
    let mut unformatted_count = 0;
    let mut formatted_count = 0;
    let mut unchanged_count = 0;
    let total_files = files.len();

    for (index, file) in files.iter().enumerate() {
        let file_start = std::time::Instant::now();

        // Progress indication for multiple files (normal verbosity)
        if args.verbosity == Verbosity::Verbose && total_files > 1 {
            eprint!("[{}/{}] {} ... ", index + 1, total_files, file.display());
        }

        let source = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read {}", file.display()))?;

        let result = atlas_formatter::format_source_with_config(&source, &config);

        match result {
            FormatResult::Ok(formatted) => {
                let changed = formatted != source;

                if args.check {
                    if changed {
                        if args.verbosity == Verbosity::Verbose {
                            eprintln!("would reformat");
                        } else if args.verbosity == Verbosity::Normal {
                            eprintln!("Would reformat: {}", file.display());
                        }
                        unformatted_count += 1;
                    } else {
                        unchanged_count += 1;
                        if args.verbosity == Verbosity::Verbose {
                            eprintln!("ok");
                        }
                    }
                } else if changed {
                    // Write mode: --write flag or default behavior (no --check)
                    if args.write || !args.check {
                        std::fs::write(file, &formatted)
                            .with_context(|| format!("Failed to write {}", file.display()))?;
                    }

                    if args.verbosity == Verbosity::Verbose {
                        let elapsed = file_start.elapsed();
                        eprintln!("formatted ({:.2}ms)", elapsed.as_secs_f64() * 1000.0);
                    } else if args.verbosity == Verbosity::Normal {
                        eprintln!("Formatted: {}", file.display());
                    }
                    formatted_count += 1;
                } else {
                    unchanged_count += 1;
                    if args.verbosity == Verbosity::Verbose {
                        eprintln!("unchanged");
                    }
                }
            }
            FormatResult::ParseError(errors) => {
                if args.verbosity == Verbosity::Verbose {
                    eprintln!("ERROR");
                }
                eprintln!("Error in {}: {}", file.display(), errors.join(", "));
                had_errors = true;
            }
        }
    }

    // Summary output
    let total_elapsed = start_time.elapsed();

    if args.check {
        if unformatted_count > 0 {
            if args.verbosity != Verbosity::Quiet {
                eprintln!();
                eprintln!("{} file(s) would be reformatted", unformatted_count);
            }
            std::process::exit(1);
        } else if args.verbosity != Verbosity::Quiet {
            eprintln!("All {} file(s) are formatted correctly", files.len());
        }
    } else if args.verbosity != Verbosity::Quiet
        && (formatted_count > 0 || args.verbosity == Verbosity::Verbose)
    {
        eprintln!();
        if formatted_count > 0 {
            eprintln!("Formatted {} file(s)", formatted_count);
        }
        if args.verbosity == Verbosity::Verbose {
            eprintln!(
                "Summary: {} formatted, {} unchanged, {} errors",
                formatted_count,
                unchanged_count,
                if had_errors { 1 } else { 0 }
            );
            eprintln!("Total time: {:.2}ms", total_elapsed.as_secs_f64() * 1000.0);
        }
    }

    if had_errors {
        std::process::exit(1);
    }

    Ok(())
}

/// Load format configuration from a file path or use defaults
fn load_config(config_path: &Option<PathBuf>) -> Result<FormatConfig> {
    if let Some(path) = config_path {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        // Simple TOML-like parsing for format config
        // Expected format:
        // indent_size = 4
        // max_width = 100
        // trailing_commas = true
        let mut config = FormatConfig::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "indent_size" => {
                        config.indent_size = value
                            .parse()
                            .with_context(|| format!("Invalid indent_size: {}", value))?;
                    }
                    "max_width" => {
                        config.max_width = value
                            .parse()
                            .with_context(|| format!("Invalid max_width: {}", value))?;
                    }
                    "trailing_commas" => {
                        config.trailing_commas = value
                            .parse()
                            .with_context(|| format!("Invalid trailing_commas: {}", value))?;
                    }
                    _ => {
                        // Ignore unknown keys for forward compatibility
                    }
                }
            }
        }

        Ok(config)
    } else {
        Ok(FormatConfig::default())
    }
}

/// Collect Atlas source files from paths (handles directories recursively)
fn collect_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_dir() {
            collect_files_recursive(path, &mut files)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "at" || ext == "atlas")
        {
            files.push(path.to_path_buf());
        } else {
            // Accept any file explicitly passed
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "at" || ext == "atlas")
        {
            files.push(path);
        }
    }
    Ok(())
}
