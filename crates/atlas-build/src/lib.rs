//! Atlas build system infrastructure
//!
//! Provides build orchestration for Atlas projects including:
//! - Build pipeline management
//! - Multiple build targets (library, binary, bytecode, test)
//! - Dependency resolution and building
//! - Parallel compilation
//! - Incremental compilation (phase-11b)
//! - Build profiles and configuration (phase-11c)
//! - Build scripts with sandboxing (phase-11c)
//! - Progress reporting and output formatting (phase-11c)

pub mod build_order;
pub mod builder;
pub mod cache;
pub mod error;
pub mod fingerprint;
pub mod incremental;
pub mod module_resolver;
pub mod output;
pub mod profile;
pub mod script;
pub mod targets;

// Re-export main types
pub use build_order::{BuildGraph, ModuleNode};
pub use builder::{BuildConfig, BuildContext, BuildStats, Builder, OptLevel};
pub use cache::{BuildCache, CacheEntry, CacheMetadata, CacheStats};
pub use error::{BuildError, BuildResult};
pub use fingerprint::{
    compute_fingerprint, compute_hash, Fingerprint, FingerprintConfig, FingerprintDb, PlatformInfo,
};
pub use incremental::{
    BuildState, IncrementalEngine, IncrementalPlan, IncrementalStats, RecompileReason,
};
pub use output::{BuildProgress, BuildSummary, ErrorFormatter, OutputMode};
pub use profile::{
    DependencyProfile, ManifestProfileConfig, Profile, ProfileConfig, ProfileManager,
};
pub use script::{
    BuildScript, ScriptContext, ScriptExecutor, ScriptKind, ScriptPhase, ScriptResult,
};
pub use targets::{ArtifactMetadata, BuildArtifact, BuildTarget, TargetKind};

// Re-export atlas-package types for convenience
pub use atlas_package::manifest::PackageManifest;
