//! Bytecode optimizer — re-export shim
//!
//! The full optimizer implementation lives in `crate::optimizer`.
//! This module re-exports the public API so that existing code using
//! `crate::bytecode::Optimizer` continues to work unchanged.

pub use crate::optimizer::{
    ConstantFoldingPass, DeadCodeEliminationPass, OptimizationPass, OptimizationStats, Optimizer,
    PeepholePass,
};
