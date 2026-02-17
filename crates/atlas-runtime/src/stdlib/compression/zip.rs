//! Zip archive creation and extraction
//!
//! Provides zip archive utilities using deflate compression for cross-platform archive management.

use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
use crate::stdlib::collections::hashmap::AtlasHashMap;
use crate::value::{RuntimeError, Value};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

// ============================================================================
// Constants
// ============================================================================

/// Default compression level (6 = good balance between speed and size)
const DEFAULT_COMPRESSION_LEVEL: i64 = 6;

// ============================================================================
// Internal types
// ============================================================================

/// Metadata for a single zip entry
#[derive(Debug, Clone)]
pub struct ZipEntry {
    pub name: String,
    pub size: u64,
    pub compressed_size: u64,
    pub is_dir: bool,
    pub method: String,
}

// ============================================================================
// Internal core helpers
// ============================================================================

/// Recursively walk a directory and collect all (absolute_path, archive_name) pairs.
///
/// `base` is the parent of the source directory — used to strip the leading prefix
/// so that archive entries look like `mydir/`, `mydir/file.txt`, etc.
fn walk_dir_for_zip(
    dir: &Path,
    base: &Path,
    out: &mut Vec<(PathBuf, String)>,
) -> Result<(), io::Error> {
    // Add the directory entry itself
    let archive_name = {
        let rel = dir.strip_prefix(base).map_err(io::Error::other)?;
        let mut s = rel.to_string_lossy().replace('\\', "/");
        if !s.ends_with('/') {
            s.push('/');
        }
        s
    };
    out.push((dir.to_path_buf(), archive_name));

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir_for_zip(&path, base, out)?;
        } else {
            let rel = path.strip_prefix(base).map_err(io::Error::other)?;
            let archive_name = rel.to_string_lossy().replace('\\', "/");
            out.push((path, archive_name));
        }
    }
    Ok(())
}

/// Build a `FileOptions` from a compression level (0 = store, 1-9 = deflate).
fn file_options(level: i64) -> FileOptions {
    if level == 0 {
        FileOptions::default().compression_method(CompressionMethod::Stored)
    } else {
        FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(level as i32))
    }
}

/// Validate that an archive-internal path doesn't escape the output directory.
///
/// Returns `Ok(resolved_path)` if safe, `Err` if path traversal is detected.
fn safe_output_path(output_dir: &Path, entry_name: &str) -> Result<PathBuf, ()> {
    let candidate = output_dir.join(entry_name);
    // Normalise by stripping `.` / `..` components (without requiring the path to exist)
    let mut components = Vec::new();
    for component in candidate.components() {
        use std::path::Component;
        match component {
            Component::ParentDir => {
                if components.pop().is_none() {
                    return Err(()); // path traversal detected
                }
            }
            Component::CurDir => {}
            other => components.push(other),
        }
    }
    let resolved: PathBuf = components.iter().collect();
    // Must still start with the output dir
    if !resolved.starts_with(output_dir) {
        return Err(());
    }
    Ok(resolved)
}

// ============================================================================
// Core Rust-level functions (called by Atlas API wrappers)
// ============================================================================

