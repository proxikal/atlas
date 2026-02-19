//! Path manipulation and utilities
//!
//! Cross-platform path operations equivalent to Node.js path module

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR, MAIN_SEPARATOR_STR};

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract string from Value, returning error if not a string
fn expect_string(value: &Value, param_name: &str, span: Span) -> Result<String, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref().clone()),
        _ => Err(RuntimeError::TypeError {
            msg: format!("{} must be a string", param_name),
            span,
        }),
    }
}

/// Extract array from Value, returning error if not an array
#[allow(dead_code)]
fn expect_array(value: &Value, param_name: &str, span: Span) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr.lock().unwrap().clone()),
        _ => Err(RuntimeError::TypeError {
            msg: format!("{} must be an array", param_name),
            span,
        }),
    }
}

// ============================================================================
// Path Construction and Parsing
// ============================================================================

/// Join path components
///
/// Takes an array of path segments and joins them with the platform-specific separator.
/// Handles edge cases like empty segments, relative paths, and normalization.
pub fn path_join(segments: &[Value], span: Span) -> Result<String, RuntimeError> {
    if segments.is_empty() {
        return Ok(".".to_string());
    }

    let mut path = PathBuf::new();

    for (i, segment) in segments.iter().enumerate() {
        let seg_str = expect_string(segment, &format!("segment {}", i), span)?;
        if !seg_str.is_empty() {
            path.push(seg_str);
        }
    }

    if path.as_os_str().is_empty() {
        Ok(".".to_string())
    } else {
        Ok(path.to_string_lossy().to_string())
    }
}

/// Parse path into components
///
/// Returns an object with: root, dir, base, ext, name
pub fn path_parse(path_str: &str, _span: Span) -> Result<Value, RuntimeError> {
    let path = Path::new(path_str);

    // Extract root (e.g., "/" or "C:\")
    let root = path
        .components()
        .next()
        .and_then(|c| match c {
            Component::Prefix(p) => Some(p.as_os_str().to_string_lossy().to_string()),
            Component::RootDir => Some(MAIN_SEPARATOR.to_string()),
            _ => None,
        })
        .unwrap_or_default();

    // Extract directory (everything except filename)
    let dir = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // Extract base filename (with extension)
    let base = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Extract extension (without dot)
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    // Extract name (without extension)
    let name = path
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Build result object as HashMap
    use crate::stdlib::collections::hash::HashKey;
    use std::sync::{Arc, Mutex};
    let mut map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
    map.insert(
        HashKey::String(Arc::new("root".to_string())),
        Value::string(root),
    );
    map.insert(
        HashKey::String(Arc::new("dir".to_string())),
        Value::string(dir),
    );
    map.insert(
        HashKey::String(Arc::new("base".to_string())),
        Value::string(base),
    );
    map.insert(
        HashKey::String(Arc::new("ext".to_string())),
        Value::string(ext),
    );
    map.insert(
        HashKey::String(Arc::new("name".to_string())),
        Value::string(name),
    );

    Ok(Value::HashMap(Arc::new(Mutex::new(map))))
}

/// Normalize path
///
/// Removes redundant separators, resolves . and .. components
pub fn path_normalize(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    let path = Path::new(path_str);
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {
                // Skip "." components unless path is empty
                if normalized.as_os_str().is_empty() {
                    normalized.push(".");
                }
            }
            Component::ParentDir => {
                // Handle ".." - pop if possible, otherwise keep it
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            _ => normalized.push(component),
        }
    }

    if normalized.as_os_str().is_empty() {
        Ok(".".to_string())
    } else {
        // Always use forward slashes for cross-platform consistency
        Ok(normalized.to_string_lossy().replace('\\', "/"))
    }
}

/// Convert to absolute path
///
/// Resolves relative path against current working directory
pub fn path_absolute(path_str: &str, span: Span) -> Result<String, RuntimeError> {
    let path = Path::new(path_str);

    if path.is_absolute() {
        return Ok(path.to_string_lossy().to_string());
    }

    // Get current working directory
    let cwd = std::env::current_dir().map_err(|e| RuntimeError::TypeError {
        msg: format!("Failed to get current directory: {}", e),
        span,
    })?;

    let absolute = cwd.join(path);
    Ok(absolute.to_string_lossy().to_string())
}

