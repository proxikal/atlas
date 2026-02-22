//! Document and workspace symbol providers
//!
//! Provides symbol extraction for:
//! - Document outline (textDocument/documentSymbol)
//! - Workspace symbol search (workspace/symbol)
//! - Query caching and performance optimization
//! - Memory-bounded indexing for large workspaces

use atlas_runtime::ast::*;
use atlas_runtime::span::Span;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use tower_lsp::lsp_types::{
    DocumentSymbol, Location, Position, Range, SymbolInformation, SymbolKind, Url,
};

/// Symbol with location info for workspace indexing
#[derive(Debug, Clone)]
pub struct IndexedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub container_name: Option<String>,
}

/// Configuration for workspace indexing
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// Maximum number of symbols to index (prevents unbounded memory growth)
    pub max_symbols: usize,
    /// Maximum number of cached query results
    pub cache_size: usize,
    /// Enable parallel indexing for large files
    pub parallel_indexing: bool,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            max_symbols: 100_000,    // 100k symbols max
            cache_size: 100,         // Cache 100 query results
            parallel_indexing: true, // Enable parallel by default
        }
    }
}

/// Cache key for query results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct QueryCacheKey {
    query: String,
    /// Kind filter as string representation, None = no filter
    kind_filter: Option<String>,
    limit: usize,
}

/// Workspace symbol index for fast searching
pub struct WorkspaceIndex {
    symbols: HashMap<Url, Vec<IndexedSymbol>>,
    query_cache: Arc<RwLock<LruCache<QueryCacheKey, Vec<SymbolInformation>>>>,
    config: IndexConfig,
}

impl Default for WorkspaceIndex {
    fn default() -> Self {
        Self::with_config(IndexConfig::default())
    }
}

impl WorkspaceIndex {
    /// Create a new workspace index with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new workspace index with custom configuration
    pub fn with_config(config: IndexConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap();
        Self {
            symbols: HashMap::new(),
            query_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            config,
        }
    }

    /// Index symbols from a document
    pub fn index_document(&mut self, uri: Url, text: &str, ast: &Program) {
        let symbols = extract_indexed_symbols(&uri, text, ast);

        // Check memory bounds
        let total_symbols: usize = self.symbols.values().map(|v| v.len()).sum();
        if total_symbols + symbols.len() > self.config.max_symbols {
            // Remove symbols from oldest document to stay within bounds
            if let Some(oldest_uri) = self.symbols.keys().next().cloned() {
                self.symbols.remove(&oldest_uri);
            }
        }

        self.symbols.insert(uri, symbols);

        // Invalidate query cache when index changes
        self.invalidate_cache();
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, uri: &Url) {
        self.symbols.remove(uri);

        // Invalidate cache when index changes
        self.invalidate_cache();
    }

    /// Invalidate the query cache
    fn invalidate_cache(&self) {
        if let Ok(mut cache) = self.query_cache.write() {
            cache.clear();
        }
    }

