//! Build orchestration and pipeline management
use crate::build_order::{BuildGraph, ModuleNode};
use crate::cache::BuildCache;
use crate::error::{BuildError, BuildResult};
use crate::fingerprint::FingerprintConfig;
use crate::incremental::{IncrementalEngine, IncrementalStats};
use crate::module_resolver::ModuleResolver;
use crate::output::OutputMode;
use crate::profile::{Profile, ProfileManager};
use crate::script::{BuildScript, ScriptContext, ScriptExecutor, ScriptPhase};
use crate::targets::{ArtifactMetadata, BuildArtifact, BuildTarget, TargetKind};

use atlas_package::manifest::PackageManifest;
use atlas_runtime::module_loader::ModuleRegistry;
use atlas_runtime::{
    Binder, Bytecode, Compiler, Diagnostic, Lexer, Parser, SymbolTable, TypeChecker,
};

// Note: Parallel compilation disabled for now due to Bytecode containing non-Send types (Rc<>)
// use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

/// Optimization level for compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OptLevel {
    /// No optimization (fast compilation)
    O0,
    /// Basic optimization
    O1,
    /// Full optimization (default for release)
    O2,
    /// Aggressive optimization
    O3,
}

#[allow(clippy::derivable_impls)]
impl Default for OptLevel {
    fn default() -> Self {
        Self::O0
    }
}

impl OptLevel {
    /// Whether to enable bytecode optimization
    pub fn should_optimize(&self) -> bool {
        !matches!(self, Self::O0)
    }
}

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Target output directory
    pub target_dir: PathBuf,
    /// Optimization level
    pub optimization_level: OptLevel,
    /// Enable parallel compilation
    pub parallel: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target_dir: PathBuf::from("target/debug"),
            optimization_level: OptLevel::O0,
            parallel: true,
            verbose: false,
        }
    }
}

/// Build context - result of a successful build
#[derive(Debug)]
pub struct BuildContext {
    /// Package manifest
    pub manifest: PackageManifest,
    /// Build statistics
    pub stats: BuildStats,
    /// Build artifacts produced
    pub artifacts: Vec<BuildArtifact>,
}

/// Build statistics
#[derive(Debug, Clone)]
pub struct BuildStats {
    /// Total number of modules
    pub total_modules: usize,
    /// Number of modules compiled
    pub compiled_modules: usize,
    /// Number of parallel build groups
    pub parallel_groups: usize,
    /// Total build time
    pub total_time: Duration,
    /// Time spent compiling
    pub compilation_time: Duration,
    /// Time spent linking
    pub linking_time: Duration,
}

impl BuildStats {
    /// Create new build statistics
    pub fn new() -> Self {
        Self {
            total_modules: 0,
            compiled_modules: 0,
            parallel_groups: 0,
            total_time: Duration::ZERO,
            compilation_time: Duration::ZERO,
            linking_time: Duration::ZERO,
        }
    }
}

impl Default for BuildStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiled module result
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for debugging and future features
struct CompiledModule {
    name: String,
    path: PathBuf,
    bytecode: Bytecode,
    compile_time: Duration,
}

/// Main builder for orchestrating builds
pub struct Builder {
    /// Project root directory
    root_dir: PathBuf,
    /// Package manifest
    manifest: PackageManifest,
    /// Build configuration
    config: BuildConfig,
}

impl Builder {
    /// Create a new builder for the project at the given path
    pub fn new(project_path: impl AsRef<Path>) -> BuildResult<Self> {
        let root_dir = project_path.as_ref().to_path_buf();

        // Load package manifest
        let manifest_path = root_dir.join("atlas.toml");
        let manifest = PackageManifest::from_file(&manifest_path)
            .map_err(|e| BuildError::manifest_read(&manifest_path, format!("{:?}", e)))?;

        Ok(Self {
            root_dir,
            manifest,
            config: BuildConfig::default(),
        })
    }

    /// Set build configuration
    pub fn with_config(mut self, config: BuildConfig) -> Self {
        self.config = config;
        self
    }

    /// Set optimization level
    pub fn with_optimization(mut self, level: OptLevel) -> Self {
        self.config.optimization_level = level;
        self
    }

