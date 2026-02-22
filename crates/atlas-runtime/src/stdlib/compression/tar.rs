//! Tar archive creation and extraction
//!
//! Provides tar and tar.gz archive utilities using the tar library for package distribution and backups.

use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
use crate::stdlib::collections::hashmap::AtlasHashMap;
use crate::value::{RuntimeError, Value};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tar::{Archive, Builder, EntryType};

// ============================================================================
// Constants
// ============================================================================

/// Default compression level for tar.gz (6 = good balance)
const DEFAULT_COMPRESSION_LEVEL: u32 = 6;

// ============================================================================
// Tar Creation Functions
// ============================================================================

/// Create a tar archive from files/directories
///
/// Takes a list of source paths and an output tar path.
/// Preserves file metadata (permissions, timestamps).
pub fn create_tar<P: AsRef<Path>>(
    sources: &[PathBuf],
    output: P,
    span: Span,
) -> Result<(), RuntimeError> {
    let file = File::create(output.as_ref()).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create tar file: {}", e),
        span,
    })?;

    let mut builder = Builder::new(file);

    for source in sources {
        if !source.exists() {
            return Err(RuntimeError::IoError {
                message: format!("Source path does not exist: {}", source.display()),
                span,
            });
        }

        let name = source.file_name().ok_or_else(|| RuntimeError::IoError {
            message: "Invalid source path".to_string(),
            span,
        })?;

        if source.is_dir() {
            builder
                .append_dir_all(name, source)
                .map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to add directory to tar: {}", e),
                    span,
                })?;
        } else {
            builder
                .append_path_with_name(source, name)
                .map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to add file to tar: {}", e),
                    span,
                })?;
        }
    }

    builder.finish().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to finalize tar archive: {}", e),
        span,
    })?;

    Ok(())
}

/// Create a tar.gz (gzip-compressed tar) archive
///
/// Combines tar creation with gzip compression.
pub fn create_tar_gz<P: AsRef<Path>>(
    sources: &[PathBuf],
    output: P,
    level: u32,
    span: Span,
) -> Result<(), RuntimeError> {
    // Validate compression level
    if level > 9 {
        return Err(RuntimeError::IoError {
            message: format!("Invalid compression level {}: must be 0-9", level),
            span,
        });
    }

    let file = File::create(output.as_ref()).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create tar.gz file: {}", e),
        span,
    })?;

    let encoder = GzEncoder::new(file, Compression::new(level));
    let mut builder = Builder::new(encoder);

    for source in sources {
        if !source.exists() {
            return Err(RuntimeError::IoError {
                message: format!("Source path does not exist: {}", source.display()),
                span,
            });
        }

        let name = source.file_name().ok_or_else(|| RuntimeError::IoError {
            message: "Invalid source path".to_string(),
            span,
        })?;

        if source.is_dir() {
            builder
                .append_dir_all(name, source)
                .map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to add directory to tar.gz: {}", e),
                    span,
                })?;
        } else {
            builder
                .append_path_with_name(source, name)
                .map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to add file to tar.gz: {}", e),
                    span,
                })?;
        }
    }

    let encoder = builder.into_inner().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to finalize tar.gz archive: {}", e),
        span,
    })?;

    encoder.finish().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to complete gzip compression: {}", e),
        span,
    })?;

    Ok(())
}

// ============================================================================
// Tar Extraction Functions
// ============================================================================

/// Extract a tar archive to a directory
///
/// Takes a tar file path and output directory, extracts all files.
/// Preserves file metadata. Prevents path traversal attacks.
pub fn extract_tar<P: AsRef<Path>, Q: AsRef<Path>>(
    tar_path: P,
    output_dir: Q,
    span: Span,
) -> Result<Vec<PathBuf>, RuntimeError> {
    let file = File::open(tar_path.as_ref()).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open tar file: {}", e),
        span,
    })?;

    let mut archive = Archive::new(file);
    let output_path = output_dir.as_ref();

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create output directory: {}", e),
        span,
    })?;

    let mut extracted_files = Vec::new();

    for entry in archive.entries().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read tar entries: {}", e),
        span,
    })? {
        let mut entry = entry.map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read tar entry: {}", e),
            span,
        })?;

        // Get entry path and validate for path traversal
        let entry_path = entry
            .path()
            .map_err(|e| RuntimeError::IoError {
                message: format!("Invalid entry path: {}", e),
                span,
            })?
            .to_path_buf();

        // Validate path doesn't escape output directory (prevent path traversal)
        let full_path = output_path.join(&entry_path);
        if !full_path.starts_with(output_path) {
            return Err(RuntimeError::IoError {
                message: format!("Path traversal detected: {}", entry_path.display()),
                span,
            });
        }

        // Unpack entry
        entry
            .unpack_in(output_path)
            .map_err(|e| RuntimeError::IoError {
                message: format!("Failed to extract {}: {}", entry_path.display(), e),
                span,
            })?;

        extracted_files.push(full_path);
    }

    Ok(extracted_files)
}

