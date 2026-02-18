//! Atlas Runtime - Core language implementation
//!
//! This library provides the complete Atlas language runtime including:
//! - Lexical analysis and parsing
//! - Type checking and binding
//! - Interpretation and bytecode compilation
//! - Standard library functions

/// Atlas runtime version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Public API modules
pub mod api;
pub mod ast;
pub mod async_runtime;
pub mod binder;
pub mod bytecode;
pub mod compiler;
pub mod debugger;
pub mod diagnostic;
pub mod ffi;
pub mod interpreter;
pub mod json_value;
pub mod lexer;
pub mod method_dispatch;
pub mod module_executor;
pub mod module_loader;
pub mod optimizer;
pub mod parser;
pub mod profiler;
pub mod reflect;
pub mod repl;
pub mod resolver;
pub mod runtime;
pub mod security;
pub mod sourcemap;
pub mod span;
pub mod stdlib;
pub mod symbol;
pub mod token;
pub mod typecheck_dump;
pub mod typechecker;
pub mod types;
pub mod value;
pub mod vm;

// Test utilities (only available in test builds)
#[cfg(test)]
pub mod test_utils;

// Re-export commonly used types
pub use binder::Binder;
pub use bytecode::{Bytecode, Opcode};
pub use compiler::Compiler;
pub use diagnostic::{
    error_codes, formatter, normalizer, sort_diagnostics, warnings, Diagnostic, DiagnosticLevel,
    RelatedLocation, DIAG_VERSION,
};
pub use interpreter::Interpreter;
pub use json_value::JsonValue;
pub use lexer::Lexer;
pub use module_executor::ModuleExecutor;
pub use module_loader::{LoadedModule, ModuleLoader};
pub use parser::Parser;
pub use repl::{ReplCore, ReplResult};
pub use resolver::ModuleResolver;
pub use runtime::{Atlas, RuntimeResult};
pub use security::{
    AuditEntry, AuditEvent, AuditLogger, MemoryAuditLogger, NullAuditLogger, Permission,
    PermissionSet, SecurityContext, SecurityError,
};
pub use span::Span;
pub use symbol::{Symbol, SymbolKind, SymbolTable};
pub use token::{Token, TokenKind};
pub use typecheck_dump::{SymbolInfo, TypeInfo, TypecheckDump, TYPECHECK_VERSION};
pub use typechecker::TypeChecker;
pub use types::Type;
pub use value::{FunctionRef, RuntimeError, Value};
pub use vm::VM;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke() {
        // Smoke test to verify the crate builds and tests run
        assert_eq!(VERSION, "0.1.0");
    }
}
