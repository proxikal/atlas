//! File system operations - comprehensive utilities
//!
//! Advanced file system operations including metadata, symlinks, temporary files,
//! and directory walking. Complements basic I/O operations in io.rs.

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[cfg(unix)]
use std::os::unix::fs as unix_fs;

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert SystemTime to Unix timestamp (seconds since epoch)
fn system_time_to_timestamp(time: SystemTime) -> f64 {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(_) => 0.0,
    }
}

// ============================================================================
// Directory Operations
// ============================================================================

/// Create a directory
///
/// Creates a single directory. Fails if parent doesn't exist or directory already exists.
pub fn mkdir(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    fs::create_dir(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create directory '{}': {}", path.display(), e),
        span,
    })?;

    Ok(Value::Null)
}

/// Create a directory recursively (mkdir -p)
///
/// Creates all parent directories as needed. Succeeds silently if directory already exists.
pub fn mkdirp(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    fs::create_dir_all(path).map_err(|e| RuntimeError::IoError {
        message: format!(
            "Failed to create directory recursively '{}': {}",
            path.display(),
            e
        ),
        span,
    })?;

    Ok(Value::Null)
}

/// Remove an empty directory
///
/// Fails if directory is not empty or doesn't exist.
pub fn rmdir(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    fs::remove_dir(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to remove directory '{}': {}", path.display(), e),
        span,
    })?;

    Ok(Value::Null)
}

/// Remove a directory recursively (rm -rf)
///
/// Removes directory and all contents. Use with caution!
pub fn rmdir_recursive(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    fs::remove_dir_all(path).map_err(|e| RuntimeError::IoError {
        message: format!(
            "Failed to remove directory recursively '{}': {}",
            path.display(),
            e
        ),
        span,
    })?;

    Ok(Value::Null)
}

/// List directory contents
///
/// Returns array of file/directory names (not full paths).
pub fn readdir(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    let entries = fs::read_dir(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read directory '{}': {}", path.display(), e),
        span,
    })?;

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read directory entry: {}", e),
            span,
        })?;

        if let Some(name) = entry.file_name().to_str() {
            names.push(Value::string(name.to_string()));
        }
    }

    Ok(Value::array(names))
}

/// Walk directory tree recursively
///
/// Returns array of all file paths (relative to the starting directory).
pub fn walk(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    let mut all_paths = Vec::new();
    walk_recursive(path, path, &mut all_paths, span)?;

    Ok(Value::array(all_paths))
}

/// Recursive helper for walk
fn walk_recursive(
    base: &Path,
    current: &Path,
    all_paths: &mut Vec<Value>,
    span: Span,
) -> Result<(), RuntimeError> {
    let entries = fs::read_dir(current).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read directory '{}': {}", current.display(), e),
        span,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read directory entry: {}", e),
            span,
        })?;

        let path = entry.path();

        // Get relative path from base
        let relative = path.strip_prefix(base).unwrap_or(&path);
        if let Some(path_str) = relative.to_str() {
            all_paths.push(Value::string(path_str.to_string()));
        }

        // Recurse into subdirectories
        if path.is_dir() {
            walk_recursive(base, &path, all_paths, span)?;
        }
    }

    Ok(())
}

/// Filter directory entries by pattern
///
/// Simple glob-style pattern matching (* wildcard).
pub fn filter_entries(
    entries: &[Value],
    pattern: &str,
    _span: Span,
) -> Result<Value, RuntimeError> {
    let filtered: Vec<Value> = entries
        .iter()
        .filter(|v| {
            if let Value::String(s) = v {
                glob_match(s.as_ref(), pattern)
            } else {
                false
            }
        })
        .cloned()
        .collect();

    Ok(Value::array(filtered))
}

/// Simple glob pattern matching (supports * wildcard)
fn glob_match(text: &str, pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // No wildcard - exact match
        return text == pattern;
    }

    // Check prefix
    if !parts[0].is_empty() && !text.starts_with(parts[0]) {
        return false;
    }

    // Check suffix
    if !parts[parts.len() - 1].is_empty() && !text.ends_with(parts[parts.len() - 1]) {
        return false;
    }

    // Check middle parts
    let mut pos = parts[0].len();
    for part in parts.iter().take(parts.len() - 1).skip(1) {
        if let Some(found_pos) = text[pos..].find(part) {
            pos += found_pos + part.len();
        } else {
            return false;
        }
    }

    true
}

/// Sort directory entries
///
/// Sorts entries alphabetically (case-insensitive).
pub fn sort_entries(entries: &[Value], _span: Span) -> Result<Value, RuntimeError> {
    let mut sorted: Vec<Value> = entries.to_vec();

    sorted.sort_by(|a, b| {
        let a_str = if let Value::String(s) = a {
            s.as_ref().to_lowercase()
        } else {
            String::new()
        };
        let b_str = if let Value::String(s) = b {
            s.as_ref().to_lowercase()
        } else {
            String::new()
        };
        a_str.cmp(&b_str)
    });

    Ok(Value::array(sorted))
}

// ============================================================================
// File Metadata
// ============================================================================

/// Get file size in bytes
pub fn size(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let metadata = fs::metadata(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", path, e),
        span,
    })?;

    Ok(Value::Number(metadata.len() as f64))
}

/// Get modified time as Unix timestamp
pub fn mtime(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let metadata = fs::metadata(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", path, e),
        span,
    })?;

    let modified = metadata.modified().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get modified time: {}", e),
        span,
    })?;

    Ok(Value::Number(system_time_to_timestamp(modified)))
}

/// Get created time as Unix timestamp
pub fn ctime(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let metadata = fs::metadata(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", path, e),
        span,
    })?;

    let created = metadata.created().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get created time: {}", e),
        span,
    })?;

    Ok(Value::Number(system_time_to_timestamp(created)))
}

