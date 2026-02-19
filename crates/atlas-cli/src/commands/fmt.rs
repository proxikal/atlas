//! Atlas code formatter CLI command

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use atlas_formatter::{FormatConfig, FormatResult};

/// Arguments for the fmt command
pub struct FmtArgs {
    pub files: Vec<String>,
    pub check: bool,
    pub indent_size: Option<usize>,
    pub max_width: Option<usize>,
    pub trailing_commas: Option<bool>,
}

/// Run the fmt command
pub fn run(args: FmtArgs) -> Result<()> {
    let mut config = FormatConfig::default();

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
        eprintln!("No Atlas files found");
        return Ok(());
    }

    let mut had_errors = false;
    let mut unformatted_count = 0;
    let mut formatted_count = 0;

    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read {}", file.display()))?;

        let result = atlas_formatter::format_source_with_config(&source, &config);

        match result {
            FormatResult::Ok(formatted) => {
                if args.check {
                    if formatted != source {
                        eprintln!("Would reformat: {}", file.display());
                        unformatted_count += 1;
                    }
                } else if formatted != source {
                    std::fs::write(file, &formatted)
                        .with_context(|| format!("Failed to write {}", file.display()))?;
                    eprintln!("Formatted: {}", file.display());
                    formatted_count += 1;
                }
            }
            FormatResult::ParseError(errors) => {
                eprintln!("Error in {}: {}", file.display(), errors.join(", "));
                had_errors = true;
            }
        }
    }

    if args.check {
        if unformatted_count > 0 {
            eprintln!("{} file(s) would be reformatted", unformatted_count);
            std::process::exit(1);
        } else {
            eprintln!("All {} file(s) are formatted correctly", files.len());
        }
    } else if formatted_count > 0 {
        eprintln!("Formatted {} file(s)", formatted_count);
    }

    if had_errors {
        std::process::exit(1);
    }

    Ok(())
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