/// Create a zip archive from a list of source paths.
///
/// - `level == 0` → STORE (no compression)
/// - `level 1-9` → DEFLATE with that level
/// - `comment` → optional archive-level comment
pub fn create_zip(
    sources: &[PathBuf],
    output: &Path,
    level: i64,
    comment: Option<&str>,
    span: Span,
) -> Result<(), RuntimeError> {
    if !(0..=9).contains(&level) {
        return Err(RuntimeError::IoError {
            message: format!("Compression level must be 0-9, got {}", level),
            span,
        });
    }

    let file = File::create(output).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create zip file: {}", e),
        span,
    })?;

    let mut zip = ZipWriter::new(file);

    if let Some(c) = comment {
        zip.set_comment(c);
    }

    let options = file_options(level);

    for source in sources {
        if !source.exists() {
            return Err(RuntimeError::IoError {
                message: format!("Source path does not exist: {}", source.display()),
                span,
            });
        }

        if source.is_dir() {
            let base = source.parent().unwrap_or(source);
            let mut entries: Vec<(PathBuf, String)> = Vec::new();
            walk_dir_for_zip(source, base, &mut entries).map_err(|e| RuntimeError::IoError {
                message: format!("Failed to walk directory {}: {}", source.display(), e),
                span,
            })?;

            for (path, name) in &entries {
                if path.is_dir() {
                    zip.add_directory(name, options)
                        .map_err(|e| RuntimeError::IoError {
                            message: format!("Failed to add directory {} to zip: {}", name, e),
                            span,
                        })?;
                } else {
                    zip.start_file(name, options)
                        .map_err(|e| RuntimeError::IoError {
                            message: format!("Failed to start file {} in zip: {}", name, e),
                            span,
                        })?;

                    let mut f = File::open(path).map_err(|e| RuntimeError::IoError {
                        message: format!("Failed to open {}: {}", path.display(), e),
                        span,
                    })?;
                    io::copy(&mut f, &mut zip).map_err(|e| RuntimeError::IoError {
                        message: format!("Failed to write {} to zip: {}", name, e),
                        span,
                    })?;
                }
            }
        } else {
            let name = source
                .file_name()
                .ok_or_else(|| RuntimeError::IoError {
                    message: format!("Invalid source path: {}", source.display()),
                    span,
                })?
                .to_string_lossy()
                .replace('\\', "/");

            zip.start_file(&name, options)
                .map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to start file {} in zip: {}", name, e),
                    span,
                })?;

            let mut f = File::open(source).map_err(|e| RuntimeError::IoError {
                message: format!("Failed to open {}: {}", source.display(), e),
                span,
            })?;
            io::copy(&mut f, &mut zip).map_err(|e| RuntimeError::IoError {
                message: format!("Failed to write {} to zip: {}", name, e),
                span,
            })?;
        }
    }

    zip.finish().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to finalize zip archive: {}", e),
        span,
    })?;

    Ok(())
}

/// Extract all files from a zip archive into `output_dir`.
///
/// Returns a list of extracted file paths.
/// Prevents path traversal attacks.
pub fn extract_zip(
    zip_path: &Path,
    output_dir: &Path,
    span: Span,
) -> Result<Vec<PathBuf>, RuntimeError> {
    let file = File::open(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open zip file: {}", e),
        span,
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read zip archive: {}", e),
        span,
    })?;

    fs::create_dir_all(output_dir).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create output directory: {}", e),
        span,
    })?;

    let mut extracted = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read zip entry {}: {}", i, e),
            span,
        })?;

        let name = entry.name().to_owned();

        // Validate path safety
        let out_path = safe_output_path(output_dir, &name).map_err(|_| RuntimeError::IoError {
            message: format!("Path traversal detected: {}", name),
            span,
        })?;

        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| RuntimeError::IoError {
                message: format!("Failed to create directory {}: {}", out_path.display(), e),
                span,
            })?;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to create parent directory: {}", e),
                    span,
                })?;
            }

            let mut out_file = File::create(&out_path).map_err(|e| RuntimeError::IoError {
                message: format!("Failed to create {}: {}", out_path.display(), e),
                span,
            })?;

            io::copy(&mut entry, &mut out_file).map_err(|e| RuntimeError::IoError {
                message: format!("Failed to extract {}: {}", name, e),
                span,
            })?;

            extracted.push(out_path);
        }
    }

    Ok(extracted)
}

/// Extract only the specified files from a zip archive.
///
/// `files` contains the archive-internal names to extract.
pub fn extract_zip_files(
    zip_path: &Path,
    output_dir: &Path,
    files: &[String],
    span: Span,
) -> Result<Vec<PathBuf>, RuntimeError> {
    let file = File::open(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open zip file: {}", e),
        span,
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read zip archive: {}", e),
        span,
    })?;

    fs::create_dir_all(output_dir).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create output directory: {}", e),
        span,
    })?;

    let mut extracted = Vec::new();

    for target in files {
        let mut entry = archive.by_name(target).map_err(|e| RuntimeError::IoError {
            message: format!("File '{}' not found in zip: {}", target, e),
            span,
        })?;

        let out_path = safe_output_path(output_dir, target).map_err(|_| RuntimeError::IoError {
            message: format!("Path traversal detected: {}", target),
            span,
        })?;

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| RuntimeError::IoError {
                message: format!("Failed to create parent directory: {}", e),
                span,
            })?;
        }

        let mut out_file = File::create(&out_path).map_err(|e| RuntimeError::IoError {
            message: format!("Failed to create {}: {}", out_path.display(), e),
            span,
        })?;

        io::copy(&mut entry, &mut out_file).map_err(|e| RuntimeError::IoError {
            message: format!("Failed to extract {}: {}", target, e),
            span,
        })?;

        extracted.push(out_path);
    }

    Ok(extracted)
}