    /// Search for symbols matching a query with optional kind filtering
    pub fn search(
        &self,
        query: &str,
        limit: usize,
        kind_filter: Option<SymbolKind>,
    ) -> Vec<SymbolInformation> {
        // Check cache first - convert kind to string for cache key
        let cache_key = QueryCacheKey {
            query: query.to_lowercase(),
            kind_filter: kind_filter.map(|k| format!("{:?}", k)),
            limit,
        };

        if let Ok(cache) = self.query_cache.read() {
            if let Some(cached) = cache.peek(&cache_key) {
                return cached.clone();
            }
        }

        // Cache miss - perform search
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for symbols in self.symbols.values() {
            for sym in symbols {
                // Apply kind filter if specified
                if let Some(filter_kind) = kind_filter {
                    if sym.kind != filter_kind {
                        continue;
                    }
                }

                // Apply fuzzy matching
                if fuzzy_match(&sym.name, &query_lower) {
                    #[allow(deprecated)]
                    results.push(SymbolInformation {
                        name: sym.name.clone(),
                        kind: sym.kind,
                        location: sym.location.clone(),
                        container_name: sym.container_name.clone(),
                        tags: None,
                        deprecated: None,
                    });

                    if results.len() >= limit {
                        break;
                    }
                }
            }

            if results.len() >= limit {
                break;
            }
        }

        // Sort by relevance (exact prefix matches first)
        results.sort_by(|a, b| {
            let a_starts = a.name.to_lowercase().starts_with(&query_lower);
            let b_starts = b.name.to_lowercase().starts_with(&query_lower);
            match (a_starts, b_starts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        // Cache the results
        if let Ok(mut cache) = self.query_cache.write() {
            cache.put(cache_key, results.clone());
        }

        results
    }

    /// Get symbol count for a document
    pub fn symbol_count(&self, uri: &Url) -> usize {
        self.symbols.get(uri).map(|s| s.len()).unwrap_or(0)
    }

    /// Get total symbol count
    pub fn total_symbols(&self) -> usize {
        self.symbols.values().map(|s| s.len()).sum()
    }

    /// Index multiple documents in parallel (for initial workspace indexing)
    ///
    /// Note: Currently performs sequential extraction due to AST not being Sync.
    /// Parallel processing would require Send+Sync bounds on Program, which would
    /// need Arc<Mutex<>> wrappers throughout the AST. For now, this method provides
    /// optimized batch indexing with reduced cache invalidations.
    pub fn index_documents_parallel(&mut self, documents: Vec<(Url, String, Program)>) {
        // Extract all symbols first (currently sequential due to AST constraints)
        let mut symbol_entries = Vec::new();
        for (uri, text, ast) in &documents {
            let symbols = extract_indexed_symbols(uri, text, ast);
            symbol_entries.push((uri.clone(), symbols));
        }

        // Insert in batch to minimize lock contention and cache invalidations
        for (uri, symbols) in symbol_entries {
            // Check memory bounds
            let total_symbols: usize = self.symbols.values().map(|v| v.len()).sum();
            if total_symbols + symbols.len() > self.config.max_symbols {
                // Remove symbols from oldest document to stay within bounds
                if let Some(oldest_uri) = self.symbols.keys().next().cloned() {
                    self.symbols.remove(&oldest_uri);
                }
            }

            self.symbols.insert(uri, symbols);
        }

        // Invalidate cache once after batch update (more efficient than per-document)
        self.invalidate_cache();
    }
}

/// Extract indexed symbols from an AST
fn extract_indexed_symbols(uri: &Url, text: &str, program: &Program) -> Vec<IndexedSymbol> {
    let mut symbols = Vec::new();

    for item in &program.items {
        match item {
            Item::Function(func) => {
                let range = span_to_range(text, func.span);

                symbols.push(IndexedSymbol {
                    name: func.name.name.clone(),
                    kind: SymbolKind::FUNCTION,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    container_name: None,
                });

                // Extract parameters as symbols
                for param in &func.params {
                    let param_range = span_to_range(text, param.span);
                    symbols.push(IndexedSymbol {
                        name: param.name.name.clone(),
                        kind: SymbolKind::VARIABLE,
                        location: Location {
                            uri: uri.clone(),
                            range: param_range,
                        },
                        container_name: Some(func.name.name.clone()),
                    });
                }

                // Extract symbols from function body
                extract_block_symbols(uri, text, &func.body, Some(&func.name.name), &mut symbols);
            }
            Item::Statement(stmt) => {
                extract_statement_symbols(uri, text, stmt, None, &mut symbols);
            }
            Item::TypeAlias(alias) => {
                let range = span_to_range(text, alias.span);

                symbols.push(IndexedSymbol {
                    name: alias.name.name.clone(),
                    kind: SymbolKind::TYPE_PARAMETER,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    container_name: None,
                });
            }
            Item::Import(import) => {
                // Extract imported symbols
                for spec in &import.specifiers {
                    match spec {
                        ImportSpecifier::Named { name, span } => {
                            let range = span_to_range(text, *span);
                            symbols.push(IndexedSymbol {
                                name: name.name.clone(),
                                kind: SymbolKind::MODULE,
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                container_name: Some(import.source.clone()),
                            });
                        }
                        ImportSpecifier::Namespace { alias, span } => {
                            let range = span_to_range(text, *span);
                            symbols.push(IndexedSymbol {
                                name: alias.name.clone(),
                                kind: SymbolKind::NAMESPACE,
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                container_name: Some(import.source.clone()),
                            });
                        }
                    }
                }
            }
            Item::Extern(ext) => {
                let range = span_to_range(text, ext.span);

                symbols.push(IndexedSymbol {
                    name: ext.name.clone(),
                    kind: SymbolKind::FUNCTION,
                    location: Location {
                        uri: uri.clone(),
                        range,
                    },
                    container_name: Some("extern".to_string()),
                });
            }
            Item::Export(export) => match &export.item {
                ExportItem::Function(func) => {
                    let range = span_to_range(text, func.span);
                    symbols.push(IndexedSymbol {
                        name: func.name.name.clone(),
                        kind: SymbolKind::FUNCTION,
                        location: Location {
                            uri: uri.clone(),
                            range,
                        },
                        container_name: None,
                    });
                }
                ExportItem::Variable(var) => {
                    let range = span_to_range(text, var.span);
                    symbols.push(IndexedSymbol {
                        name: var.name.name.clone(),
                        kind: if var.mutable {
                            SymbolKind::VARIABLE
                        } else {
                            SymbolKind::CONSTANT
                        },
                        location: Location {
                            uri: uri.clone(),
                            range,
                        },
                        container_name: None,
                    });
                }
                ExportItem::TypeAlias(alias) => {
                    let range = span_to_range(text, alias.span);
                    symbols.push(IndexedSymbol {
                        name: alias.name.name.clone(),
                        kind: SymbolKind::TYPE_PARAMETER,
                        location: Location {
                            uri: uri.clone(),
                            range,
                        },
                        container_name: None,
                    });
                }
            },
            Item::Trait(_) | Item::Impl(_) => {
                // Trait/impl symbol extraction handled in Block 3
            }
        }
    }

    symbols
}

/// Extract symbols from a block
fn extract_block_symbols(
    uri: &Url,
    text: &str,
    block: &Block,
    container: Option<&str>,
    symbols: &mut Vec<IndexedSymbol>,
) {
    for stmt in &block.statements {
        extract_statement_symbols(uri, text, stmt, container, symbols);
    }
}

/// Extract symbols from a statement
fn extract_statement_symbols(
    uri: &Url,
    text: &str,
    stmt: &Stmt,
    container: Option<&str>,
    symbols: &mut Vec<IndexedSymbol>,
) {
    match stmt {
        Stmt::VarDecl(var) => {
            let range = span_to_range(text, var.span);

            symbols.push(IndexedSymbol {
                name: var.name.name.clone(),
                kind: if var.mutable {
                    SymbolKind::VARIABLE
                } else {
                    SymbolKind::CONSTANT
                },
                location: Location {
                    uri: uri.clone(),
                    range,
                },
                container_name: container.map(String::from),
            });
        }
        Stmt::FunctionDecl(func) => {
            let range = span_to_range(text, func.span);

            symbols.push(IndexedSymbol {
                name: func.name.name.clone(),
                kind: SymbolKind::FUNCTION,
                location: Location {
                    uri: uri.clone(),
                    range,
                },
                container_name: container.map(String::from),
            });

            // Recursively extract from nested function
            extract_block_symbols(uri, text, &func.body, Some(&func.name.name), symbols);
        }
        Stmt::If(if_stmt) => {
            extract_block_symbols(uri, text, &if_stmt.then_block, container, symbols);
            if let Some(else_block) = &if_stmt.else_block {
                extract_block_symbols(uri, text, else_block, container, symbols);
            }
        }
        Stmt::While(while_stmt) => {
            extract_block_symbols(uri, text, &while_stmt.body, container, symbols);
        }
        Stmt::For(for_stmt) => {
            // Extract init statement (may contain variable declaration)
            extract_statement_symbols(uri, text, &for_stmt.init, container, symbols);
            extract_block_symbols(uri, text, &for_stmt.body, container, symbols);
        }
        Stmt::ForIn(for_in) => {
            // Iterator variable
            let range = span_to_range(text, for_in.variable.span);
            symbols.push(IndexedSymbol {
                name: for_in.variable.name.clone(),
                kind: SymbolKind::VARIABLE,
                location: Location {
                    uri: uri.clone(),
                    range,
                },
                container_name: container.map(String::from),
            });

            extract_block_symbols(uri, text, &for_in.body, container, symbols);
        }
        // Other statements don't introduce new symbols
        _ => {}
    }
}

/// Extract document symbols with hierarchy
pub fn extract_document_symbols(text: &str, program: &Program) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for item in &program.items {
        match item {
            Item::Function(func) => {
                let range = span_to_range(text, func.span);
                let selection_range = span_to_range(text, func.name.span);

                let children = extract_function_children(text, func);

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: func.name.name.clone(),
                    detail: Some(format_function_signature(func)),
                    kind: SymbolKind::FUNCTION,
                    range,
                    selection_range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                    tags: None,
                    deprecated: None,
                });
            }
            Item::Statement(Stmt::VarDecl(var)) => {
                let range = span_to_range(text, var.span);
                let selection_range = span_to_range(text, var.name.span);

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: var.name.name.clone(),
                    detail: var.type_ref.as_ref().map(format_type_ref),
                    kind: if var.mutable {
                        SymbolKind::VARIABLE
                    } else {
                        SymbolKind::CONSTANT
                    },
                    range,
                    selection_range,
                    children: None,
                    tags: None,
                    deprecated: None,
                });
            }
            Item::TypeAlias(alias) => {
                let range = span_to_range(text, alias.span);
                let selection_range = span_to_range(text, alias.name.span);

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: alias.name.name.clone(),
                    detail: Some(format_type_ref(&alias.type_ref)),
                    kind: SymbolKind::TYPE_PARAMETER,
                    range,
                    selection_range,
                    children: None,
                    tags: None,
                    deprecated: None,
                });
            }
            Item::Import(import) => {
                let range = span_to_range(text, import.span);

                // Create import group symbol with children for each import
                let children: Vec<DocumentSymbol> = import
                    .specifiers
                    .iter()
                    .map(|spec| match spec {
                        ImportSpecifier::Named { name, span } => {
                            let imp_range = span_to_range(text, *span);
                            #[allow(deprecated)]
                            DocumentSymbol {
                                name: name.name.clone(),
                                detail: None,
                                kind: SymbolKind::MODULE,
                                range: imp_range,
                                selection_range: imp_range,
                                children: None,
                                tags: None,
                                deprecated: None,
                            }
                        }
                        ImportSpecifier::Namespace { alias, span } => {
                            let imp_range = span_to_range(text, *span);
                            #[allow(deprecated)]
                            DocumentSymbol {
                                name: alias.name.clone(),
                                detail: Some("namespace".to_string()),
                                kind: SymbolKind::NAMESPACE,
                                range: imp_range,
                                selection_range: imp_range,
                                children: None,
                                tags: None,
                                deprecated: None,
                            }
                        }
                    })
                    .collect();

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: format!("import \"{}\"", import.source),
                    detail: None,
                    kind: SymbolKind::MODULE,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                    tags: None,
                    deprecated: None,
                });
            }
            Item::Extern(ext) => {
                let range = span_to_range(text, ext.span);

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: ext.name.clone(),
                    detail: Some(format!("extern from \"{}\"", ext.library)),
                    kind: SymbolKind::FUNCTION,
                    range,
                    selection_range: range,
                    children: None,
                    tags: None,
                    deprecated: None,
                });
            }
            Item::Export(export) => match &export.item {
                ExportItem::Function(func) => {
                    let range = span_to_range(text, func.span);
                    let selection_range = span_to_range(text, func.name.span);
                    let children = extract_function_children(text, func);

                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: func.name.name.clone(),
                        detail: Some(format!("export {}", format_function_signature(func))),
                        kind: SymbolKind::FUNCTION,
                        range,
                        selection_range,
                        children: if children.is_empty() {
                            None
                        } else {
                            Some(children)
                        },
                        tags: None,
                        deprecated: None,
                    });
                }
                ExportItem::Variable(var) => {
                    let range = span_to_range(text, var.span);
                    let selection_range = span_to_range(text, var.name.span);

                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: var.name.name.clone(),
                        detail: Some("export".to_string()),
                        kind: if var.mutable {
                            SymbolKind::VARIABLE
                        } else {
                            SymbolKind::CONSTANT
                        },
                        range,
                        selection_range,
                        children: None,
                        tags: None,
                        deprecated: None,
                    });
                }
                ExportItem::TypeAlias(alias) => {
                    let range = span_to_range(text, alias.span);
                    let selection_range = span_to_range(text, alias.name.span);

                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: alias.name.name.clone(),
                        detail: Some(format!(
                            "export type = {}",
                            format_type_ref(&alias.type_ref)
                        )),
                        kind: SymbolKind::TYPE_PARAMETER,
                        range,
                        selection_range,
                        children: None,
                        tags: None,
                        deprecated: None,
                    });
                }
            },
            _ => {}
        }
    }

    symbols
}

