//! Atlas JIT Compilation Engine
//!
//! Provides just-in-time compilation for Atlas bytecode using Cranelift
//! as the native code generation backend. Hot functions identified by
//! the VM profiler are compiled to native machine code for 5-10x speedup.
//!
//! # Status: Foundation Complete — Not Yet Wired to Production
//!
//! The JIT compiles **arithmetic-only** functions (numeric constants, local variables,
//! arithmetic operators, comparisons). It does NOT support control flow (jump/call),
//! global variables, or collection opcodes. See `JIT_STATUS.md` for the full capability
//! matrix and v0.3 integration requirements.
//!
//! ## Supported Opcodes
//!
//! `Constant`, `True`, `False`, `Null`, `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate`,
//! `Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`, `Not`,
//! `GetLocal`, `SetLocal`, `Pop`, `Dup`, `Return`, `Halt`
//!
//! ## Unsupported Opcodes (bail out to interpreter)
//!
//! `GetGlobal`, `SetGlobal`, `Jump`, `JumpIfFalse`, `Loop`, `Call`, `And`, `Or`,
//! `Array`, `GetIndex`, `SetIndex`, `IsOptionSome`, `IsOptionNone`, `IsResultOk`,
//! `IsResultErr`, `ExtractOptionValue`, `ExtractResultValue`, `IsArray`, `GetArrayLen`

pub mod backend;
pub mod cache;
pub mod codegen;
pub mod hotspot;

use thiserror::Error;

/// JIT compilation errors
#[derive(Debug, Error)]
pub enum JitError {
    #[error("compilation failed: {0}")]
    CompilationFailed(String),

    #[error("unsupported opcode: {0:?}")]
    UnsupportedOpcode(atlas_runtime::bytecode::Opcode),

    #[error("code cache full (limit: {limit} bytes, used: {used} bytes)")]
    CacheFull { limit: usize, used: usize },

    #[error("invalid bytecode: {0}")]
    InvalidBytecode(String),

    #[error("native execution error: {0}")]
    ExecutionError(String),
}

/// Result type for JIT operations
pub type JitResult<T> = Result<T, JitError>;

/// Configuration for the JIT compiler
#[derive(Debug, Clone)]
pub struct JitConfig {
    /// Minimum execution count before a function is JIT-compiled
    pub compilation_threshold: u64,
    /// Maximum bytes of native code to cache
    pub cache_size_limit: usize,
    /// Whether to enable JIT compilation
    pub enabled: bool,
    /// Optimization level for Cranelift (0=none, 1=speed, 2=speed+size)
    pub opt_level: u8,
}

impl Default for JitConfig {
    fn default() -> Self {
        Self {
            compilation_threshold: 100,
            cache_size_limit: 64 * 1024 * 1024, // 64 MB
            enabled: true,
            opt_level: 1,
        }
    }
}

impl JitConfig {
    /// Create a config suitable for testing (low thresholds)
    pub fn for_testing() -> Self {
        Self {
            compilation_threshold: 2,
            cache_size_limit: 4 * 1024 * 1024,
            enabled: true,
            opt_level: 0,
        }
    }
}

/// The main JIT engine — integrates hotspot tracking, compilation, caching,
/// and native execution dispatch.
///
/// Attach this to a VM to enable tiered compilation: functions start
/// interpreted, and once called enough times they are compiled to native
/// code for subsequent invocations.
pub struct JitEngine {
    config: JitConfig,
    tracker: hotspot::HotspotTracker,
    cache: cache::CodeCache,
    backend: backend::NativeBackend,
    translator: codegen::IrTranslator,
    /// Total number of JIT compilations performed
    compilations: u64,
    /// Total number of JIT executions (cache hits that ran native code)
    jit_executions: u64,
    /// Total number of interpreter fallbacks
    interpreter_fallbacks: u64,
}