/// Compute relative path from 'from' to 'to'
///
/// Returns the relative path needed to get from 'from' to 'to'
pub fn path_relative(from: &str, to: &str, span: Span) -> Result<String, RuntimeError> {
    let from_path = PathBuf::from(from);
    let to_path = PathBuf::from(to);

    // Make both paths absolute for proper comparison
    let from_abs = if from_path.is_absolute() {
        from_path
    } else {
        std::env::current_dir()
            .map_err(|e| RuntimeError::TypeError {
                msg: format!("Failed to get current directory: {}", e),
                span,
            })?
            .join(from_path)
    };

    let to_abs = if to_path.is_absolute() {
        to_path
    } else {
        std::env::current_dir()
            .map_err(|e| RuntimeError::TypeError {
                msg: format!("Failed to get current directory: {}", e),
                span,
            })?
            .join(to_path)
    };

    // Use pathdiff for computing relative path
    match pathdiff::diff_paths(&to_abs, &from_abs) {
        Some(rel) => Ok(rel.to_string_lossy().to_string()),
        None => Ok(to_abs.to_string_lossy().to_string()),
    }
}

/// Get parent directory
///
/// Returns the parent directory path, or empty string if no parent
pub fn path_parent(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    let path = Path::new(path_str);
    Ok(path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default())
}

/// Get base filename (with extension)
pub fn path_basename(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    let path = Path::new(path_str);
    Ok(path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default())
}

/// Get directory name (everything except filename)
pub fn path_dirname(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    let path = Path::new(path_str);
    Ok(path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default())
}

/// Get file extension (without dot)
pub fn path_extension(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    let path = Path::new(path_str);
    Ok(path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default())
}

// ============================================================================
// Path Comparison and Validation
// ============================================================================

/// Check if path is absolute
pub fn path_is_absolute(path_str: &str, _span: Span) -> Result<bool, RuntimeError> {
    Ok(Path::new(path_str).is_absolute())
}

/// Check if path is relative
pub fn path_is_relative(path_str: &str, _span: Span) -> Result<bool, RuntimeError> {
    Ok(!Path::new(path_str).is_absolute())
}

/// Check if path exists
pub fn path_exists(path_str: &str, _span: Span) -> Result<bool, RuntimeError> {
    Ok(Path::new(path_str).exists())
}

/// Get canonical path (resolves symlinks and makes absolute)
pub fn path_canonical(path_str: &str, span: Span) -> Result<String, RuntimeError> {
    let path = Path::new(path_str);
    match path.canonicalize() {
        Ok(canonical) => Ok(canonical.to_string_lossy().to_string()),
        Err(e) => Err(RuntimeError::TypeError {
            msg: format!("Failed to canonicalize path '{}': {}", path_str, e),
            span,
        }),
    }
}

/// Compare two paths for equality
///
/// Normalizes both paths before comparison
pub fn path_equals(path1: &str, path2: &str, _span: Span) -> Result<bool, RuntimeError> {
    let p1 = Path::new(path1);
    let p2 = Path::new(path2);

    // On Windows, do case-insensitive comparison
    #[cfg(target_os = "windows")]
    {
        Ok(p1.to_string_lossy().to_lowercase() == p2.to_string_lossy().to_lowercase())
    }

    // On Unix, do case-sensitive comparison
    #[cfg(not(target_os = "windows"))]
    {
        Ok(p1 == p2)
    }
}

// ============================================================================
// Path Utilities
// ============================================================================

/// Get home directory
pub fn path_homedir(span: Span) -> Result<String, RuntimeError> {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| RuntimeError::TypeError {
            msg: "Failed to determine home directory".to_string(),
            span,
        })
}

/// Get current working directory
pub fn path_cwd(span: Span) -> Result<String, RuntimeError> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| RuntimeError::TypeError {
            msg: format!("Failed to get current directory: {}", e),
            span,
        })
}

/// Get system temporary directory
pub fn path_tempdir(_span: Span) -> Result<String, RuntimeError> {
    Ok(std::env::temp_dir().to_string_lossy().to_string())
}

/// Get path separator (platform-specific)
pub fn path_separator(_span: Span) -> Result<String, RuntimeError> {
    Ok(MAIN_SEPARATOR.to_string())
}