/// Extract children symbols from a function
fn extract_function_children(text: &str, func: &FunctionDecl) -> Vec<DocumentSymbol> {
    let mut children = Vec::new();

    // Parameters as children
    for param in &func.params {
        let range = span_to_range(text, param.span);

        #[allow(deprecated)]
        children.push(DocumentSymbol {
            name: param.name.name.clone(),
            detail: Some(format_type_ref(&param.type_ref)),
            kind: SymbolKind::VARIABLE,
            range,
            selection_range: range,
            children: None,
            tags: None,
            deprecated: None,
        });
    }

    // Variables and nested functions from body
    extract_block_children(text, &func.body, &mut children);

    children
}

/// Extract children symbols from a block
fn extract_block_children(text: &str, block: &Block, children: &mut Vec<DocumentSymbol>) {
    for stmt in &block.statements {
        match stmt {
            Stmt::VarDecl(var) => {
                let range = span_to_range(text, var.span);
                let selection_range = span_to_range(text, var.name.span);

                #[allow(deprecated)]
                children.push(DocumentSymbol {
                    name: var.name.name.clone(),
                    detail: var.type_ref.as_ref().map(format_type_ref),
                    kind: if var.mutable {
                        SymbolKind::VARIABLE
                    } else {
                        SymbolKind::CONSTANT
                    },
                    range,
                    selection_range,
                    children: None,
                    tags: None,
                    deprecated: None,
                });
            }
            Stmt::FunctionDecl(nested) => {
                let range = span_to_range(text, nested.span);
                let selection_range = span_to_range(text, nested.name.span);
                let nested_children = extract_function_children(text, nested);

                #[allow(deprecated)]
                children.push(DocumentSymbol {
                    name: nested.name.name.clone(),
                    detail: Some(format_function_signature(nested)),
                    kind: SymbolKind::FUNCTION,
                    range,
                    selection_range,
                    children: if nested_children.is_empty() {
                        None
                    } else {
                        Some(nested_children)
                    },
                    tags: None,
                    deprecated: None,
                });
            }
            Stmt::If(if_stmt) => {
                extract_block_children(text, &if_stmt.then_block, children);
                if let Some(else_block) = &if_stmt.else_block {
                    extract_block_children(text, else_block, children);
                }
            }
            Stmt::While(while_stmt) => {
                extract_block_children(text, &while_stmt.body, children);
            }
            Stmt::For(for_stmt) => {
                extract_block_children(text, &for_stmt.body, children);
            }
            Stmt::ForIn(for_in) => {
                extract_block_children(text, &for_in.body, children);
            }
            _ => {}
        }
    }
}