impl JitEngine {
    /// Create a new JIT engine with the given configuration
    pub fn new(config: JitConfig) -> JitResult<Self> {
        let backend = backend::NativeBackend::new(config.opt_level)?;
        Ok(Self {
            tracker: hotspot::HotspotTracker::new(config.compilation_threshold),
            cache: cache::CodeCache::new(config.cache_size_limit),
            translator: codegen::IrTranslator::new(config.opt_level),
            backend,
            config,
            compilations: 0,
            jit_executions: 0,
            interpreter_fallbacks: 0,
        })
    }

    /// Record a function call and potentially trigger JIT compilation
    ///
    /// Returns `Some(result)` if the function was executed via JIT,
    /// or `None` if the interpreter should handle it.
    pub fn notify_call(
        &mut self,
        function_offset: usize,
        bytecode: &atlas_runtime::bytecode::Bytecode,
        function_end: usize,
    ) -> Option<f64> {
        if !self.config.enabled {
            return None;
        }

        self.tracker.record_call(function_offset);

        // Check if already cached
        if self.cache.contains(function_offset) {
            if let Some(entry) = self.cache.get(function_offset) {
                let result = unsafe {
                    let func: unsafe fn() -> f64 = std::mem::transmute(entry.code_ptr);
                    func()
                };
                self.jit_executions += 1;
                return Some(result);
            }
        }

        // Check if hot enough to compile
        if self.tracker.is_hot(function_offset) {
            match self.try_compile(function_offset, bytecode, function_end) {
                Ok(result) => {
                    self.jit_executions += 1;
                    return Some(result);
                }
                Err(_) => {
                    // Compilation failed — mark as compiled to avoid retrying
                    self.tracker.mark_compiled(function_offset);
                    self.interpreter_fallbacks += 1;
                }
            }
        }

        None
    }

    /// Try to compile a function and execute it
    fn try_compile(
        &mut self,
        offset: usize,
        bytecode: &atlas_runtime::bytecode::Bytecode,
        end: usize,
    ) -> JitResult<f64> {
        let func = self.translator.translate(bytecode, offset, end)?;
        let compiled = self.backend.compile(func)?;

        let result = unsafe { compiled.call_no_args() };

        self.cache
            .insert(offset, compiled.code_ptr, 64, 0)
            .map_err(|e| JitError::CacheFull {
                limit: e.limit,
                used: e.used,
            })?;

        self.tracker.mark_compiled(offset);
        self.compilations += 1;

        Ok(result)
    }

    /// Whether JIT is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable JIT compilation
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable JIT compilation (existing cache is preserved)
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Get statistics about the JIT engine
    pub fn stats(&self) -> JitStats {
        JitStats {
            compilations: self.compilations,
            jit_executions: self.jit_executions,
            interpreter_fallbacks: self.interpreter_fallbacks,
            cached_functions: self.cache.len(),
            cache_bytes: self.cache.total_bytes(),
            cache_hit_rate: self.cache.hit_rate(),
            tracked_functions: self.tracker.tracked_count(),
            compiled_functions: self.tracker.compiled_count(),
        }
    }

    /// Reset all state
    pub fn reset(&mut self) {
        self.tracker.reset();
        self.cache.clear();
        self.compilations = 0;
        self.jit_executions = 0;
        self.interpreter_fallbacks = 0;
    }

    /// Get the compilation threshold
    pub fn threshold(&self) -> u64 {
        self.config.compilation_threshold
    }

    /// Invalidate all cached native code
    pub fn invalidate_cache(&mut self) {
        self.cache.invalidate_all();
    }
}

/// Statistics from the JIT engine
#[derive(Debug, Clone)]
pub struct JitStats {
    /// Total JIT compilations performed
    pub compilations: u64,
    /// Total native code executions
    pub jit_executions: u64,
    /// Total interpreter fallbacks (JIT failed)
    pub interpreter_fallbacks: u64,
    /// Number of functions in the code cache
    pub cached_functions: usize,
    /// Total bytes of cached native code
    pub cache_bytes: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Number of functions being tracked
    pub tracked_functions: usize,
    /// Number of functions that have been compiled
    pub compiled_functions: usize,
}