/// Extract a tar.gz (gzip-compressed tar) archive
///
/// Auto-decompresses and extracts in one operation.
pub fn extract_tar_gz<P: AsRef<Path>, Q: AsRef<Path>>(
    tar_gz_path: P,
    output_dir: Q,
    span: Span,
) -> Result<Vec<PathBuf>, RuntimeError> {
    let file = File::open(tar_gz_path.as_ref()).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open tar.gz file: {}", e),
        span,
    })?;

    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let output_path = output_dir.as_ref();

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create output directory: {}", e),
        span,
    })?;

    let mut extracted_files = Vec::new();

    for entry in archive.entries().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read tar.gz entries: {}", e),
        span,
    })? {
        let mut entry = entry.map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read tar.gz entry: {}", e),
            span,
        })?;

        // Get entry path and validate for path traversal
        let entry_path = entry
            .path()
            .map_err(|e| RuntimeError::IoError {
                message: format!("Invalid entry path: {}", e),
                span,
            })?
            .to_path_buf();

        // Validate path doesn't escape output directory
        let full_path = output_path.join(&entry_path);
        if !full_path.starts_with(output_path) {
            return Err(RuntimeError::IoError {
                message: format!("Path traversal detected: {}", entry_path.display()),
                span,
            });
        }

        // Unpack entry
        entry
            .unpack_in(output_path)
            .map_err(|e| RuntimeError::IoError {
                message: format!("Failed to extract {}: {}", entry_path.display(), e),
                span,
            })?;

        extracted_files.push(full_path);
    }

    Ok(extracted_files)
}

// ============================================================================
// Tar Utility Functions
// ============================================================================

/// List contents of a tar archive
///
/// Returns list of file paths and metadata.
pub fn list_tar<P: AsRef<Path>>(tar_path: P, span: Span) -> Result<Vec<TarEntry>, RuntimeError> {
    let file = File::open(tar_path.as_ref()).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open tar file: {}", e),
        span,
    })?;

    let mut archive = Archive::new(file);
    let mut entries = Vec::new();

    for entry in archive.entries().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read tar entries: {}", e),
        span,
    })? {
        let entry = entry.map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read tar entry: {}", e),
            span,
        })?;

        let path = entry
            .path()
            .map_err(|e| RuntimeError::IoError {
                message: format!("Invalid entry path: {}", e),
                span,
            })?
            .to_path_buf();

        let header = entry.header();
        let size = header.size().map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read entry size: {}", e),
            span,
        })?;

        let entry_type = match header.entry_type() {
            EntryType::Regular => "file",
            EntryType::Directory => "directory",
            EntryType::Symlink => "symlink",
            _ => "other",
        };

        entries.push(TarEntry {
            path,
            size,
            entry_type: entry_type.to_string(),
        });
    }

    Ok(entries)
}

/// Tar entry metadata
#[derive(Debug, Clone)]
pub struct TarEntry {
    pub path: PathBuf,
    pub size: u64,
    pub entry_type: String,
}

/// Check if file exists in tar archive
pub fn tar_contains<P: AsRef<Path>, Q: AsRef<Path>>(
    tar_path: P,
    file_path: Q,
    span: Span,
) -> Result<bool, RuntimeError> {
    let entries = list_tar(tar_path, span)?;
    let search_path = file_path.as_ref();

    Ok(entries.iter().any(|e| e.path == search_path))
}

// ============================================================================
// Atlas Stdlib API Functions
// ============================================================================

/// tarCreate(sources: array<string>, output: string) -> null
///
/// Create a tar archive from files/directories.
pub fn tar_create(sources: &Value, output: &Value, span: Span) -> Result<Value, RuntimeError> {
    // Extract source paths
    let source_paths = match sources {
        Value::Array(arr) => {
            let arr_slice = arr.as_slice();
            let mut paths = Vec::new();
            for val in arr_slice.iter() {
                match val {
                    Value::String(s) => paths.push(PathBuf::from(s.as_ref())),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Sources array must contain only strings".to_string(),
                            span,
                        });
                    }
                }
            }
            paths
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Sources must be an array of strings".to_string(),
                span,
            });
        }
    };

    // Extract output path
    let output_path = match output {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Output must be a string".to_string(),
                span,
            });
        }
    };

    create_tar(&source_paths, output_path, span)?;
    Ok(Value::Null)
}