/// List all entries in a zip archive.
pub fn list_zip(zip_path: &Path, span: Span) -> Result<Vec<ZipEntry>, RuntimeError> {
    let file = File::open(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open zip file: {}", e),
        span,
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read zip archive: {}", e),
        span,
    })?;

    let mut entries = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read zip entry {}: {}", i, e),
            span,
        })?;

        let method = match entry.compression() {
            CompressionMethod::Stored => "stored",
            CompressionMethod::Deflated => "deflated",
            _ => "unknown",
        };

        entries.push(ZipEntry {
            name: entry.name().to_owned(),
            size: entry.size(),
            compressed_size: entry.compressed_size(),
            is_dir: entry.is_dir(),
            method: method.to_string(),
        });
    }

    Ok(entries)
}

/// Check whether a named entry exists in a zip archive.
pub fn zip_contains(zip_path: &Path, entry_name: &str, span: Span) -> Result<bool, RuntimeError> {
    let file = File::open(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open zip file: {}", e),
        span,
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read zip archive: {}", e),
        span,
    })?;

    // Materialise the bool before `archive` is dropped so the ZipFile<'_>
    // borrow ends before the end of this block.
    let found = archive.by_name(entry_name).is_ok();
    Ok(found)
}

/// Compute the overall compression ratio of a zip archive.
///
/// Returns `compressed_total / uncompressed_total`. Returns 1.0 if archive is empty.
pub fn zip_ratio(zip_path: &Path, span: Span) -> Result<f64, RuntimeError> {
    let file = File::open(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open zip file: {}", e),
        span,
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read zip archive: {}", e),
        span,
    })?;

    let mut total_size: u64 = 0;
    let mut total_compressed: u64 = 0;

    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read zip entry {}: {}", i, e),
            span,
        })?;
        total_size += entry.size();
        total_compressed += entry.compressed_size();
    }

    if total_size == 0 {
        return Ok(1.0);
    }

    Ok(total_compressed as f64 / total_size as f64)
}

/// Add a single file to an existing zip archive.
///
/// Reads all existing entries (without re-compressing) and appends the new file.
pub fn zip_add_file(
    zip_path: &Path,
    file_to_add: &Path,
    entry_name: &str,
    level: i64,
    span: Span,
) -> Result<(), RuntimeError> {
    if !file_to_add.exists() {
        return Err(RuntimeError::IoError {
            message: format!("File to add does not exist: {}", file_to_add.display()),
            span,
        });
    }

    // Read the existing zip into a buffer
    let existing_bytes = fs::read(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read existing zip: {}", e),
        span,
    })?;

    // Build new zip: copy existing entries + new file
    let out_file = File::create(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open zip for writing: {}", e),
        span,
    })?;

    let mut new_zip = ZipWriter::new(out_file);

    {
        let cursor = std::io::Cursor::new(&existing_bytes);
        let mut old_archive = ZipArchive::new(cursor).map_err(|e| RuntimeError::IoError {
            message: format!("Failed to parse existing zip: {}", e),
            span,
        })?;

        // Preserve archive comment
        let comment = old_archive.comment().to_vec();
        if !comment.is_empty() {
            if let Ok(s) = std::str::from_utf8(&comment) {
                new_zip.set_comment(s);
            }
        }

        for i in 0..old_archive.len() {
            let entry = old_archive
                .by_index_raw(i)
                .map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to read existing entry {}: {}", i, e),
                    span,
                })?;

            new_zip
                .raw_copy_file(entry)
                .map_err(|e| RuntimeError::IoError {
                    message: format!("Failed to copy existing entry: {}", e),
                    span,
                })?;
        }
    }

    // Add new file
    let add_options = file_options(level);
    let clean_name = entry_name.replace('\\', "/");
    new_zip
        .start_file(&clean_name, add_options)
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to start new entry {}: {}", clean_name, e),
            span,
        })?;

    let mut f = File::open(file_to_add).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open file to add: {}", e),
        span,
    })?;
    io::copy(&mut f, &mut new_zip).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to write new entry: {}", e),
        span,
    })?;

    new_zip.finish().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to finalize updated zip: {}", e),
        span,
    })?;

    Ok(())
}