/// Convert a byte-offset span to LSP Range
pub fn span_to_range(text: &str, span: Span) -> Range {
    let start = offset_to_position(text, span.start);
    let end = offset_to_position(text, span.end);
    Range { start, end }
}

/// Convert a byte offset to LSP Position
pub fn offset_to_position(text: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0usize;

    for ch in text.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += ch.len_utf8();
    }

    Position {
        line,
        character: col,
    }
}

/// Convert LSP Position to byte offset
pub fn position_to_offset(text: &str, position: Position) -> usize {
    let mut offset = 0;
    let mut line = 0u32;

    for ch in text.chars() {
        if line == position.line {
            break;
        }
        if ch == '\n' {
            line += 1;
        }
        offset += ch.len_utf8();
    }

    // Add character offset within the line
    for (col, ch) in text[offset..].chars().enumerate() {
        if col as u32 >= position.character || ch == '\n' {
            break;
        }
        offset += ch.len_utf8();
    }

    offset
}

/// Fuzzy match for workspace symbol search
fn fuzzy_match(name: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let name_lower = name.to_lowercase();

    // Exact prefix match
    if name_lower.starts_with(query) {
        return true;
    }

    // Substring match
    if name_lower.contains(query) {
        return true;
    }

    // Camel case / snake_case matching
    let query_chars: Vec<char> = query.chars().collect();
    let mut query_idx = 0;

    for ch in name_lower.chars() {
        if query_idx < query_chars.len() && ch == query_chars[query_idx] {
            query_idx += 1;
        }
    }

    query_idx == query_chars.len()
}

