/// Build target types and artifact management
use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Kind of build target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetKind {
    /// Reusable library package
    Library,
    /// Executable binary program
    Binary,
    /// Standalone bytecode file
    Bytecode,
    /// Test suite
    Test,
    /// Benchmark suite
    Benchmark,
}

impl TargetKind {
    /// Get the conventional output directory name for this target kind
    pub fn output_dir_name(&self) -> &'static str {
        match self {
            Self::Library => "lib",
            Self::Binary => "bin",
            Self::Bytecode => "bytecode",
            Self::Test => "test",
            Self::Benchmark => "bench",
        }
    }

    /// Get the file extension for this target kind
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Library => "atl.bc",
            Self::Binary => "atl.bc",
            Self::Bytecode => "atl.bc",
            Self::Test => "atl.bc",
            Self::Benchmark => "atl.bc",
        }
    }

    /// Whether this target requires an entry point (main function)
    pub fn requires_entry_point(&self) -> bool {
        matches!(self, Self::Binary | Self::Test | Self::Benchmark)
    }
}

impl std::fmt::Display for TargetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library => write!(f, "library"),
            Self::Binary => write!(f, "binary"),
            Self::Bytecode => write!(f, "bytecode"),
            Self::Test => write!(f, "test"),
            Self::Benchmark => write!(f, "benchmark"),
        }
    }
}

/// A build target specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildTarget {
    /// Target name
    pub name: String,
    /// Target kind
    pub kind: TargetKind,
    /// Entry point file for binaries/tests (relative to src/)
    pub entry_point: Option<PathBuf>,
    /// Source files to include (relative to project root)
    pub sources: Vec<PathBuf>,
    /// Dependencies (package names)
    pub dependencies: Vec<String>,
}

impl BuildTarget {
    /// Create a new build target
    pub fn new(name: impl Into<String>, kind: TargetKind) -> Self {
        Self {
            name: name.into(),
            kind,
            entry_point: None,
            sources: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set the entry point
    pub fn with_entry_point(mut self, entry_point: impl Into<PathBuf>) -> Self {
        self.entry_point = Some(entry_point.into());
        self
    }

    /// Add source files
    pub fn with_sources(mut self, sources: Vec<PathBuf>) -> Self {
        self.sources = sources;
        self
    }

    /// Add dependencies
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    /// Get the output file name for this target
    pub fn output_filename(&self) -> String {
        format!("{}.{}", self.name, self.kind.file_extension())
    }

    /// Validate the target configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Target name cannot be empty".to_string());
        }

        if self.kind.requires_entry_point() && self.entry_point.is_none() {
            return Err(format!(
                "{} target '{}' requires an entry point",
                self.kind, self.name
            ));
        }

        if self.sources.is_empty() {
            return Err(format!("Target '{}' has no source files", self.name));
        }

        Ok(())
    }
}

/// Build artifact produced by a build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    /// The target that produced this artifact
    pub target: BuildTarget,
    /// Output file path
    pub output_path: PathBuf,
    /// Compiled bytecode
    pub bytecode: Vec<u8>,
    /// Artifact metadata
    pub metadata: ArtifactMetadata,
}

impl BuildArtifact {
    /// Create a new build artifact
    pub fn new(
        target: BuildTarget,
        output_path: PathBuf,
        bytecode: Vec<u8>,
        metadata: ArtifactMetadata,
    ) -> Self {
        Self {
            target,
            output_path,
            bytecode,
            metadata,
        }
    }

    /// Get the artifact size in bytes
    pub fn size(&self) -> usize {
        self.bytecode.len()
    }
}

/// Metadata about a build artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Compilation duration
    pub compile_time: Duration,
    /// Number of modules compiled
    pub module_count: usize,
    /// Bytecode size in bytes
    pub bytecode_size: usize,
    /// Atlas compiler version
    pub atlas_version: String,
    /// Build timestamp
    #[serde(with = "serde_millis")]
    pub build_time: std::time::SystemTime,
}

impl ArtifactMetadata {
    /// Create new artifact metadata
    pub fn new(
        compile_time: Duration,
        module_count: usize,
        bytecode_size: usize,
    ) -> Self {
        Self {
            compile_time,
            module_count,
            bytecode_size,
            atlas_version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: std::time::SystemTime::now(),
        }
    }
}