/// Validate that a file is a parseable zip archive.
///
/// Returns `true` if the archive is valid, `false` otherwise.
pub fn zip_validate(zip_path: &Path) -> bool {
    if let Ok(file) = File::open(zip_path) {
        ZipArchive::new(file).is_ok()
    } else {
        false
    }
}

/// Read the archive-level comment from a zip file.
///
/// Returns an empty string if there is no comment.
pub fn zip_read_comment(zip_path: &Path, span: Span) -> Result<String, RuntimeError> {
    let file = File::open(zip_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to open zip file: {}", e),
        span,
    })?;

    let archive = ZipArchive::new(file).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read zip archive: {}", e),
        span,
    })?;

    let comment_bytes = archive.comment();
    let comment = std::str::from_utf8(comment_bytes).unwrap_or("").to_owned();
    Ok(comment)
}

// ============================================================================
// Atlas Stdlib API Functions
// ============================================================================

/// zipCreate(sources: array<string>, output: string, level?: number) -> null
///
/// Create a zip archive. Level 0 = store, 1-9 = deflate (default 6).
pub fn zip_create(
    sources: &Value,
    output: &Value,
    level_opt: Option<&Value>,
    span: Span,
) -> Result<Value, RuntimeError> {
    let level = extract_level(level_opt, span)?;
    let source_paths = extract_string_array(sources, "sources", span)?;
    let output_path = extract_str(output, "output", span)?;

    create_zip(&source_paths, Path::new(output_path), level, None, span)?;
    Ok(Value::Null)
}

/// zipCreateWithComment(sources: array<string>, output: string, comment: string, level?: number) -> null
///
/// Create a zip archive with an archive-level comment.
pub fn zip_create_with_comment(
    sources: &Value,
    output: &Value,
    comment: &Value,
    level_opt: Option<&Value>,
    span: Span,
) -> Result<Value, RuntimeError> {
    let level = extract_level(level_opt, span)?;
    let source_paths = extract_string_array(sources, "sources", span)?;
    let output_path = extract_str(output, "output", span)?;
    let comment_str = extract_str(comment, "comment", span)?;

    create_zip(
        &source_paths,
        Path::new(output_path),
        level,
        Some(comment_str),
        span,
    )?;
    Ok(Value::Null)
}

/// zipExtract(zipPath: string, outputDir: string) -> array<string>
///
/// Extract all files from a zip archive. Returns list of extracted file paths.
pub fn zip_extract(
    zip_path: &Value,
    output_dir: &Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    let out = extract_str(output_dir, "outputDir", span)?;

    let extracted = extract_zip(Path::new(zip), Path::new(out), span)?;
    let result: Vec<Value> = extracted
        .iter()
        .map(|p| Value::string(p.to_string_lossy().to_string()))
        .collect();
    Ok(Value::array(result))
}

/// zipExtractFiles(zipPath: string, outputDir: string, files: array<string>) -> array<string>
///
/// Extract specific named files from a zip archive.
pub fn zip_extract_files(
    zip_path: &Value,
    output_dir: &Value,
    files: &Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    let out = extract_str(output_dir, "outputDir", span)?;
    let file_names: Vec<String> = extract_string_array(files, "files", span)?
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let extracted = extract_zip_files(Path::new(zip), Path::new(out), &file_names, span)?;
    let result: Vec<Value> = extracted
        .iter()
        .map(|p| Value::string(p.to_string_lossy().to_string()))
        .collect();
    Ok(Value::array(result))
}

/// zipList(zipPath: string) -> array<HashMap>
///
/// List zip archive contents. Each entry is a HashMap with keys:
/// "name", "size", "compressedSize", "isDir", "method".
pub fn zip_list(zip_path: &Value, span: Span) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    let entries = list_zip(Path::new(zip), span)?;

    let result: Vec<Value> = entries
        .iter()
        .map(|entry| {
            let mut map = AtlasHashMap::new();
            map.insert(
                HashKey::String(Arc::new("name".to_string())),
                Value::string(entry.name.clone()),
            );
            map.insert(
                HashKey::String(Arc::new("size".to_string())),
                Value::Number(entry.size as f64),
            );
            map.insert(
                HashKey::String(Arc::new("compressedSize".to_string())),
                Value::Number(entry.compressed_size as f64),
            );
            map.insert(
                HashKey::String(Arc::new("isDir".to_string())),
                Value::Bool(entry.is_dir),
            );
            map.insert(
                HashKey::String(Arc::new("method".to_string())),
                Value::string(entry.method.clone()),
            );
            Value::HashMap(Arc::new(Mutex::new(map)))
        })
        .collect();

    Ok(Value::array(result))
}