/// Get directory separator (alias for path_separator)
pub fn path_delimiter(_span: Span) -> Result<String, RuntimeError> {
    // PATH delimiter, not path separator
    #[cfg(target_os = "windows")]
    {
        Ok(";".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(":".to_string())
    }
}

/// Get extension separator
pub fn path_ext_separator(_span: Span) -> Result<String, RuntimeError> {
    Ok(".".to_string())
}

/// Extract drive letter (Windows only)
///
/// Returns empty string on non-Windows platforms or if no drive letter
pub fn path_drive(_path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    #[cfg(target_os = "windows")]
    {
        let path = Path::new(_path_str);
        if let Some(Component::Prefix(prefix)) = path.components().next() {
            return Ok(prefix.as_os_str().to_string_lossy().to_string());
        }
    }

    Ok(String::new())
}

/// Convert path to use platform-specific separators
pub fn path_to_platform(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    // Replace both / and \ with platform separator
    let normalized = path_str.replace('/', MAIN_SEPARATOR_STR);
    let normalized = normalized.replace('\\', MAIN_SEPARATOR_STR);
    Ok(normalized)
}

/// Convert path to use forward slashes (cross-platform representation)
pub fn path_to_posix(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    Ok(path_str.replace('\\', "/"))
}

/// Convert path to use backslashes (Windows representation)
pub fn path_to_windows(path_str: &str, _span: Span) -> Result<String, RuntimeError> {
    Ok(path_str.replace('/', "\\"))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    fn test_span() -> Span {
        Span::dummy()
    }

    #[test]
    fn test_path_join_basic() {
        let segments = vec![
            Value::string("foo".to_string()),
            Value::string("bar".to_string()),
            Value::string("baz.txt".to_string()),
        ];
        let result = path_join(&segments, test_span()).unwrap();

        #[cfg(target_os = "windows")]
        assert_eq!(result, "foo\\bar\\baz.txt");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, "foo/bar/baz.txt");
    }

    #[test]
    fn test_path_join_empty() {
        let segments = vec![];
        let result = path_join(&segments, test_span()).unwrap();
        assert_eq!(result, ".");
    }

    #[test]
    fn test_path_join_with_empty_segments() {
        let segments = vec![
            Value::string("foo".to_string()),
            Value::string("".to_string()),
            Value::string("bar".to_string()),
        ];
        let result = path_join(&segments, test_span()).unwrap();

        #[cfg(target_os = "windows")]
        assert_eq!(result, "foo\\bar");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, "foo/bar");
    }

    #[test]
    fn test_path_basename() {
        assert_eq!(
            path_basename("/foo/bar/baz.txt", test_span()).unwrap(),
            "baz.txt"
        );
        assert_eq!(path_basename("/foo/bar/", test_span()).unwrap(), "bar");
        assert_eq!(path_basename("baz.txt", test_span()).unwrap(), "baz.txt");
    }

    #[test]
    fn test_path_dirname() {
        assert_eq!(
            path_dirname("/foo/bar/baz.txt", test_span()).unwrap(),
            "/foo/bar"
        );
        assert_eq!(path_dirname("baz.txt", test_span()).unwrap(), "");
    }

    #[test]
    fn test_path_extension() {
        assert_eq!(
            path_extension("foo/bar/baz.txt", test_span()).unwrap(),
            "txt"
        );
        assert_eq!(
            path_extension("foo/bar/baz.tar.gz", test_span()).unwrap(),
            "gz"
        );
        assert_eq!(path_extension("foo/bar/baz", test_span()).unwrap(), "");
    }

    #[test]
    fn test_path_is_absolute() {
        #[cfg(target_os = "windows")]
        {
            assert!(path_is_absolute("C:\\foo\\bar", test_span()).unwrap());
            assert!(!path_is_absolute("foo\\bar", test_span()).unwrap());
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert!(path_is_absolute("/foo/bar", test_span()).unwrap());
            assert!(!path_is_absolute("foo/bar", test_span()).unwrap());
        }
    }

    #[test]
    fn test_path_normalize() {
        // path_normalize always returns forward slashes for cross-platform consistency
        assert_eq!(
            path_normalize("foo/bar/../baz", test_span()).unwrap(),
            "foo/baz"
        );
        assert_eq!(path_normalize("foo/./bar", test_span()).unwrap(), "foo/bar");
        assert_eq!(path_normalize("foo//bar", test_span()).unwrap(), "foo/bar");
    }

    #[test]
    fn test_path_separator() {
        #[cfg(target_os = "windows")]
        assert_eq!(path_separator(test_span()).unwrap(), "\\");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(path_separator(test_span()).unwrap(), "/");
    }

    #[test]
    fn test_path_to_posix() {
        assert_eq!(
            path_to_posix("foo\\bar\\baz", test_span()).unwrap(),
            "foo/bar/baz"
        );
    }

    #[test]
    fn test_path_to_windows() {
        assert_eq!(
            path_to_windows("foo/bar/baz", test_span()).unwrap(),
            "foo\\bar\\baz"
        );
    }
}
