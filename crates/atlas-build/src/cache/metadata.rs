//! Change detection and file state tracking for incremental builds

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Change detector tracks file state and detects changes
#[derive(Debug)]
pub struct ChangeDetector {
    previous_state: HashMap<PathBuf, FileState>,
}

/// File state snapshot
#[derive(Debug, Clone)]
pub struct FileState {
    /// Last modified timestamp
    pub timestamp: SystemTime,
    /// File size in bytes
    pub size: u64,
    /// Content hash (computed lazily when timestamp changes)
    pub hash: Option<String>,
}

/// Changed file information
#[derive(Debug, Clone)]
pub struct ChangedFile {
    /// File path
    pub path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
}

/// Type of file change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// File content modified
    Modified,
    /// New file added
    Added,
    /// File removed
    Removed,
    /// File moved from another location
    Moved { from: PathBuf },
}

impl ChangeDetector {
    /// Create a new change detector
    pub fn new() -> Self {
        Self {
            previous_state: HashMap::new(),
        }
    }

    /// Detect changes in files since last check
    pub fn detect_changes(&mut self, files: &[PathBuf]) -> Vec<ChangedFile> {
        let mut changes = Vec::new();
        let mut current_state = HashMap::new();
        let mut content_hashes: HashMap<String, PathBuf> = HashMap::new();

        // Check each file for changes
        for path in files {
            if !path.exists() {
                continue;
            }

            let current_file_state = match Self::read_file_state(path) {
                Ok(state) => state,
                Err(_) => continue,
            };

            // Check if file existed before
            if let Some(previous) = self.previous_state.get(path) {
                // File existed - check for modifications
                if self.file_changed(path, &current_file_state, previous) {
                    changes.push(ChangedFile {
                        path: path.clone(),
                        change_type: ChangeType::Modified,
                    });
                }
            } else {
                // New file - check if it was moved
                let hash = self.get_or_compute_hash(path, &current_file_state);

                if let Some(old_path) = self.find_file_by_hash(&hash) {
                    // File was moved
                    changes.push(ChangedFile {
                        path: path.clone(),
                        change_type: ChangeType::Moved {
                            from: old_path.clone(),
                        },
                    });
                } else {
                    // Truly new file
                    changes.push(ChangedFile {
                        path: path.clone(),
                        change_type: ChangeType::Added,
                    });
                }

                content_hashes.insert(hash, path.clone());
            }

            current_state.insert(path.clone(), current_file_state);
        }

        // Check for removed files
        for path in self.previous_state.keys() {
            if !current_state.contains_key(path) {
                changes.push(ChangedFile {
                    path: path.clone(),
                    change_type: ChangeType::Removed,
                });
            }
        }

        // Update state
        self.previous_state = current_state;

        changes
    }

    /// Update state for a single file without detecting changes
    pub fn update_file(&mut self, path: &Path) {
        if let Ok(state) = Self::read_file_state(path) {
            self.previous_state.insert(path.to_path_buf(), state);
        }
    }

    /// Clear all tracked state
    pub fn clear(&mut self) {
        self.previous_state.clear();
    }

    /// Check if a file has changed
    fn file_changed(&self, path: &Path, current: &FileState, previous: &FileState) -> bool {
        // Quick check: timestamp and size
        if current.timestamp == previous.timestamp && current.size == previous.size {
            return false; // Definitely not changed
        }

        // Size changed - definitely modified
        if current.size != previous.size {
            return true;
        }

        // Timestamp changed but size same - need to check content
        let current_hash = self.get_or_compute_hash(path, current);
        let previous_hash = previous.hash.as_deref().unwrap_or("");

        current_hash != previous_hash
    }

    /// Get hash from state or compute it
    fn get_or_compute_hash(&self, path: &Path, state: &FileState) -> String {
        if let Some(ref hash) = state.hash {
            return hash.clone();
        }

        Self::compute_file_hash(path).unwrap_or_default()
    }

    /// Find a file by its content hash (for move detection)
    fn find_file_by_hash(&self, hash: &str) -> Option<&PathBuf> {
        for (path, state) in &self.previous_state {
            if let Some(ref file_hash) = state.hash {
                if file_hash == hash {
                    return Some(path);
                }
            }
        }
        None
    }

    /// Read file state (timestamp, size)
    fn read_file_state(path: &Path) -> std::io::Result<FileState> {
        let metadata = fs::metadata(path)?;
        let timestamp = metadata.modified()?;
        let size = metadata.len();

        Ok(FileState {
            timestamp,
            size,
            hash: None, // Computed lazily when needed
        })
    }

    /// Compute SHA-256 hash of file content
    fn compute_file_hash(path: &Path) -> std::io::Result<String> {
        let content = fs::read_to_string(path)?;
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_new_change_detector() {
        let detector = ChangeDetector::new();
        assert!(detector.previous_state.is_empty());
    }

    #[test]
    fn test_detect_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let mut detector = ChangeDetector::new();
        let changes = detector.detect_changes(std::slice::from_ref(&file_path));

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, file_path);
        assert_eq!(changes[0].change_type, ChangeType::Added);
    }

    #[test]
    fn test_detect_modified_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "original").unwrap();

        let mut detector = ChangeDetector::new();
        detector.detect_changes(std::slice::from_ref(&file_path));

        // Modify file
        thread::sleep(Duration::from_millis(10));
        fs::write(&file_path, "modified").unwrap();

        let changes = detector.detect_changes(std::slice::from_ref(&file_path));

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Modified);
    }

    #[test]
    fn test_detect_removed_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let mut detector = ChangeDetector::new();
        detector.detect_changes(std::slice::from_ref(&file_path));

        // Remove file
        fs::remove_file(&file_path).unwrap();

        let changes = detector.detect_changes(&[]);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Removed);
    }

    #[test]
    fn test_no_change_same_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let mut detector = ChangeDetector::new();
        detector.detect_changes(std::slice::from_ref(&file_path));

        // No modification
        let changes = detector.detect_changes(std::slice::from_ref(&file_path));

        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_compute_file_hash() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let hash1 = ChangeDetector::compute_file_hash(&file_path).unwrap();
        let hash2 = ChangeDetector::compute_file_hash(&file_path).unwrap();

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_update_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let mut detector = ChangeDetector::new();
        detector.update_file(&file_path);

        assert_eq!(detector.previous_state.len(), 1);
        assert!(detector.previous_state.contains_key(&file_path));
    }

    #[test]
    fn test_clear() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let mut detector = ChangeDetector::new();
        detector.detect_changes(&[file_path]);

        detector.clear();
        assert!(detector.previous_state.is_empty());
    }
}
