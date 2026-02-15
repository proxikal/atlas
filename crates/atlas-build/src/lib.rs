//! Atlas build system infrastructure
//!
//! Provides build orchestration for Atlas projects including:
//! - Build pipeline management
//! - Multiple build targets (library, binary, bytecode, test)
//! - Dependency resolution and building
//! - Parallel compilation
//! - Incremental compilation (phase-11b)
//! - Build profiles and configuration (phase-11c)

pub mod build_order;
pub mod builder;
pub mod error;
pub mod targets;

// Re-export main types
pub use builder::{BuildConfig, BuildContext, BuildStats, Builder, OptLevel};
pub use build_order::{BuildGraph, ModuleNode};
pub use error::{BuildError, BuildResult};
pub use targets::{ArtifactMetadata, BuildArtifact, BuildTarget, TargetKind};

// Re-export atlas-package types for convenience
pub use atlas_package::manifest::PackageManifest;