/// zipContains(zipPath: string, entryName: string) -> bool
///
/// Check if a named entry exists in the zip archive.
pub fn zip_contains_file(
    zip_path: &Value,
    entry_name: &Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    let name = extract_str(entry_name, "entryName", span)?;
    let found = zip_contains(Path::new(zip), name, span)?;
    Ok(Value::Bool(found))
}

/// zipCompressionRatio(zipPath: string) -> number
///
/// Return the overall compression ratio (compressedBytes / originalBytes).
/// Returns 1.0 for an empty archive.
pub fn zip_compression_ratio(zip_path: &Value, span: Span) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    let ratio = zip_ratio(Path::new(zip), span)?;
    Ok(Value::Number(ratio))
}

/// zipAddFile(zipPath: string, filePath: string, entryName?: string, level?: number) -> null
///
/// Add a file to an existing zip archive. `entryName` defaults to the file's basename.
pub fn zip_add_file_fn(
    zip_path: &Value,
    file_path: &Value,
    entry_name_opt: Option<&Value>,
    level_opt: Option<&Value>,
    span: Span,
) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    let file = extract_str(file_path, "filePath", span)?;

    let default_name;
    let entry_name = if let Some(name_val) = entry_name_opt {
        extract_str(name_val, "entryName", span)?
    } else {
        default_name = Path::new(file)
            .file_name()
            .ok_or_else(|| RuntimeError::IoError {
                message: format!("Invalid file path: {}", file),
                span,
            })?
            .to_string_lossy()
            .to_string();
        &default_name
    };

    let level = extract_level(level_opt, span)?;

    zip_add_file(Path::new(zip), Path::new(file), entry_name, level, span)?;
    Ok(Value::Null)
}

/// zipValidate(zipPath: string) -> bool
///
/// Check whether a file is a valid zip archive.
pub fn zip_validate_fn(zip_path: &Value, span: Span) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    Ok(Value::Bool(zip_validate(Path::new(zip))))
}

/// zipComment(zipPath: string) -> string
///
/// Read the archive-level comment. Returns empty string if none.
pub fn zip_comment_fn(zip_path: &Value, span: Span) -> Result<Value, RuntimeError> {
    let zip = extract_str(zip_path, "zipPath", span)?;
    let comment = zip_read_comment(Path::new(zip), span)?;
    Ok(Value::string(comment))
}

// ============================================================================
// Private helpers for argument extraction
// ============================================================================

fn extract_str<'a>(value: &'a Value, name: &str, span: Span) -> Result<&'a str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref()),
        _ => Err(RuntimeError::TypeError {
            msg: format!("{} must be a string", name),
            span,
        }),
    }
}

fn extract_string_array(
    value: &Value,
    name: &str,
    span: Span,
) -> Result<Vec<PathBuf>, RuntimeError> {
    match value {
        Value::Array(arr) => {
            let guard = arr.lock().unwrap();
            let mut paths = Vec::with_capacity(guard.len());
            for v in guard.iter() {
                match v {
                    Value::String(s) => paths.push(PathBuf::from(s.as_ref())),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: format!("{} array must contain only strings", name),
                            span,
                        })
                    }
                }
            }
            Ok(paths)
        }
        _ => Err(RuntimeError::TypeError {
            msg: format!("{} must be an array of strings", name),
            span,
        }),
    }
}

fn extract_level(level_opt: Option<&Value>, span: Span) -> Result<i64, RuntimeError> {
    if let Some(level_val) = level_opt {
        match level_val {
            Value::Number(n) => {
                let l = *n as i64;
                if !(0..=9).contains(&l) {
                    return Err(RuntimeError::IoError {
                        message: format!("Compression level must be 0-9, got {}", l),
                        span,
                    });
                }
                Ok(l)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "Compression level must be a number".to_string(),
                span,
            }),
        }
    } else {
        Ok(DEFAULT_COMPRESSION_LEVEL)
    }
}