/// Helper module for serde SystemTime serialization
mod serde_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0));
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u128::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_millis(millis as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_kind_output_dir() {
        assert_eq!(TargetKind::Library.output_dir_name(), "lib");
        assert_eq!(TargetKind::Binary.output_dir_name(), "bin");
        assert_eq!(TargetKind::Bytecode.output_dir_name(), "bytecode");
        assert_eq!(TargetKind::Test.output_dir_name(), "test");
        assert_eq!(TargetKind::Benchmark.output_dir_name(), "bench");
    }

    #[test]
    fn test_target_kind_requires_entry_point() {
        assert!(!TargetKind::Library.requires_entry_point());
        assert!(TargetKind::Binary.requires_entry_point());
        assert!(!TargetKind::Bytecode.requires_entry_point());
        assert!(TargetKind::Test.requires_entry_point());
        assert!(TargetKind::Benchmark.requires_entry_point());
    }

    #[test]
    fn test_build_target_validation_empty_name() {
        let target = BuildTarget::new("", TargetKind::Library);
        assert!(target.validate().is_err());
    }

    #[test]
    fn test_build_target_validation_binary_requires_entry_point() {
        let target = BuildTarget::new("mybin", TargetKind::Binary)
            .with_sources(vec![PathBuf::from("src/main.atlas")]);
        assert!(target.validate().is_err());

        let target = target.with_entry_point("src/main.atlas");
        assert!(target.validate().is_ok());
    }

    #[test]
    fn test_build_target_validation_no_sources() {
        let target = BuildTarget::new("mylib", TargetKind::Library);
        assert!(target.validate().is_err());
    }

    #[test]
    fn test_build_target_output_filename() {
        let target = BuildTarget::new("mylib", TargetKind::Library);
        assert_eq!(target.output_filename(), "mylib.atl.bc");

        let target = BuildTarget::new("mybin", TargetKind::Binary);
        assert_eq!(target.output_filename(), "mybin.atl.bc");
    }

    #[test]
    fn test_library_target_creation() {
        let target = BuildTarget::new("mylib", TargetKind::Library)
            .with_sources(vec![PathBuf::from("src/lib.atlas")]);

        assert_eq!(target.name, "mylib");
        assert_eq!(target.kind, TargetKind::Library);
        assert_eq!(target.sources.len(), 1);
    }

    #[test]
    fn test_binary_target_with_entry() {
        let target = BuildTarget::new("mybin", TargetKind::Binary)
            .with_entry_point("src/main.atlas")
            .with_sources(vec![PathBuf::from("src/main.atlas")]);

        assert!(target.entry_point.is_some());
        assert_eq!(target.kind, TargetKind::Binary);
    }

    #[test]
    fn test_target_kind_display_format() {
        assert_eq!(TargetKind::Library.to_string(), "library");
        assert_eq!(TargetKind::Binary.to_string(), "binary");
        assert_eq!(TargetKind::Bytecode.to_string(), "bytecode");
        assert_eq!(TargetKind::Test.to_string(), "test");
        assert_eq!(TargetKind::Benchmark.to_string(), "benchmark");
    }

    #[test]
    fn test_target_with_deps() {
        let target = BuildTarget::new("app", TargetKind::Binary)
            .with_entry_point("src/main.atlas")
            .with_sources(vec![PathBuf::from("src/main.atlas")])
            .with_dependencies(vec!["dep1".to_string(), "dep2".to_string()]);

        assert_eq!(target.dependencies.len(), 2);
        assert!(target.dependencies.contains(&"dep1".to_string()));
    }

    #[test]
    fn test_bytecode_target_no_entry() {
        let target = BuildTarget::new("module", TargetKind::Bytecode)
            .with_sources(vec![PathBuf::from("src/module.atlas")]);

        assert!(target.validate().is_ok());
        assert!(target.entry_point.is_none());
    }

    #[test]
    fn test_test_target_needs_entry() {
        let target = BuildTarget::new("tests", TargetKind::Test)
            .with_sources(vec![PathBuf::from("tests/test.atlas")]);

        assert!(target.validate().is_err());

        let target = target.with_entry_point("tests/test.atlas");
        assert!(target.validate().is_ok());
    }

    #[test]
    fn test_output_extensions() {
        assert_eq!(TargetKind::Library.file_extension(), "atl.bc");
        assert_eq!(TargetKind::Binary.file_extension(), "atl.bc");
        assert_eq!(TargetKind::Bytecode.file_extension(), "atl.bc");
    }

    #[test]
    fn test_multiple_source_files() {
        let target = BuildTarget::new("app", TargetKind::Library)
            .with_sources(vec![
                PathBuf::from("src/lib.atlas"),
                PathBuf::from("src/utils.atlas"),
                PathBuf::from("src/helpers.atlas"),
            ]);

        assert_eq!(target.sources.len(), 3);
    }
}
