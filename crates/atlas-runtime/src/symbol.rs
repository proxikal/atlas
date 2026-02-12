//! Symbol table and name binding

use crate::span::Span;
use crate::types::Type;
use std::collections::HashMap;

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub ty: Type,
    /// Whether the symbol is mutable
    pub mutable: bool,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Declaration location
    pub span: Span,
}

/// Symbol classification
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// Variable binding
    Variable,
    /// Function binding
    Function,
    /// Parameter binding
    Parameter,
    /// Builtin function
    Builtin,
}

/// Symbol table for name resolution
pub struct SymbolTable {
    /// Stack of scopes (innermost last)
    scopes: Vec<HashMap<String, Symbol>>,
    /// Top-level hoisted functions
    functions: HashMap<String, Symbol>,
}

impl SymbolTable {
    /// Create a new symbol table with builtins
    pub fn new() -> Self {
        let mut table = Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        };

        // Add prelude builtins
        table.define_builtin("print", Type::Function {
            params: vec![Type::Unknown], // Accepts any type
            return_type: Box::new(Type::Void),
        });
        table.define_builtin("len", Type::Function {
            params: vec![Type::Unknown], // String or Array
            return_type: Box::new(Type::Number),
        });
        table.define_builtin("str", Type::Function {
            params: vec![Type::Unknown], // Converts any type to string
            return_type: Box::new(Type::String),
        });

        table
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a symbol in the current scope
    /// Returns Err with existing symbol if symbol already exists in current scope
    pub fn define(&mut self, symbol: Symbol) -> Result<(), (String, Option<Symbol>)> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(existing) = scope.get(&symbol.name) {
                return Err((
                    format!("Symbol '{}' is already defined in this scope", symbol.name),
                    Some(existing.clone()),
                ));
            }
            scope.insert(symbol.name.clone(), symbol);
            Ok(())
        } else {
            Err(("No scope to define symbol in".to_string(), None))
        }
    }

    /// Define a top-level function (hoisted)
    /// Returns Err with existing symbol if function already exists
    pub fn define_function(&mut self, symbol: Symbol) -> Result<(), (String, Option<Symbol>)> {
        if let Some(existing) = self.functions.get(&symbol.name) {
            return Err((
                format!("Function '{}' is already defined", symbol.name),
                Some(existing.clone()),
            ));
        }
        self.functions.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Look up a symbol in all scopes (innermost first, then functions)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Check local scopes first (innermost to outermost)
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }

        // Check top-level functions (hoisted)
        self.functions.get(name)
    }

    /// Define a builtin function
    fn define_builtin(&mut self, name: &str, ty: Type) {
        self.functions.insert(name.to_string(), Symbol {
            name: name.to_string(),
            ty,
            mutable: false,
            kind: SymbolKind::Builtin,
            span: Span::dummy(),
        });
    }

    /// Get all symbols from all scopes and functions
    /// Returns a vector of all symbols in the table
    pub fn all_symbols(&self) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        // Collect from all scopes
        for scope in &self.scopes {
            for symbol in scope.values() {
                symbols.push(symbol.clone());
            }
        }

        // Collect from functions (excluding builtins for cleaner output)
        for symbol in self.functions.values() {
            if symbol.kind != SymbolKind::Builtin {
                symbols.push(symbol.clone());
            }
        }

        symbols
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        let result = table.define(Symbol {
            name: "x".to_string(),
            ty: Type::Number,
            mutable: false,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
        });
        assert!(result.is_ok());
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_redeclaration_error() {
        let mut table = SymbolTable::new();
        table.define(Symbol {
            name: "x".to_string(),
            ty: Type::Number,
            mutable: false,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
        }).unwrap();

        let result = table.define(Symbol {
            name: "x".to_string(),
            ty: Type::String,
            mutable: false,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
        });

        assert!(result.is_err());
        let (msg, _) = result.unwrap_err();
        assert!(msg.contains("already defined"));
    }

    #[test]
    fn test_builtin_functions() {
        let table = SymbolTable::new();

        // Check that builtins are defined
        assert!(table.lookup("print").is_some());
        assert!(table.lookup("len").is_some());
        assert!(table.lookup("str").is_some());

        // Check that builtins have correct kind
        assert_eq!(table.lookup("print").unwrap().kind, SymbolKind::Builtin);
    }

    #[test]
    fn test_function_hoisting() {
        let mut table = SymbolTable::new();

        // Define a top-level function
        table.define_function(Symbol {
            name: "foo".to_string(),
            ty: Type::Function {
                params: vec![],
                return_type: Box::new(Type::Void),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: Span::dummy(),
        }).unwrap();

        // Should be able to look it up
        assert!(table.lookup("foo").is_some());

        // Should not be able to redefine
        let result = table.define_function(Symbol {
            name: "foo".to_string(),
            ty: Type::Function {
                params: vec![],
                return_type: Box::new(Type::Void),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: Span::dummy(),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_scope_shadowing() {
        let mut table = SymbolTable::new();

        // Define in outer scope
        table.define(Symbol {
            name: "x".to_string(),
            ty: Type::Number,
            mutable: false,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
        }).unwrap();

        // Enter new scope
        table.enter_scope();

        // Shadow in inner scope
        table.define(Symbol {
            name: "x".to_string(),
            ty: Type::String,
            mutable: false,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
        }).unwrap();

        // Should find inner scope's x
        let symbol = table.lookup("x").unwrap();
        assert_eq!(symbol.ty, Type::String);

        // Exit scope
        table.exit_scope();

        // Should find outer scope's x again
        let symbol = table.lookup("x").unwrap();
        assert_eq!(symbol.ty, Type::Number);
    }
}