    /// Set target directory
    pub fn with_target_dir(mut self, target_dir: PathBuf) -> Self {
        self.config.target_dir = target_dir;
        self
    }

    /// Enable/disable parallel compilation
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.config.parallel = parallel;
        self
    }

    /// Enable/disable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    /// Execute the build
    pub fn build(&mut self) -> BuildResult<BuildContext> {
        let build_start = Instant::now();

        if self.config.verbose {
            println!(
                "Building {} v{}",
                self.manifest.package.name, self.manifest.package.version
            );
        }

        // Discover source files
        let source_files = self.discover_source_files()?;

        if source_files.is_empty() {
            return Err(BuildError::BuildFailed(
                "No source files found in src/ directory".to_string(),
            ));
        }

        // Build dependency graph from imports
        let graph = self.build_dependency_graph(&source_files)?;

        // Validate graph
        graph.validate()?;

        // Compute build order
        let build_order = if self.config.parallel {
            // Get parallel groups
            let groups = graph.parallel_build_groups()?;
            if self.config.verbose {
                println!("Parallel build groups: {}", groups.len());
            }
            groups
        } else {
            // Sequential build order
            let order = graph.compute_build_order()?;
            vec![order] // Single group containing all modules in order
        };

        // Compile modules
        let compile_start = Instant::now();
        let compiled_modules = self.compile_modules(&graph, &build_order)?;
        let compilation_time = compile_start.elapsed();

        if self.config.verbose {
            println!(
                "Compiled {} modules in {:.2}s",
                compiled_modules.len(),
                compilation_time.as_secs_f64()
            );
        }

        // Create build targets
        let targets = self.create_build_targets(&source_files)?;

        // Link artifacts
        let link_start = Instant::now();
        let artifacts = self.link_artifacts(&targets, &compiled_modules)?;
        let linking_time = link_start.elapsed();

        let total_time = build_start.elapsed();

        // Build statistics
        let stats = BuildStats {
            total_modules: graph.len(),
            compiled_modules: compiled_modules.len(),
            parallel_groups: build_order.len(),
            total_time,
            compilation_time,
            linking_time,
        };

        if self.config.verbose {
            println!("Build completed in {:.2}s", total_time.as_secs_f64());
        }

        Ok(BuildContext {
            manifest: self.manifest.clone(),
            stats,
            artifacts,
        })
    }

    /// Build with incremental compilation (recompile only changed modules)
    pub fn build_incremental(&mut self) -> BuildResult<BuildContext> {
        let build_start = Instant::now();

        if self.config.verbose {
            println!(
                "Building {} v{} (incremental)",
                self.manifest.package.name, self.manifest.package.version
            );
        }

        // Discover source files
        let source_files = self.discover_source_files()?;

        if source_files.is_empty() {
            return Err(BuildError::BuildFailed(
                "No source files found in src/ directory".to_string(),
            ));
        }

        // Build dependency graph
        let graph = self.build_dependency_graph(&source_files)?;
        graph.validate()?;

        // Initialize incremental engine
        let state_dir = self.config.target_dir.join("incremental");
        let fp_config = FingerprintConfig {
            optimization: format!("{:?}", self.config.optimization_level),
            ..Default::default()
        };
        let mut engine = IncrementalEngine::new(state_dir, fp_config);

        // Analyze what needs recompilation
        let analysis_start = Instant::now();
        let plan = engine.plan(&graph)?;
        let analysis_time = analysis_start.elapsed();

        if self.config.verbose {
            println!(
                "Incremental analysis: {} to recompile, {} cached ({:.2}ms)",
                plan.recompile.len(),
                plan.cached.len(),
                analysis_time.as_secs_f64() * 1000.0
            );
            for (name, reason) in &plan.reasons {
                println!("  {} -> {:?}", name, reason);
            }
        }

        // Load build cache for artifact caching
        let cache_dir = self.config.target_dir.join("cache");
        let mut cache = BuildCache::load(&cache_dir)?;

        // Compile modules in topological order with cross-module resolution
        let compile_start = Instant::now();
        let mut compiled_modules = Vec::new();
        let mut resolver = ModuleResolver::new();
        let recompile_set: std::collections::HashSet<String> =
            plan.recompile.iter().cloned().collect();

        let topo_order = graph.compute_build_order()?;
        for module_name in &topo_order {
            let node = graph.get_module(module_name).unwrap();
            let registry = resolver.build_registry_for(&node.dependencies);

            if recompile_set.contains(module_name) {
                // Recompile this module
                let compiled = self.compile_single_module(module_name, &node.path, &registry)?;

                // Read source for fingerprint + cache
                let source =
                    fs::read_to_string(&node.path).map_err(|e| BuildError::io(&node.path, e))?;

                // Record in incremental engine
                let dep_hashes = self.gather_dependency_hashes(&node.dependencies, &engine);
                engine.record_compilation(module_name, &node.path, &source, dep_hashes);

                // Store in build cache
                cache.store(
                    &node.name,
                    node.path.clone(),
                    &source,
                    serialize_bytecode(&compiled.bytecode)?,
                    node.dependencies.clone(),
                    compiled.compile_time,
                )?;

                compiled_modules.push(compiled);
            } else {
                // Cached module - still need to compile for linking
                if self.config.verbose {
                    println!("  Cache hit: {}", module_name);
                }
                let compiled = self.compile_single_module(module_name, &node.path, &registry)?;
                compiled_modules.push(compiled);
            }

            // Register this module's exports for downstream dependents
            let symbol_table = self.extract_symbol_table(module_name, &node.path, &registry)?;
            resolver.register_module(module_name.clone(), node.path.clone(), symbol_table);
        }

        let compilation_time = compile_start.elapsed();

        // Update and persist state
        engine.update_state(&graph);
        engine.save()?;
        cache.save()?;

        // Create build targets
        let targets = self.create_build_targets(&source_files)?;

        // Link artifacts
        let link_start = Instant::now();
        let artifacts = self.link_artifacts(&targets, &compiled_modules)?;
        let linking_time = link_start.elapsed();

        let total_time = build_start.elapsed();

        // Build statistics
        let stats = BuildStats {
            total_modules: graph.len(),
            compiled_modules: compiled_modules.len(),
            parallel_groups: 1,
            total_time,
            compilation_time,
            linking_time,
        };

        if self.config.verbose {
            let inc_stats = IncrementalStats {
                total_modules: graph.len(),
                recompiled: recompile_set.len(),
                from_cache: graph.len() - recompile_set.len(),
                analysis_time,
                compilation_time,
                time_saved: Duration::ZERO, // Would need baseline to compute
                was_full_rebuild: recompile_set.len() == graph.len(),
            };
            println!("{}", inc_stats.summary());
        }

        Ok(BuildContext {
            manifest: self.manifest.clone(),
            stats,
            artifacts,
        })
    }

    /// Gather dependency hashes for fingerprinting
    fn gather_dependency_hashes(
        &self,
        dependencies: &[String],
        engine: &IncrementalEngine,
    ) -> std::collections::BTreeMap<String, String> {
        dependencies
            .iter()
            .filter_map(|dep| {
                engine
                    .fingerprint_db()
                    .get(dep)
                    .map(|fp| (dep.clone(), fp.hash.clone()))
            })
            .collect()
    }

    /// Compile a single module (standalone, no cross-module resolution).
    ///
    /// Used by incremental builds where modules are compiled individually.
    /// For cross-module support, use `compile_module_with_imports`.
    fn compile_single_module(
        &self,
        module_name: &str,
        source_path: &Path,
        registry: &ModuleRegistry,
    ) -> BuildResult<CompiledModule> {
        self.compile_module_with_imports(module_name, source_path, registry)
    }

    /// Discover all source files in the project
    fn discover_source_files(&self) -> BuildResult<Vec<PathBuf>> {
        let src_dir = self.root_dir.join("src");

        if !src_dir.exists() {
            return Err(BuildError::BuildFailed(format!(
                "Source directory not found: {}",
                src_dir.display()
            )));
        }

        let mut source_files = Vec::new();

        for entry in WalkDir::new(&src_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("atlas") {
                    source_files.push(path.to_path_buf());
                }
            }
        }

        Ok(source_files)
    }

    /// Build dependency graph from source files
    fn build_dependency_graph(&self, source_files: &[PathBuf]) -> BuildResult<BuildGraph> {
        let mut graph = BuildGraph::new();

        for source_path in source_files {
            // Parse file to extract imports
            let source =
                fs::read_to_string(source_path).map_err(|e| BuildError::io(source_path, e))?;

            let module_name = self.path_to_module_name(source_path)?;

            // Quick parse to get imports (don't need full type checking yet)
            let mut lexer = Lexer::new(&source);
            let (tokens, lex_diagnostics) = lexer.tokenize();

            if !lex_diagnostics.is_empty() {
                return Err(BuildError::compilation(
                    &module_name,
                    format_diagnostics(&lex_diagnostics),
                ));
            }

            let mut parser = Parser::new(tokens);
            let (program, parse_diagnostics) = parser.parse();

            if !parse_diagnostics.is_empty() {
                return Err(BuildError::compilation(
                    &module_name,
                    format_diagnostics(&parse_diagnostics),
                ));
            }

            // Extract dependencies from imports
            let dependencies = program
                .items
                .iter()
                .filter_map(|item| {
                    if let atlas_runtime::ast::Item::Import(import_decl) = item {
                        Some(import_decl.source.clone())
                    } else {
                        None
                    }
                })
                .collect();

            let module =
                ModuleNode::new(module_name, source_path.clone()).with_dependencies(dependencies);

            graph.add_module(module);
        }

        Ok(graph)
    }

    /// Compile modules in topological order with cross-module symbol resolution.
    ///
    /// After each module compiles, its exports are registered so that
    /// dependent modules can resolve their imports during binding.
    fn compile_modules(
        &self,
        graph: &BuildGraph,
        build_order: &[Vec<String>],
    ) -> BuildResult<Vec<CompiledModule>> {
        let mut compiled = Vec::new();
        let mut resolver = ModuleResolver::new();

        for (group_idx, group) in build_order.iter().enumerate() {
            if self.config.verbose {
                println!(
                    "Compiling group {} ({} modules)",
                    group_idx + 1,
                    group.len()
                );
            }

            // Compile each module in the group, injecting dependency exports
            for module_name in group {
                let module = graph
                    .get_module(module_name)
                    .ok_or_else(|| BuildError::module_not_found(module_name))?;

                // Build a registry containing only this module's dependencies
                let registry = resolver.build_registry_for(&module.dependencies);

                let compiled_module =
                    self.compile_module_with_imports(module_name, &module.path, &registry)?;

                // Register this module's symbol table for downstream dependents
                let symbol_table =
                    self.extract_symbol_table(module_name, &module.path, &registry)?;
                resolver.register_module(module_name.clone(), module.path.clone(), symbol_table);

                compiled.push(compiled_module);
            }
        }

        Ok(compiled)
    }

    /// Compile a single module with cross-module import resolution.
    fn compile_module_with_imports(
        &self,
        module_name: &str,
        source_path: &Path,
        registry: &ModuleRegistry,
    ) -> BuildResult<CompiledModule> {
        let compile_start = Instant::now();

        if self.config.verbose {
            println!("  Compiling {}", module_name);
        }

        // Read source
        let source = fs::read_to_string(source_path).map_err(|e| BuildError::io(source_path, e))?;

        // Lex
        let mut lexer = Lexer::new(&source);
        let (tokens, lex_diagnostics) = lexer.tokenize();

        if !lex_diagnostics.is_empty() {
            return Err(BuildError::compilation(
                module_name,
                format_diagnostics(&lex_diagnostics),
            ));
        }

        // Parse
        let mut parser = Parser::new(tokens);
        let (program, parse_diagnostics) = parser.parse();

        if !parse_diagnostics.is_empty() {
            return Err(BuildError::compilation(
                module_name,
                format_diagnostics(&parse_diagnostics),
            ));
        }

        // Bind with cross-module support
        let mut binder = Binder::new();
        let (mut symbol_table, bind_diagnostics) =
            binder.bind_with_modules(&program, source_path, registry);

        if !bind_diagnostics.is_empty() {
            return Err(BuildError::compilation(
                module_name,
                format_diagnostics(&bind_diagnostics),
            ));
        }

        // Type check
        let mut type_checker = TypeChecker::new(&mut symbol_table);
        let type_diagnostics = type_checker.check(&program);

        if !type_diagnostics.is_empty() {
            return Err(BuildError::compilation(
                module_name,
                format_diagnostics(&type_diagnostics),
            ));
        }

        // Compile to bytecode
        let mut compiler = if self.config.optimization_level.should_optimize() {
            Compiler::with_optimization()
        } else {
            Compiler::new()
        };

        let bytecode = compiler.compile(&program).map_err(|diagnostics| {
            BuildError::compilation(module_name, format_diagnostics(&diagnostics))
        })?;

        let compile_time = compile_start.elapsed();

        Ok(CompiledModule {
            name: module_name.to_string(),
            path: source_path.to_path_buf(),
            bytecode,
            compile_time,
        })
    }

    /// Extract a module's symbol table (for registering exports).
    ///
    /// Re-runs lex/parse/bind to get the symbol table with exports marked.
    /// This is separate from compilation because the compiler consumes the program.
    fn extract_symbol_table(
        &self,
        module_name: &str,
        source_path: &Path,
        registry: &ModuleRegistry,
    ) -> BuildResult<SymbolTable> {
        let source = fs::read_to_string(source_path).map_err(|e| BuildError::io(source_path, e))?;

        let mut lexer = Lexer::new(&source);
        let (tokens, _) = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();

        let mut binder = Binder::new();
        let (symbol_table, bind_diagnostics) =
            binder.bind_with_modules(&program, source_path, registry);

        if !bind_diagnostics.is_empty() {
            return Err(BuildError::compilation(
                module_name,
                format_diagnostics(&bind_diagnostics),
            ));
        }

        Ok(symbol_table)
    }

    /// Create build targets from source files
    fn create_build_targets(&self, source_files: &[PathBuf]) -> BuildResult<Vec<BuildTarget>> {
        let mut targets = Vec::new();

        // Determine if this is a library or binary based on lib.atlas vs main.atlas
        let has_lib = source_files.iter().any(|p| p.ends_with("lib.atlas"));
        let has_main = source_files.iter().any(|p| p.ends_with("main.atlas"));

        if has_lib {
            // Library target
            let target = BuildTarget::new(self.manifest.package.name.as_str(), TargetKind::Library)
                .with_sources(source_files.to_vec());
            targets.push(target);
        }

        if has_main {
            // Binary target
            let target = BuildTarget::new(self.manifest.package.name.as_str(), TargetKind::Binary)
                .with_entry_point("src/main.atlas")
                .with_sources(source_files.to_vec());
            targets.push(target);
        }

        if targets.is_empty() {
            return Err(BuildError::BuildFailed(
                "No lib.atlas or main.atlas found in src/".to_string(),
            ));
        }

        // Validate targets
        for target in &targets {
            target.validate().map_err(BuildError::InvalidTarget)?;
        }

        Ok(targets)
    }

    /// Link compiled modules into artifacts
    fn link_artifacts(
        &self,
        targets: &[BuildTarget],
        compiled_modules: &[CompiledModule],
    ) -> BuildResult<Vec<BuildArtifact>> {
        let mut artifacts = Vec::new();

        for target in targets {
            if self.config.verbose {
                println!("Linking {} target '{}'", target.kind, target.name);
            }

            // Combine all bytecode from modules
            // For now, we just concatenate bytecode (simplified linking)
            // TODO: Proper linking with module resolution in future phase
            let mut combined_bytecode = Vec::new();
            let mut total_compile_time = Duration::ZERO;

            for module in compiled_modules {
                let bytes = serialize_bytecode(&module.bytecode)?;
                combined_bytecode.extend_from_slice(&bytes);
                total_compile_time += module.compile_time;
            }

            // Create output directory
            let output_dir = self.config.target_dir.join(target.kind.output_dir_name());
            fs::create_dir_all(&output_dir).map_err(|e| BuildError::io(&output_dir, e))?;

            // Write artifact
            let output_path = output_dir.join(target.output_filename());
            fs::write(&output_path, &combined_bytecode)
                .map_err(|e| BuildError::io(&output_path, e))?;

            let metadata = ArtifactMetadata::new(
                total_compile_time,
                compiled_modules.len(),
                combined_bytecode.len(),
            );

            artifacts.push(BuildArtifact::new(
                target.clone(),
                output_path,
                combined_bytecode,
                metadata,
            ));
        }

        Ok(artifacts)
    }

    /// Convert file path to module name
    fn path_to_module_name(&self, path: &Path) -> BuildResult<String> {
        let src_dir = self.root_dir.join("src");
        let relative = path.strip_prefix(&src_dir).map_err(|_| {
            BuildError::BuildFailed(format!("Path {} is not under src/", path.display()))
        })?;

        let module_name = relative
            .with_extension("")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "::");

        Ok(module_name)
    }

    /// Build with specific profile
    pub fn build_with_profile(
        &mut self,
        profile: Profile,
        scripts: &[BuildScript],
        output_mode: OutputMode,
    ) -> BuildResult<BuildContext> {
        // Load profile manager and get profile config
        let profile_manager = ProfileManager::new();

        // Load custom profiles from manifest if any
        // TODO: Add profile section to PackageManifest

        let profile_config = profile_manager.get(&profile)?;

        // Apply profile configuration to build config
        self.config.optimization_level = profile_config.optimization_level;
        self.config.verbose = matches!(output_mode, OutputMode::Verbose);

        // Create script context
        let script_context = ScriptContext::new(
            profile.clone(),
            self.config.target_dir.clone(),
            self.root_dir.join("src"),
            self.manifest.package.name.clone(),
            self.manifest.package.version.to_string(),
        );

        let script_executor = ScriptExecutor::new(script_context).with_verbose(self.config.verbose);

        // Execute pre-build scripts
        script_executor.execute_phase(scripts, ScriptPhase::PreBuild)?;

        // Perform build (incremental if profile allows)
        let context = if profile_config.incremental {
            self.build_incremental()?
        } else {
            self.build()?
        };

        // Execute post-build scripts
        script_executor.execute_phase(scripts, ScriptPhase::PostBuild)?;

        // Execute post-link scripts
        script_executor.execute_phase(scripts, ScriptPhase::PostLink)?;

        Ok(context)
    }

    /// Set profile
    pub fn with_profile(self, _profile: Profile) -> Self {
        // Profile will be applied in build_with_profile
        self
    }

    /// Set output mode
    pub fn with_output_mode(self, _mode: OutputMode) -> Self {
        // Output mode will be applied in build_with_profile
        self
    }

    /// Clean build artifacts
    pub fn clean(&mut self) -> BuildResult<()> {
        let target_dir = &self.config.target_dir;
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| BuildError::io(target_dir, e))?;
        }
        Ok(())
    }
}

/// Format diagnostics for error messages
fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|d| d.message.clone())
        .collect::<Vec<_>>()
        .join("; ")
}

/// Serialize bytecode to bytes
/// TODO: Implement proper bytecode serialization format
/// For now, this is a placeholder - in future phases we'll add proper serialization
fn serialize_bytecode(_bytecode: &Bytecode) -> BuildResult<Vec<u8>> {
    // Placeholder: Return empty vec for now
    // Phase-11b or later will implement proper bytecode serialization
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_level_should_optimize() {
        assert!(!OptLevel::O0.should_optimize());
        assert!(OptLevel::O1.should_optimize());
        assert!(OptLevel::O2.should_optimize());
        assert!(OptLevel::O3.should_optimize());
    }

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert_eq!(config.optimization_level, OptLevel::O0);
        assert!(config.parallel);
        assert!(!config.verbose);
    }

    #[test]
    fn test_build_stats_default() {
        let stats = BuildStats::default();
        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.compiled_modules, 0);
        assert_eq!(stats.parallel_groups, 0);
    }
}
