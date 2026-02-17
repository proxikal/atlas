//! Symbol table and name binding

use crate::ast::TypeAliasDecl;
use crate::span::Span;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

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
    /// Whether this symbol is exported (for module system)
    pub exported: bool,
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
#[derive(Clone, Debug)]
pub struct SymbolTable {
    /// Stack of scopes (innermost last)
    scopes: Vec<HashMap<String, Symbol>>,
    /// Top-level hoisted functions
    functions: HashMap<String, Symbol>,
    /// Type alias declarations (name -> alias)
    type_aliases: HashMap<String, TypeAliasDecl>,
    /// Exported type alias names
    type_alias_exports: HashSet<String>,
}

impl SymbolTable {
    /// Create a new symbol table with builtins
    pub fn new() -> Self {
        let mut table = Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            type_aliases: HashMap::new(),
            type_alias_exports: HashSet::new(),
        };

        // Add prelude builtins
        table.define_builtin(
            "print",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Unknown], // Accepts any type
                return_type: Box::new(Type::Void),
            },
        );
        table.define_builtin(
            "len",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Unknown], // String or Array
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "str",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Unknown], // Converts any type to string
                return_type: Box::new(Type::String),
            },
        );

        // String functions - Core Operations
        table.define_builtin(
            "split",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Array(Box::new(Type::String))),
            },
        );
        table.define_builtin(
            "join",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::String)), Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "trim",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "trimStart",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "trimEnd",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );

        // String functions - Search Operations
        table.define_builtin(
            "indexOf",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "lastIndexOf",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "includes",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Bool),
            },
        );

        // String functions - Transformation
        table.define_builtin(
            "toUpperCase",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "toLowerCase",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "substring",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number, Type::Number],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "charAt",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "repeat",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "replace",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String, Type::String],
                return_type: Box::new(Type::String),
            },
        );

        // String functions - Formatting
        table.define_builtin(
            "padStart",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number, Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "padEnd",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number, Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "startsWith",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "endsWith",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Bool),
            },
        );

        // Array functions - Use Unknown for array element types to support any array type
        // This allows string[], number[], etc. to work with these functions
        table.define_builtin(
            "pop",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Unknown))],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "shift",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Unknown))],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "unshift",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Unknown)), Type::Unknown],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "reverse",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Unknown))],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "concat",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Array(Box::new(Type::Unknown)),
                ],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "flatten",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Array(Box::new(Type::Unknown))))],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "arrayIndexOf",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Unknown)), Type::Unknown],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "arrayLastIndexOf",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Unknown)), Type::Unknown],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "arrayIncludes",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Unknown)), Type::Unknown],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "slice",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Number,
                    Type::Number,
                ],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );

        // Array intrinsics (callback-based) - use Unknown for generic array support
        table.define_builtin(
            "map",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Unknown),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "filter",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "reduce",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown, Type::Unknown],
                        return_type: Box::new(Type::Unknown),
                    },
                    Type::Unknown,
                ],
                return_type: Box::new(Type::Unknown),
            },
        );
        table.define_builtin(
            "forEach",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Void),
                    },
                ],
                return_type: Box::new(Type::Null),
            },
        );
        table.define_builtin(
            "find",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Unknown),
            },
        );
        table.define_builtin(
            "findIndex",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "flatMap",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "some",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "every",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "sort",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown, Type::Unknown],
                        return_type: Box::new(Type::Number),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );
        table.define_builtin(
            "sortBy",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::Unknown)),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::Unknown],
                        return_type: Box::new(Type::Number),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::Unknown))),
            },
        );

        // Math functions - Basic Operations
        table.define_builtin(
            "abs",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "floor",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "ceil",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "round",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "min",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "max",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number],
                return_type: Box::new(Type::Number),
            },
        );

        // Math functions - Exponential/Power
        table.define_builtin(
            "sqrt",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "pow",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "log",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );

        // Math functions - Trigonometry
        table.define_builtin(
            "sin",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "cos",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "tan",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "asin",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "acos",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "atan",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );

        // Math functions - Utilities
        table.define_builtin(
            "clamp",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number, Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "sign",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "random",
            Type::Function {
                type_params: vec![],
                params: vec![],
                return_type: Box::new(Type::Number),
            },
        );

        // Math constants (registered as variables, not functions)
        table
            .define(Symbol {
                name: "PI".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok(); // Ignore if already defined

        table
            .define(Symbol {
                name: "E".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        table
            .define(Symbol {
                name: "SQRT2".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        table
            .define(Symbol {
                name: "LN2".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        table
            .define(Symbol {
                name: "LN10".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        table
    }

    /// Define a type alias in the current module
    pub fn define_type_alias(
        &mut self,
        alias: TypeAliasDecl,
    ) -> Result<(), Box<(String, Option<TypeAliasDecl>)>> {
        if let Some(existing) = self.type_aliases.get(&alias.name.name) {
            return Err(Box::new((
                format!("Type alias '{}' already defined", alias.name.name),
                Some(existing.clone()),
            )));
        }
        self.type_aliases.insert(alias.name.name.clone(), alias);
        Ok(())
    }

    /// Look up a type alias by name
    pub fn get_type_alias(&self, name: &str) -> Option<&TypeAliasDecl> {
        self.type_aliases.get(name)
    }

    /// Get all type aliases
    pub fn type_aliases(&self) -> &HashMap<String, TypeAliasDecl> {
        &self.type_aliases
    }

    /// Mark a type alias as exported
    pub fn mark_type_alias_exported(&mut self, name: &str) -> bool {
        if self.type_aliases.contains_key(name) {
            self.type_alias_exports.insert(name.to_string());
            true
        } else {
            false
        }
    }

    /// Get exported type aliases
    pub fn get_type_alias_exports(&self) -> HashMap<String, TypeAliasDecl> {
        self.type_alias_exports
            .iter()
            .filter_map(|name| {
                self.type_aliases
                    .get(name)
                    .cloned()
                    .map(|alias| (name.clone(), alias))
            })
            .collect()
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
    pub fn define(&mut self, symbol: Symbol) -> Result<(), Box<(String, Option<Symbol>)>> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(existing) = scope.get(&symbol.name) {
                return Err(Box::new((
                    format!("Symbol '{}' is already defined in this scope", symbol.name),
                    Some(existing.clone()),
                )));
            }
            scope.insert(symbol.name.clone(), symbol);
            Ok(())
        } else {
            Err(Box::new(("No scope to define symbol in".to_string(), None)))
        }
    }

    /// Define a top-level function (hoisted)
    /// Returns Err with existing symbol if function already exists
    pub fn define_function(&mut self, symbol: Symbol) -> Result<(), Box<(String, Option<Symbol>)>> {
        if let Some(existing) = self.functions.get(&symbol.name) {
            return Err(Box::new((
                format!("Function '{}' is already defined", symbol.name),
                Some(existing.clone()),
            )));
        }
        self.functions.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Define a scoped function (nested function, not hoisted)
    ///
    /// This defines a function in the current scope on the stack, rather than
    /// in the global functions table. Nested functions are not hoisted and
    /// follow normal lexical scoping rules.
    ///
    /// Returns Err with existing symbol if name already exists in current scope
    pub fn define_scoped_function(
        &mut self,
        symbol: Symbol,
    ) -> Result<(), Box<(String, Option<Symbol>)>> {
        // Define in current scope (not global functions HashMap)
        // This allows nested functions to shadow outer functions and follow
        // lexical scoping rules
        self.define(symbol)
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

    /// Look up a symbol mutably in all scopes (innermost first, then functions)
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        // Check local scopes first (innermost to outermost)
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                return scope.get_mut(name);
            }
        }

        // Check top-level functions (hoisted)
        self.functions.get_mut(name)
    }

    /// Define a builtin function
    fn define_builtin(&mut self, name: &str, ty: Type) {
        self.functions.insert(
            name.to_string(),
            Symbol {
                name: name.to_string(),
                ty,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            },
        );
    }

    /// Check if a name is a prelude builtin
    pub fn is_prelude_builtin(&self, name: &str) -> bool {
        if let Some(symbol) = self.functions.get(name) {
            symbol.kind == SymbolKind::Builtin
        } else {
            false
        }
    }

    /// Check if we're currently in the global scope
    pub fn is_global_scope(&self) -> bool {
        self.scopes.len() == 1
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

    /// Merge another symbol table into this one (for REPL state persistence)
    ///
    /// Adds new symbols from the other table to the top-level scope.
    /// Overwrites existing symbols with the same name.
    /// Does not merge nested scopes (only top-level scope and functions).
    pub fn merge(&mut self, other: SymbolTable) {
        // Merge top-level scope (index 0)
        if let Some(other_top_scope) = other.scopes.first() {
            if let Some(self_top_scope) = self.scopes.first_mut() {
                for (name, symbol) in other_top_scope {
                    self_top_scope.insert(name.clone(), symbol.clone());
                }
            }
        }

        // Merge functions (overwrite existing)
        for (name, symbol) in other.functions {
            // Don't overwrite builtins
            if symbol.kind != SymbolKind::Builtin {
                self.functions.insert(name, symbol);
            }
        }
    }

    /// Get all exported symbols from this symbol table
    ///
    /// Returns symbols marked as exported (for module system)
    pub fn get_exports(&self) -> HashMap<String, Symbol> {
        let mut exports = HashMap::new();

        // Check top-level scope for exported symbols
        if let Some(top_scope) = self.scopes.first() {
            for (name, symbol) in top_scope {
                if symbol.exported {
                    exports.insert(name.clone(), symbol.clone());
                }
            }
        }

        // Check top-level functions for exported symbols
        for (name, symbol) in &self.functions {
            if symbol.exported && symbol.kind != SymbolKind::Builtin {
                exports.insert(name.clone(), symbol.clone());
            }
        }

        exports
    }

    /// Mark a symbol as exported
    ///
    /// Used by binder when processing export declarations
    pub fn mark_exported(&mut self, name: &str) -> bool {
        // Check top-level scope first
        if let Some(top_scope) = self.scopes.first_mut() {
            if let Some(symbol) = top_scope.get_mut(name) {
                symbol.exported = true;
                return true;
            }
        }

        // Check top-level functions
        if let Some(symbol) = self.functions.get_mut(name) {
            symbol.exported = true;
            return true;
        }

        false
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
            exported: false,
        });
        assert!(result.is_ok());
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_redeclaration_error() {
        let mut table = SymbolTable::new();
        table
            .define(Symbol {
                name: "x".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Variable,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        let result = table.define(Symbol {
            name: "x".to_string(),
            ty: Type::String,
            mutable: false,
            kind: SymbolKind::Variable,
            span: Span::dummy(),
            exported: false,
        });

        assert!(result.is_err());
        let (msg, _) = *result.unwrap_err();
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
        table
            .define_function(Symbol {
                name: "foo".to_string(),
                ty: Type::Function {
                    type_params: vec![],
                    params: vec![],
                    return_type: Box::new(Type::Void),
                },
                mutable: false,
                kind: SymbolKind::Function,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Should be able to look it up
        assert!(table.lookup("foo").is_some());

        // Should not be able to redefine
        let result = table.define_function(Symbol {
            name: "foo".to_string(),
            ty: Type::Function {
                type_params: vec![],
                params: vec![],
                return_type: Box::new(Type::Void),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: Span::dummy(),
            exported: false,
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_scope_shadowing() {
        let mut table = SymbolTable::new();

        // Define in outer scope
        table
            .define(Symbol {
                name: "x".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Variable,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Enter new scope
        table.enter_scope();

        // Shadow in inner scope
        table
            .define(Symbol {
                name: "x".to_string(),
                ty: Type::String,
                mutable: false,
                kind: SymbolKind::Variable,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Should find inner scope's x
        let symbol = table.lookup("x").unwrap();
        assert_eq!(symbol.ty, Type::String);

        // Exit scope
        table.exit_scope();

        // Should find outer scope's x again
        let symbol = table.lookup("x").unwrap();
        assert_eq!(symbol.ty, Type::Number);
    }

    #[test]
    fn test_scoped_function_definition() {
        let mut table = SymbolTable::new();

        // Enter a nested scope
        table.enter_scope();

        // Define a scoped function (nested function)
        let result = table.define_scoped_function(Symbol {
            name: "helper".to_string(),
            ty: Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: Span::dummy(),
            exported: false,
        });

        assert!(result.is_ok());

        // Should be able to look it up
        assert!(table.lookup("helper").is_some());

        // Exit scope - function should no longer be visible
        table.exit_scope();
        assert!(table.lookup("helper").is_none());
    }

    #[test]
    fn test_scoped_function_shadows_global() {
        let mut table = SymbolTable::new();

        // Define a global function
        table
            .define_function(Symbol {
                name: "foo".to_string(),
                ty: Type::Function {
                    type_params: vec![],
                    params: vec![],
                    return_type: Box::new(Type::Number),
                },
                mutable: false,
                kind: SymbolKind::Function,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Verify we can look up the global function
        let symbol = table.lookup("foo").unwrap();
        assert_eq!(
            symbol.ty,
            Type::Function {
                type_params: vec![],
                params: vec![],
                return_type: Box::new(Type::Number),
            }
        );

        // Enter nested scope
        table.enter_scope();

        // Define a scoped function with same name (shadows global)
        table
            .define_scoped_function(Symbol {
                name: "foo".to_string(),
                ty: Type::Function {
                    type_params: vec![],
                    params: vec![],
                    return_type: Box::new(Type::String),
                },
                mutable: false,
                kind: SymbolKind::Function,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Should find the nested function (shadows global)
        let symbol = table.lookup("foo").unwrap();
        assert_eq!(
            symbol.ty,
            Type::Function {
                type_params: vec![],
                params: vec![],
                return_type: Box::new(Type::String),
            }
        );

        // Exit scope
        table.exit_scope();

        // Should find global function again
        let symbol = table.lookup("foo").unwrap();
        assert_eq!(
            symbol.ty,
            Type::Function {
                type_params: vec![],
                params: vec![],
                return_type: Box::new(Type::Number),
            }
        );
    }

    #[test]
    fn test_scoped_function_shadows_builtin() {
        let mut table = SymbolTable::new();

        // Verify builtin exists
        assert!(table.lookup("print").is_some());
        assert_eq!(table.lookup("print").unwrap().kind, SymbolKind::Builtin);

        // Enter nested scope
        table.enter_scope();

        // Define a scoped function that shadows builtin
        table
            .define_scoped_function(Symbol {
                name: "print".to_string(),
                ty: Type::Function {
                    type_params: vec![],
                    params: vec![Type::String],
                    return_type: Box::new(Type::Void),
                },
                mutable: false,
                kind: SymbolKind::Function,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Should find the nested function (shadows builtin)
        let symbol = table.lookup("print").unwrap();
        assert_eq!(symbol.kind, SymbolKind::Function);

        // Exit scope
        table.exit_scope();

        // Should find builtin again
        let symbol = table.lookup("print").unwrap();
        assert_eq!(symbol.kind, SymbolKind::Builtin);
    }

    #[test]
    fn test_multiple_scoped_functions_same_scope() {
        let mut table = SymbolTable::new();

        // Enter nested scope
        table.enter_scope();

        // Define first scoped function
        table
            .define_scoped_function(Symbol {
                name: "helper1".to_string(),
                ty: Type::Function {
                    type_params: vec![],
                    params: vec![],
                    return_type: Box::new(Type::Number),
                },
                mutable: false,
                kind: SymbolKind::Function,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Define second scoped function in same scope
        table
            .define_scoped_function(Symbol {
                name: "helper2".to_string(),
                ty: Type::Function {
                    type_params: vec![],
                    params: vec![],
                    return_type: Box::new(Type::String),
                },
                mutable: false,
                kind: SymbolKind::Function,
                span: Span::dummy(),
                exported: false,
            })
            .unwrap();

        // Both should be visible
        assert!(table.lookup("helper1").is_some());
        assert!(table.lookup("helper2").is_some());

        // Exit scope - neither should be visible
        table.exit_scope();
        assert!(table.lookup("helper1").is_none());
        assert!(table.lookup("helper2").is_none());
    }
}