/// Format a function signature
fn format_function_signature(func: &FunctionDecl) -> String {
    let params: Vec<String> = func
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.name, format_type_ref(&p.type_ref)))
        .collect();

    format!(
        "fn {}({}) -> {}",
        func.name.name,
        params.join(", "),
        format_type_ref(&func.return_type)
    )
}

/// Format a type reference for display
fn format_type_ref(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name, _) => name.clone(),
        TypeRef::Array(inner, _) => format!("{}[]", format_type_ref(inner)),
        TypeRef::Union { members, .. } => {
            let formatted: Vec<String> = members.iter().map(format_type_ref).collect();
            formatted.join(" | ")
        }
        TypeRef::Function {
            params,
            return_type,
            ..
        } => {
            let param_strs: Vec<String> = params.iter().map(format_type_ref).collect();
            format!(
                "({}) -> {}",
                param_strs.join(", "),
                format_type_ref(return_type)
            )
        }
        TypeRef::Structural { members, .. } => {
            let field_strs: Vec<String> = members
                .iter()
                .map(|m| format!("{}: {}", m.name, format_type_ref(&m.type_ref)))
                .collect();
            format!("{{ {} }}", field_strs.join(", "))
        }
        TypeRef::Generic {
            name, type_args, ..
        } => {
            let arg_strs: Vec<String> = type_args.iter().map(format_type_ref).collect();
            format!("{}<{}>", name, arg_strs.join(", "))
        }
        TypeRef::Intersection { members, .. } => {
            let formatted: Vec<String> = members.iter().map(format_type_ref).collect();
            formatted.join(" & ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_to_position_simple() {
        let text = "hello";
        let pos = offset_to_position(text, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);

        let pos = offset_to_position(text, 3);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 3);
    }

    #[test]
    fn test_offset_to_position_multiline() {
        let text = "line1\nline2\nline3";
        let pos = offset_to_position(text, 7);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 1);

        let pos = offset_to_position(text, 12);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_fuzzy_match_exact() {
        assert!(fuzzy_match("hello", "hello"));
        assert!(fuzzy_match("Hello", "hello"));
    }

    #[test]
    fn test_fuzzy_match_prefix() {
        assert!(fuzzy_match("fooBar", "foo"));
        assert!(fuzzy_match("FooBar", "foo"));
    }

    #[test]
    fn test_fuzzy_match_substring() {
        assert!(fuzzy_match("fooBarBaz", "bar"));
    }

    #[test]
    fn test_fuzzy_match_camel_case() {
        assert!(fuzzy_match("fooBarBaz", "fbb"));
        assert!(fuzzy_match("myVariableName", "mvn"));
    }

    #[test]
    fn test_fuzzy_match_empty_query() {
        assert!(fuzzy_match("anything", ""));
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        assert!(!fuzzy_match("hello", "xyz"));
    }
}
