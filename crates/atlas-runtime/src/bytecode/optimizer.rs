//! Bytecode optimizer
//!
//! Provides a pass manager infrastructure for bytecode optimizations.
//! In v0.1, this is a skeleton with placeholders - actual optimizations
//! will be added in future versions.

use super::Bytecode;

/// Optimization pass trait
///
/// Each optimization pass implements this trait to transform bytecode.
/// Passes should preserve program semantics.
pub trait OptimizationPass {
    /// Name of this optimization pass (for debugging)
    fn name(&self) -> &str;

    /// Apply this optimization pass to bytecode
    ///
    /// Returns the optimized bytecode. May return the same bytecode
    /// if no optimizations were applicable.
    fn optimize(&self, bytecode: Bytecode) -> Bytecode;
}

/// Bytecode optimizer with configurable passes
///
/// In v0.1, this is a skeleton. Future versions will add:
/// - Constant folding
/// - Dead code elimination
/// - Peephole optimizations
/// - Jump threading
pub struct Optimizer {
    /// Whether optimization is enabled
    enabled: bool,
    /// Registered optimization passes
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl Optimizer {
    /// Create a new optimizer with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::bytecode::Optimizer;
    ///
    /// let optimizer = Optimizer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            enabled: false, // Disabled by default in v0.1
            passes: Vec::new(),
        }
    }

    /// Create an optimizer with all default passes enabled
    ///
    /// In v0.1, this is the same as `new()` with enabled set to true.
    /// Future versions will register actual optimization passes.
    pub fn with_default_passes() -> Self {
        let mut optimizer = Self {
            enabled: true,
            passes: Vec::new(),
        };

        // Register placeholder passes
        optimizer.add_pass(Box::new(ConstantFoldingPass));

        optimizer
    }

    /// Enable or disable optimization
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if optimization is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add an optimization pass
    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }

    /// Optimize bytecode by running all registered passes
    ///
    /// If optimization is disabled, returns the bytecode unchanged.
    pub fn optimize(&self, bytecode: Bytecode) -> Bytecode {
        if !self.enabled {
            return bytecode;
        }

        let mut result = bytecode;
        for pass in &self.passes {
            result = pass.optimize(result);
        }
        result
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Constant folding optimization pass (placeholder)
///
/// In v0.1, this is a no-op placeholder. Future versions will implement:
/// - Compile-time evaluation of constant expressions
/// - Folding arithmetic operations on constants
/// - Propagating constant values
///
/// Example transformations (future):
/// ```text
/// Constant 2
/// Constant 3
/// Add
/// ```
/// becomes:
/// ```text
/// Constant 5
/// ```
pub struct ConstantFoldingPass;

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &str {
        "constant-folding"
    }

    fn optimize(&self, bytecode: Bytecode) -> Bytecode {
        // TODO: Implement constant folding in future version
        // For now, return bytecode unchanged
        bytecode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use crate::bytecode::Opcode;

    #[test]
    fn test_optimizer_new() {
        let optimizer = Optimizer::new();
        assert!(!optimizer.is_enabled());
    }

    #[test]
    fn test_optimizer_with_default_passes() {
        let optimizer = Optimizer::with_default_passes();
        assert!(optimizer.is_enabled());
        assert_eq!(optimizer.passes.len(), 1); // Constant folding pass
    }

    #[test]
    fn test_optimizer_enable_disable() {
        let mut optimizer = Optimizer::new();
        assert!(!optimizer.is_enabled());

        optimizer.set_enabled(true);
        assert!(optimizer.is_enabled());

        optimizer.set_enabled(false);
        assert!(!optimizer.is_enabled());
    }

    #[test]
    fn test_optimizer_disabled_returns_unchanged() {
        let optimizer = Optimizer::new();
        assert!(!optimizer.is_enabled());

        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Halt, Span::dummy());

        let original_len = bytecode.instructions.len();
        let optimized = optimizer.optimize(bytecode);

        // Should be unchanged when disabled
        assert_eq!(optimized.instructions.len(), original_len);
    }

    #[test]
    fn test_optimizer_with_passes() {
        let mut optimizer = Optimizer::new();
        optimizer.set_enabled(true);
        optimizer.add_pass(Box::new(ConstantFoldingPass));

        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Null, Span::dummy());
        bytecode.emit(Opcode::Halt, Span::dummy());

        let optimized = optimizer.optimize(bytecode);
        // Passes are no-ops in v0.1, but should run without error
        assert_eq!(optimized.instructions.len(), 2);
    }

    #[test]
    fn test_constant_folding_pass_name() {
        let pass = ConstantFoldingPass;
        assert_eq!(pass.name(), "constant-folding");
    }

    #[test]
    fn test_constant_folding_pass_is_noop() {
        // In v0.1, constant folding is a no-op placeholder
        let pass = ConstantFoldingPass;
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(1);
        bytecode.emit(Opcode::Add, Span::dummy());

        let original_len = bytecode.instructions.len();
        let optimized = pass.optimize(bytecode);

        // Should be unchanged (no optimization implemented yet)
        assert_eq!(optimized.instructions.len(), original_len);
    }

    #[test]
    fn test_add_multiple_passes() {
        let mut optimizer = Optimizer::new();
        optimizer.add_pass(Box::new(ConstantFoldingPass));
        optimizer.add_pass(Box::new(ConstantFoldingPass)); // Can add same type multiple times

        assert_eq!(optimizer.passes.len(), 2);
    }

    #[test]
    fn test_optimizer_default() {
        let optimizer = Optimizer::default();
        assert!(!optimizer.is_enabled());
        assert_eq!(optimizer.passes.len(), 0);
    }
}