/// Get access time as Unix timestamp
pub fn atime(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let metadata = fs::metadata(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", path, e),
        span,
    })?;

    let accessed = metadata.accessed().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get accessed time: {}", e),
        span,
    })?;

    Ok(Value::Number(system_time_to_timestamp(accessed)))
}

/// Get file permissions (Unix mode as number, or basic info for other platforms)
pub fn permissions(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let metadata = fs::metadata(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", path, e),
        span,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        Ok(Value::Number((mode & 0o777) as f64))
    }

    #[cfg(not(unix))]
    {
        let readonly = metadata.permissions().readonly();
        // Return simplified permission: 0o444 (read-only) or 0o666 (read-write)
        Ok(Value::Number(if readonly {
            0o444 as f64
        } else {
            0o666 as f64
        }))
    }
}

/// Check if path is a directory
pub fn is_dir(path: &str, _span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);
    Ok(Value::Bool(path.is_dir()))
}

/// Check if path is a file
pub fn is_file(path: &str, _span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);
    Ok(Value::Bool(path.is_file()))
}

/// Check if path is a symlink
pub fn is_symlink(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);
    let metadata = fs::symlink_metadata(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", path.display(), e),
        span,
    })?;

    Ok(Value::Bool(metadata.is_symlink()))
}

/// Get inode number (Unix only)
#[cfg(unix)]
pub fn inode(path: &str, span: Span) -> Result<Value, RuntimeError> {
    use std::os::unix::fs::MetadataExt;

    let metadata = fs::metadata(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", path, e),
        span,
    })?;

    Ok(Value::Number(metadata.ino() as f64))
}

#[cfg(not(unix))]
pub fn inode(_path: &str, span: Span) -> Result<Value, RuntimeError> {
    Err(RuntimeError::IoError {
        message: "inode() is only available on Unix platforms".to_string(),
        span,
    })
}

// ============================================================================
// Temporary Files and Directories
// ============================================================================

/// Create a temporary file
///
/// Returns path to the temporary file. File is NOT automatically deleted.
pub fn tmpfile(span: Span) -> Result<Value, RuntimeError> {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let filename = format!("atlas_tmp_{}.tmp", timestamp);
    let temp_path = temp_dir.join(filename);

    // Create the file
    fs::File::create(&temp_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create temporary file: {}", e),
        span,
    })?;

    Ok(Value::string(temp_path.to_string_lossy().to_string()))
}

/// Create a temporary directory
///
/// Returns path to the temporary directory. Directory is NOT automatically deleted.
pub fn tmpdir(span: Span) -> Result<Value, RuntimeError> {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dirname = format!("atlas_tmp_dir_{}", timestamp);
    let temp_path = temp_dir.join(dirname);

    // Create the directory
    fs::create_dir(&temp_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create temporary directory: {}", e),
        span,
    })?;

    Ok(Value::string(temp_path.to_string_lossy().to_string()))
}

/// Create a named temporary file
///
/// Creates a temporary file with a specific name prefix.
pub fn tmpfile_named(prefix: &str, span: Span) -> Result<Value, RuntimeError> {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let filename = format!("{}_{}.tmp", prefix, timestamp);
    let temp_path = temp_dir.join(filename);

    // Create the file
    fs::File::create(&temp_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create named temporary file: {}", e),
        span,
    })?;

    Ok(Value::string(temp_path.to_string_lossy().to_string()))
}

/// Get system temporary directory path
pub fn get_temp_dir(_span: Span) -> Result<Value, RuntimeError> {
    use std::env;
    let temp_dir = env::temp_dir();
    Ok(Value::string(temp_dir.to_string_lossy().to_string()))
}

// ============================================================================
// Symlink Operations
// ============================================================================

/// Create a symbolic link
///
/// Creates a symlink from `link` pointing to `target`.
/// On Unix: uses symlink. On Windows: requires admin privileges.
pub fn symlink(target: &str, link: &str, span: Span) -> Result<Value, RuntimeError> {
    let target = Path::new(target);
    let link = Path::new(link);

    #[cfg(unix)]
    {
        unix_fs::symlink(target, link).map_err(|e| RuntimeError::IoError {
            message: format!(
                "Failed to create symlink '{}' -> '{}': {}",
                link.display(),
                target.display(),
                e
            ),
            span,
        })?;
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs as windows_fs;

        // On Windows, we need to know if target is a file or directory
        if target.is_dir() {
            windows_fs::symlink_dir(target, link).map_err(|e| RuntimeError::IoError {
                message: format!(
                    "Failed to create directory symlink '{}' -> '{}': {}",
                    link.display(),
                    target.display(),
                    e
                ),
                span,
            })?;
        } else {
            windows_fs::symlink_file(target, link).map_err(|e| RuntimeError::IoError {
                message: format!(
                    "Failed to create file symlink '{}' -> '{}': {}",
                    link.display(),
                    target.display(),
                    e
                ),
                span,
            })?;
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(RuntimeError::IoError {
            message: "Symlinks are not supported on this platform".to_string(),
            span,
        });
    }

    Ok(Value::Null)
}

/// Read symlink target
///
/// Returns the path that the symlink points to.
pub fn readlink(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    let target = fs::read_link(path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read symlink '{}': {}", path.display(), e),
        span,
    })?;

    Ok(Value::string(target.to_string_lossy().to_string()))
}

/// Resolve symlink chain to final target
///
/// Follows symlinks until reaching a non-symlink path.
pub fn resolve_symlink(path: &str, span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path);

    let canonical = path.canonicalize().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to resolve symlink '{}': {}", path.display(), e),
        span,
    })?;

    Ok(Value::string(canonical.to_string_lossy().to_string()))
}
