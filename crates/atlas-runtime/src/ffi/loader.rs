//! Dynamic library loading for FFI
//!
//! Provides cross-platform dynamic library loading using `libloading`.
//! Handles platform-specific library naming conventions and search paths.

use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Library loading errors
#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    /// Library file not found in search paths
    LibraryNotFound(String),
    /// Symbol not found in library
    SymbolNotFound { library: String, symbol: String },
    /// Failed to load library
    LoadFailed(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::LibraryNotFound(name) => write!(f, "Library not found: {}", name),
            LoadError::SymbolNotFound { library, symbol } => {
                write!(f, "Symbol '{}' not found in library '{}'", symbol, library)
            }
            LoadError::LoadFailed(msg) => write!(f, "Failed to load library: {}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

/// Dynamic library loader with caching and platform-specific path resolution
///
/// # Safety
///
/// Loading dynamic libraries is inherently unsafe. The loaded code runs in the
/// same process and can perform arbitrary operations.
pub struct LibraryLoader {
    /// Cache of loaded libraries by absolute path
    loaded: HashMap<PathBuf, Library>,
    /// Platform-specific library search paths
    search_paths: Vec<PathBuf>,
}

impl LibraryLoader {
    /// Create a new library loader with default search paths
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
            search_paths: Self::default_search_paths(),
        }
    }

    /// Get platform-specific default library search paths
    ///
    /// Returns standard system library paths for the current platform:
    /// - Linux: /usr/lib, /usr/local/lib, /lib
    /// - macOS: /usr/lib, /usr/local/lib, /opt/homebrew/lib
    /// - Windows: C:\Windows\System32
    /// - All platforms: current working directory
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Platform-specific standard paths
        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from("/usr/lib"));
            paths.push(PathBuf::from("/usr/local/lib"));
            paths.push(PathBuf::from("/lib"));

            // Also try lib64 on 64-bit systems
            if cfg!(target_pointer_width = "64") {
                paths.push(PathBuf::from("/usr/lib64"));
                paths.push(PathBuf::from("/lib64"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/usr/lib"));
            paths.push(PathBuf::from("/usr/local/lib"));
            paths.push(PathBuf::from("/opt/homebrew/lib"));
        }

        #[cfg(target_os = "windows")]
        {
            paths.push(PathBuf::from("C:\\Windows\\System32"));
            if let Ok(system_root) = std::env::var("SystemRoot") {
                paths.push(PathBuf::from(format!("{}\\System32", system_root)));
            }
        }

        // Current working directory (highest priority)
        if let Ok(cwd) = std::env::current_dir() {
            paths.insert(0, cwd);
        }

        paths
    }

    /// Resolve library name to full path with platform-specific naming
    ///
    /// Handles platform-specific library naming conventions:
    /// - Linux: lib{name}.so
    /// - macOS: lib{name}.dylib or lib{name}.so
    /// - Windows: {name}.dll
    ///
    /// Searches in all configured search paths.
    fn resolve_library_path(&self, name: &str) -> Option<PathBuf> {
        // If name is already a path, use it directly
        let path = Path::new(name);
        if path.is_absolute() && path.exists() {
            return Some(path.to_path_buf());
        }

        // Platform-specific extensions (in priority order)
        let extensions = if cfg!(target_os = "windows") {
            vec!["dll"]
        } else if cfg!(target_os = "macos") {
            vec!["dylib", "so"]
        } else {
            vec!["so"]
        };

        // Platform-specific prefixes (try both with and without "lib" prefix)
        let prefixes = if cfg!(target_os = "windows") {
            vec!["", "lib"] // Windows rarely uses "lib" prefix but try it
        } else {
            vec!["lib", ""] // Unix prefers "lib" prefix
        };

        // Try each combination in search paths
        for search_path in &self.search_paths {
            for prefix in &prefixes {
                for ext in &extensions {
                    let filename = if prefix.is_empty() {
                        format!("{}.{}", name, ext)
                    } else {
                        format!("{}{}.{}", prefix, name, ext)
                    };

                    let full_path = search_path.join(&filename);
                    if full_path.exists() {
                        return Some(full_path);
                    }
                }
            }
        }

        None
    }

    /// Load a library by name or path
    ///
    /// Loads the library if not already loaded, or returns the cached instance.
    /// Library name can be:
    /// - Short name: "m" -> lib{m}.{ext}
    /// - Full path: "/path/to/libfoo.so"
    ///
    /// # Safety
    ///
    /// Loading a dynamic library executes its initialization code and makes its
    /// symbols available. The caller must ensure the library is trusted.
    pub fn load(&mut self, name: &str) -> Result<&Library, LoadError> {
        // Resolve to absolute path
        let path = self
            .resolve_library_path(name)
            .ok_or_else(|| LoadError::LibraryNotFound(name.to_string()))?;

        // Return cached library if already loaded
        if self.loaded.contains_key(&path) {
            return Ok(&self.loaded[&path]);
        }

        // Load library
        let library =
            unsafe { Library::new(&path).map_err(|e| LoadError::LoadFailed(e.to_string()))? };

        self.loaded.insert(path.clone(), library);
        Ok(&self.loaded[&path])
    }

    /// Lookup a symbol in a loaded library
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - The symbol exists and has the correct type T
    /// - The function pointer's signature matches the actual symbol
    /// - The library remains loaded while the symbol is in use
    pub unsafe fn lookup_symbol<T>(
        &self,
        library_name: &str,
        symbol_name: &str,
    ) -> Result<Symbol<'_, T>, LoadError> {
        // Resolve library path
        let path = self
            .resolve_library_path(library_name)
            .ok_or_else(|| LoadError::LibraryNotFound(library_name.to_string()))?;

        // Get loaded library
        let library = self.loaded.get(&path).ok_or_else(|| {
            LoadError::LibraryNotFound(format!("{} (not loaded - call load() first)", library_name))
        })?;

        // Lookup symbol
        library
            .get(symbol_name.as_bytes())
            .map_err(|_| LoadError::SymbolNotFound {
                library: library_name.to_string(),
                symbol: symbol_name.to_string(),
            })
    }

    /// Add a custom search path (prepended to search list)
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.insert(0, path);
    }

    /// Get the number of loaded libraries
    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }
}

impl Default for LibraryLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_paths_not_empty() {
        let paths = LibraryLoader::default_search_paths();
        assert!(!paths.is_empty());

        // Current directory should be first
        if let Ok(cwd) = std::env::current_dir() {
            assert_eq!(paths[0], cwd);
        }
    }

    #[test]
    fn test_platform_specific_paths() {
        let paths = LibraryLoader::default_search_paths();

        #[cfg(target_os = "linux")]
        {
            assert!(paths.iter().any(|p| p == Path::new("/usr/lib")));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(paths.iter().any(|p| p == Path::new("/usr/lib")));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(paths
                .iter()
                .any(|p| p.to_str().unwrap().contains("System32")));
        }
    }

    #[test]
    fn test_library_not_found() {
        let mut loader = LibraryLoader::new();
        let result = loader.load("nonexistent_library_xyz");
        assert!(matches!(result, Err(LoadError::LibraryNotFound(_))));
    }

    #[test]
    fn test_loader_caching() {
        let loader = LibraryLoader::new();
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_add_custom_search_path() {
        let mut loader = LibraryLoader::new();
        let custom_path = PathBuf::from("/custom/path");
        loader.add_search_path(custom_path.clone());

        // Custom path should be first
        assert_eq!(loader.search_paths[0], custom_path);
    }
}
