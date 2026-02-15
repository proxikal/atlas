use crate::manifest::{Dependency, PackageManifest};
use semver::{Version, VersionReq};
use std::collections::HashMap;
use thiserror::Error;

pub mod conflict;
mod graph;
mod version_solver;

pub use conflict::{Conflict, ConflictResolver, ConflictingConstraint};
pub use graph::DependencyGraph;
pub use version_solver::VersionSolver;

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("No version of package '{package}' satisfies constraints: {constraints}")]
    NoSatisfyingVersion {
        package: String,
        constraints: String,
    },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Invalid version requirement: {0}")]
    InvalidVersionReq(String),

    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),
}

pub type ResolverResult<T> = Result<T, ResolverError>;

/// Core dependency resolver using PubGrub algorithm
pub struct Resolver {
    /// Dependency graph being built
    graph: DependencyGraph,

    /// Version constraints for each package
    constraints: HashMap<String, Vec<VersionConstraint>>,

    /// Version solver
    solver: VersionSolver,
}

/// Version constraint with source tracking
#[derive(Debug, Clone, PartialEq)]
pub struct VersionConstraint {
    pub requirement: VersionReq,
    pub source: String,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            constraints: HashMap::new(),
            solver: VersionSolver::new(),
        }
    }

    /// Resolve dependencies from a manifest
    pub fn resolve(&mut self, manifest: &PackageManifest) -> ResolverResult<Resolution> {
        // Add root package to graph
        let root_name = manifest.package.name.clone();
        let root_version = manifest.package.version.clone();
        self.graph.add_package(root_name.clone(), root_version);

        // Add direct dependencies from manifest
        for (name, dep) in &manifest.dependencies {
            self.add_constraint(&root_name, name, dep)?;
        }

        // For now, we'll simulate available versions
        // In phase-08b, this will query the registry
        self.populate_available_versions();

        // Run constraint solver to find compatible versions
        self.solve()
    }

    /// Add version constraint for a package
    fn add_constraint(
        &mut self,
        source: &str,
        package: &str,
        dep: &Dependency,
    ) -> ResolverResult<()> {
        // Get version constraint from dependency
        let version_str = dep.version_constraint().unwrap_or("*");

        // Parse version requirement
        let version_req = version_str.parse::<VersionReq>().map_err(|e| {
            ResolverError::InvalidVersionReq(format!("Invalid version '{}': {}", version_str, e))
        })?;

        let constraint = VersionConstraint {
            requirement: version_req,
            source: source.to_string(),
        };

        self.constraints
            .entry(package.to_string())
            .or_default()
            .push(constraint);

        Ok(())
    }

    /// Populate available versions (stub for phase-08a)
    /// In phase-08b, this will query the registry
    fn populate_available_versions(&mut self) {
        // For testing purposes, add some mock versions
        // This will be replaced by registry queries in phase-08b
        for package in self.constraints.keys() {
            let versions = vec![
                Version::new(1, 0, 0),
                Version::new(1, 1, 0),
                Version::new(1, 2, 0),
                Version::new(2, 0, 0),
            ];
            self.solver.add_package_versions(package, versions);
        }
    }

    /// Run constraint solver to find compatible versions
    fn solve(&mut self) -> ResolverResult<Resolution> {
        let mut resolved_packages = HashMap::new();

        // For each package with constraints, find compatible version
        for (package, constraints) in &self.constraints {
            let requirements: Vec<VersionReq> =
                constraints.iter().map(|c| c.requirement.clone()).collect();

            // Find maximum version satisfying all constraints
            let version = self
                .solver
                .max_satisfying_version(package, &requirements)
                .ok_or_else(|| ResolverError::NoSatisfyingVersion {
                    package: package.clone(),
                    constraints: format!("{:?}", requirements),
                })?;

            resolved_packages.insert(
                package.clone(),
                ResolvedPackage {
                    name: package.clone(),
                    version,
                    dependencies: Vec::new(), // Will populate in phase-08c
                },
            );
        }

        Ok(Resolution {
            packages: resolved_packages,
        })
    }

    /// Check if constraints are compatible
    pub fn check_compatibility(&self, package: &str) -> ResolverResult<bool> {
        if let Some(constraints) = self.constraints.get(package) {
            let requirements: Vec<VersionReq> =
                constraints.iter().map(|c| c.requirement.clone()).collect();

            // Try to find any version that satisfies all constraints
            if self
                .solver
                .max_satisfying_version(package, &requirements)
                .is_some()
            {
                return Ok(true);
            }

            return Ok(false);
        }

        // No constraints means compatible
        Ok(true)
    }

    /// Get all constraints for a package
    pub fn get_constraints(&self, package: &str) -> Option<&Vec<VersionConstraint>> {
        self.constraints.get(package)
    }

    /// Add edge to dependency graph
    pub fn add_dependency_edge(&mut self, from: &str, to: &str) -> ResolverResult<()> {
        self.graph.add_edge(from, to).map_err(|e| match e {
            graph::GraphError::CircularDependency(msg) => ResolverError::CircularDependency(msg),
            graph::GraphError::PackageNotFound(msg) => ResolverError::PackageNotFound(msg),
        })
    }

    /// Get topological build order
    pub fn compute_build_order(&self) -> ResolverResult<Vec<String>> {
        self.graph.topological_sort().map_err(|e| match e {
            graph::GraphError::CircularDependency(msg) => ResolverError::CircularDependency(msg),
            graph::GraphError::PackageNotFound(msg) => ResolverError::PackageNotFound(msg),
        })
    }

    /// Resolve using existing lockfile if available
    pub fn resolve_with_lockfile(
        &mut self,
        manifest: &PackageManifest,
        lockfile: Option<&crate::lockfile::Lockfile>,
    ) -> ResolverResult<Resolution> {
        // If lockfile exists and is valid, use it
        if let Some(lock) = lockfile {
            if self.lockfile_is_valid(manifest, lock)? {
                return self.resolution_from_lockfile(lock);
            }
        }

        // Otherwise, resolve fresh
        self.resolve(manifest)
    }

    /// Check if lockfile matches manifest constraints
    fn lockfile_is_valid(
        &self,
        manifest: &PackageManifest,
        lockfile: &crate::lockfile::Lockfile,
    ) -> ResolverResult<bool> {
        // Verify lockfile integrity first
        if lockfile.verify().is_err() {
            return Ok(false);
        }

        // Check that all manifest dependencies are in lockfile with compatible versions
        for (name, dep) in &manifest.dependencies {
            let locked_pkg = match lockfile.get_package(name) {
                Some(pkg) => pkg,
                None => return Ok(false), // Missing dependency
            };

            // Get version constraint from dependency
            let version_str = dep.version_constraint().unwrap_or("*");
            let version_req = match version_str.parse::<VersionReq>() {
                Ok(req) => req,
                Err(_) => return Ok(false), // Invalid constraint
            };

            // Check if locked version satisfies manifest constraint
            if !version_req.matches(&locked_pkg.version) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Create resolution from lockfile
    fn resolution_from_lockfile(
        &self,
        lockfile: &crate::lockfile::Lockfile,
    ) -> ResolverResult<Resolution> {
        let mut resolution = Resolution::new();

        for locked_pkg in &lockfile.packages {
            let dependencies: Vec<String> = locked_pkg.dependencies.keys().cloned().collect();

            resolution.add_package(ResolvedPackage {
                name: locked_pkg.name.clone(),
                version: locked_pkg.version.clone(),
                dependencies,
            });
        }

        Ok(resolution)
    }

    /// Generate lockfile from resolution
    pub fn generate_lockfile(&self, resolution: &Resolution) -> crate::lockfile::Lockfile {
        use crate::lockfile::{LockedPackage, LockedSource, Lockfile, LockfileMetadata};

        let mut lockfile = Lockfile::new();

        for (name, package) in &resolution.packages {
            // Build dependencies map
            let mut dependencies = HashMap::new();
            for dep_name in &package.dependencies {
                if let Some(dep_pkg) = resolution.packages.get(dep_name) {
                    dependencies.insert(dep_name.clone(), dep_pkg.version.clone());
                }
            }

            lockfile.add_package(LockedPackage {
                name: name.clone(),
                version: package.version.clone(),
                source: LockedSource::Registry { registry: None },
                checksum: None, // Would come from registry in phase-08b integration
                dependencies,
            });
        }

        // Add metadata
        lockfile.metadata = LockfileMetadata {
            generated_at: Some(chrono::Utc::now().to_rfc3339()),
            atlas_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };

        lockfile
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolved dependency set with exact versions
#[derive(Debug, Clone, PartialEq)]
pub struct Resolution {
    /// Resolved packages with exact versions
    pub packages: HashMap<String, ResolvedPackage>,
}

impl Resolution {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    pub fn add_package(&mut self, package: ResolvedPackage) {
        self.packages.insert(package.name.clone(), package);
    }

    pub fn get_package(&self, name: &str) -> Option<&ResolvedPackage> {
        self.packages.get(name)
    }

    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<String>,
}

impl ResolvedPackage {
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependencies(name: String, version: Version, dependencies: Vec<String>) -> Self {
        Self {
            name,
            version,
            dependencies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_new() {
        let resolver = Resolver::new();
        assert_eq!(resolver.constraints.len(), 0);
    }

    #[test]
    fn test_resolution_new() {
        let resolution = Resolution::new();
        assert_eq!(resolution.package_count(), 0);
    }

    #[test]
    fn test_resolution_add_package() {
        let mut resolution = Resolution::new();
        resolution.add_package(ResolvedPackage::new(
            "test".to_string(),
            Version::new(1, 0, 0),
        ));
        assert_eq!(resolution.package_count(), 1);
    }

    #[test]
    fn test_resolved_package_new() {
        let pkg = ResolvedPackage::new("test".to_string(), Version::new(1, 0, 0));
        assert_eq!(pkg.name, "test");
        assert_eq!(pkg.version, Version::new(1, 0, 0));
        assert_eq!(pkg.dependencies.len(), 0);
    }

    #[test]
    fn test_resolved_package_with_dependencies() {
        let pkg = ResolvedPackage::with_dependencies(
            "test".to_string(),
            Version::new(1, 0, 0),
            vec!["dep1".to_string(), "dep2".to_string()],
        );
        assert_eq!(pkg.dependencies.len(), 2);
    }
}