/// tarCreateGz(sources: array<string>, output: string, level?: number) -> null
///
/// Create a tar.gz (gzip-compressed) archive. Level 0-9 (default 6).
pub fn tar_create_gz(
    sources: &Value,
    output: &Value,
    level_opt: Option<&Value>,
    span: Span,
) -> Result<Value, RuntimeError> {
    // Extract compression level (default 6)
    let level = if let Some(level_val) = level_opt {
        match level_val {
            Value::Number(n) => {
                let l = *n as u32;
                if l > 9 {
                    return Err(RuntimeError::IoError {
                        message: format!("Compression level must be 0-9, got {}", l),
                        span,
                    });
                }
                l
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Compression level must be a number".to_string(),
                    span,
                });
            }
        }
    } else {
        DEFAULT_COMPRESSION_LEVEL
    };

    // Extract source paths
    let source_paths = match sources {
        Value::Array(arr) => {
            let arr_slice = arr.as_slice();
            let mut paths = Vec::new();
            for val in arr_slice.iter() {
                match val {
                    Value::String(s) => paths.push(PathBuf::from(s.as_ref())),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Sources array must contain only strings".to_string(),
                            span,
                        });
                    }
                }
            }
            paths
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Sources must be an array of strings".to_string(),
                span,
            });
        }
    };

    // Extract output path
    let output_path = match output {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Output must be a string".to_string(),
                span,
            });
        }
    };

    create_tar_gz(&source_paths, output_path, level, span)?;
    Ok(Value::Null)
}

/// tarExtract(tarPath: string, outputDir: string) -> array<string>
///
/// Extract a tar archive. Returns list of extracted files.
pub fn tar_extract(
    tar_path: &Value,
    output_dir: &Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    let tar = match tar_path {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Tar path must be a string".to_string(),
                span,
            });
        }
    };

    let output = match output_dir {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Output directory must be a string".to_string(),
                span,
            });
        }
    };

    let extracted = extract_tar(tar, output, span)?;

    // Convert to Atlas string array
    let result: Vec<Value> = extracted
        .iter()
        .map(|p| Value::string(p.to_string_lossy().to_string()))
        .collect();
    Ok(Value::array(result))
}

/// tarExtractGz(tarGzPath: string, outputDir: string) -> array<string>
///
/// Extract a tar.gz archive. Returns list of extracted files.
pub fn tar_extract_gz(
    tar_gz_path: &Value,
    output_dir: &Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    let tar_gz = match tar_gz_path {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Tar.gz path must be a string".to_string(),
                span,
            });
        }
    };

    let output = match output_dir {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Output directory must be a string".to_string(),
                span,
            });
        }
    };

    let extracted = extract_tar_gz(tar_gz, output, span)?;

    // Convert to Atlas string array
    let result: Vec<Value> = extracted
        .iter()
        .map(|p| Value::string(p.to_string_lossy().to_string()))
        .collect();
    Ok(Value::array(result))
}

/// tarList(tarPath: string) -> array<HashMap>
///
/// List tar archive contents. Returns array of entry objects.
pub fn tar_list(tar_path: &Value, span: Span) -> Result<Value, RuntimeError> {
    let tar = match tar_path {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Tar path must be a string".to_string(),
                span,
            });
        }
    };

    let entries = list_tar(tar, span)?;

    // Convert to Atlas array of HashMaps
    let result: Vec<Value> = entries
        .iter()
        .map(|entry| {
            let mut map = AtlasHashMap::new();
            map.insert(
                HashKey::String(Arc::new("path".to_string())),
                Value::string(entry.path.to_string_lossy().to_string()),
            );
            map.insert(
                HashKey::String(Arc::new("size".to_string())),
                Value::Number(entry.size as f64),
            );
            map.insert(
                HashKey::String(Arc::new("type".to_string())),
                Value::string(entry.entry_type.clone()),
            );
            Value::HashMap(crate::value::ValueHashMap::from_atlas(map))
        })
        .collect();

    Ok(Value::array(result))
}

/// tarContains(tarPath: string, filePath: string) -> bool
///
/// Check if file exists in tar archive.
pub fn tar_contains_file(
    tar_path: &Value,
    file_path: &Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    let tar = match tar_path {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Tar path must be a string".to_string(),
                span,
            });
        }
    };

    let file = match file_path {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "File path must be a string".to_string(),
                span,
            });
        }
    };

    let contains = tar_contains(tar, file, span)?;
    Ok(Value::Bool(contains))
}
