# LSP Navigation Features

**Status:** Complete (Phase 05A/B/C)
**Version:** 0.2.0

This document describes the comprehensive code navigation features provided by the Atlas Language Server Protocol (LSP) implementation.

---

## Table of Contents

1. [Overview](#overview)
2. [Symbol Indexing](#symbol-indexing)
3. [Find All References](#find-all-references)
4. [Call Hierarchy](#call-hierarchy)
5. [Workspace Symbol Search](#workspace-symbol-search)
6. [Performance Optimizations](#performance-optimizations)
7. [Configuration](#configuration)
8. [API Reference](#api-reference)
9. [Examples](#examples)

---

## Overview

Atlas LSP provides four main navigation features that work together to enable efficient code exploration:

- **Symbol Indexing**: Maintains a workspace-wide index of all symbol definitions and references
- **Find All References**: Locates all uses of a symbol across the workspace
- **Call Hierarchy**: Visualizes function call relationships (incoming and outgoing calls)
- **Workspace Symbol Search**: Fuzzy search for symbols across all files

All features are designed for:
- **Speed**: Optimized for large workspaces (100k+ symbols)
- **Accuracy**: AST-based analysis ensures precise results
- **Scalability**: Memory-bounded with intelligent caching

---

## Symbol Indexing

### Architecture

The symbol index maintains three main data structures:

```rust
pub struct SymbolIndex {
    definitions: HashMap<String, Vec<SymbolDefinition>>,
    references: HashMap<String, Vec<SymbolReference>>,
    file_definitions: HashMap<Url, Vec<String>>,
    file_references: HashMap<Url, Vec<String>>,
}
```

### Symbol Types

The index tracks:
- **Functions**: Top-level and nested function declarations
- **Variables**: Local variables, parameters, and constants
- **Parameters**: Function parameters
- **Types**: Type aliases and type definitions

### Incremental Updates

The index updates incrementally when files change:
1. **File Open**: Parse AST, extract symbols, index definitions and references
2. **File Change**: Re-parse, remove old symbols, index new symbols
3. **File Close**: Remove all symbols from the file

### Scope Tracking

Symbols include scope information:
- **Global scope**: Top-level declarations (container_name = None)
- **Function scope**: Local variables and nested functions (container_name = function name)

---

## Find All References

### Capabilities

- Find all references to variables, functions, and parameters
- Include/exclude the definition in results
- Cross-file reference tracking
- Write vs. read reference distinction

### Usage

```typescript
// Client request
textDocument/references
{
  "textDocument": { "uri": "file:///path/to/file.atl" },
  "position": { "line": 5, "character": 8 },
  "context": { "includeDeclaration": true }
}

// Server response
[
  {
    "uri": "file:///path/to/file.atl",
    "range": {
      "start": { "line": 2, "character": 8 },
      "end": { "line": 2, "character": 9 }
    }
  },
  // ... more references
]
```

### Implementation Details

**Reference Extraction:**
- Traverses the AST to find all identifier usages
- Tracks assignment vs. read contexts
- Handles shadowing (local scope takes precedence)

**Performance:**
- O(1) lookup by symbol name
- Results filtered by position for precise matching
- Cached at the AST level (invalidated on file change)

---

## Call Hierarchy

### Capabilities

**Incoming Calls**: Find all functions that call a target function
```
helper() is called by:
  ├─ main()
  ├─ processData()
  └─ validate()
```

**Outgoing Calls**: Find all functions called by a target function
```
main() calls:
  ├─ helper()
  ├─ len() [stdlib]
  └─ print() [stdlib]
```

### Features

- **Recursive Call Detection**: Identifies self-recursion and mutual recursion
- **Depth Limiting**: Prevents infinite loops in call trees
- **Stdlib Recognition**: Distinguishes between user functions and stdlib calls
- **Method-Style Calls**: Handles both `foo(x)` and `x.foo()` syntax
- **Cross-File Support**: Tracks calls across file boundaries

### Usage

```typescript
// 1. Prepare call hierarchy
textDocument/prepareCallHierarchy
{
  "textDocument": { "uri": "file:///test.atl" },
  "position": { "line": 5, "character": 3 }
}

// Response: CallHierarchyItem
{
  "name": "targetFunction",
  "kind": SymbolKind.Function,
  "uri": "file:///test.atl",
  "range": { ... },
  "selectionRange": { ... }
}

// 2. Get incoming calls
callHierarchy/incomingCalls
{
  "item": { ... }  // CallHierarchyItem from step 1
}

// Response: CallHierarchyIncomingCall[]
[
  {
    "from": {
      "name": "caller",
      "kind": SymbolKind.Function,
      // ...
    },
    "fromRanges": [ { ... } ]  // Call sites
  }
]

// 3. Get outgoing calls
callHierarchy/outgoingCalls
{
  "item": { ... }
}

// Response: CallHierarchyOutgoingCall[]
[
  {
    "to": {
      "name": "callee",
      "kind": SymbolKind.Function,
      // ...
    },
    "fromRanges": [ { ... } ]  // Call sites
  }
]
```

### Recursion Handling

The call hierarchy protects against infinite recursion:

```atlas
// Direct recursion
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);  // Detected
}

// Mutual recursion
fn isEven(n: number) -> boolean {
    if (n == 0) { return true; }
    return isOdd(n - 1);  // Calls isOdd
}

fn isOdd(n: number) -> boolean {
    if (n == 0) { return false; }
    return isEven(n - 1);  // Calls isEven
}
```

**Strategy:**
- Track visited functions in a HashSet during traversal
- Stop recursion when a cycle is detected
- Include recursive calls in results with a marker

---

## Workspace Symbol Search

### Capabilities

- **Fuzzy Matching**: Search with abbreviated queries
  - `"mvn"` matches `myVariableName`
  - `"proc"` matches `processData`
- **Exact Matching**: Prefix and substring matching
- **Kind Filtering**: Filter by symbol type (function, variable, type)
- **Relevance Ranking**: Exact prefix matches ranked first
- **Cross-File Search**: Search across entire workspace

### Matching Strategies

1. **Exact Prefix**: `query` at start of symbol name (highest priority)
   - `"test"` → `testFunction` ✓

2. **Substring**: `query` anywhere in symbol name (medium priority)
   - `"test"` → `myTestHelper` ✓

3. **CamelCase Fuzzy**: Characters in order (lowest priority)
   - `"fbb"` → `fooBarBaz` ✓
   - `"mvn"` → `myVariableName` ✓

### Usage

```typescript
workspace/symbol
{
  "query": "process"
}

// Response: SymbolInformation[]
[
  {
    "name": "processData",
    "kind": SymbolKind.Function,
    "location": {
      "uri": "file:///utils.atl",
      "range": { ... }
    },
    "containerName": null
  },
  {
    "name": "processRequest",
    "kind": SymbolKind.Function,
    "location": {
      "uri": "file:///handler.atl",
      "range": { ... }
    }
  }
]
```

### Search Examples

| Query | Matches | Strategy |
|-------|---------|----------|
| `"test"` | `testFunction`, `myTestHelper`, `anotherTest` | Prefix + Substring |
| `"TF"` | `testFunction`, `TestFixture` | CamelCase |
| `"get"` | `getData`, `getUserInfo`, `target` | Prefix + Substring |
| `""` | All symbols (up to limit) | Match all |

---

## Performance Optimizations

### Query Caching

**Problem**: Repeated searches are expensive for large workspaces.

**Solution**: LRU cache stores query results.

```rust
pub struct WorkspaceIndex {
    query_cache: Arc<RwLock<LruCache<QueryCacheKey, Vec<SymbolInformation>>>>,
    // ...
}

// Cache key includes:
// - Query string (lowercased)
// - Symbol kind filter
// - Result limit
```

**Benefits:**
- 10-100x speedup for cached queries
- Automatic eviction (LRU policy)
- Thread-safe (RwLock)
- Invalidated on index changes

### Memory Bounds

**Problem**: Large workspaces can consume unbounded memory.

**Solution**: Configurable symbol limit with automatic eviction.

```rust
pub struct IndexConfig {
    max_symbols: usize,      // Default: 100,000
    cache_size: usize,       // Default: 100 queries
    parallel_indexing: bool, // Default: true
}
```

**Behavior:**
- When limit exceeded, oldest document's symbols are removed
- Cache size bounded independently
- Prevents OOM on massive codebases

### Batch Indexing

**Problem**: Indexing files one-by-one invalidates cache frequently.

**Solution**: Batch API that indexes multiple files with a single cache invalidation.

```rust
index.index_documents_parallel(vec![
    (uri1, text1, ast1),
    (uri2, text2, ast2),
    // ...
]);
// Cache invalidated once after all files indexed
```

**Benefits:**
- Reduces cache invalidation overhead
- Better for initial workspace indexing
- Optimized for bulk operations

### Incremental Updates

**Strategy:**
- Only re-index changed files
- Remove old symbols, insert new symbols
- Invalidate cache (future: partial invalidation)

**Performance:**
- O(n) where n = symbols in changed file
- Independent of workspace size
- Fast document updates (< 10ms typical)

---

## Configuration

### Index Configuration

```rust
use atlas_lsp::symbols::{IndexConfig, WorkspaceIndex};

let config = IndexConfig {
    max_symbols: 200_000,      // Allow 200k symbols
    cache_size: 200,            // Cache 200 query results
    parallel_indexing: true,    // Enable batch optimization
};

let index = WorkspaceIndex::with_config(config);
```

### Recommended Settings

| Workspace Size | max_symbols | cache_size |
|----------------|-------------|------------|
| Small (< 10 files) | 10,000 | 50 |
| Medium (10-100 files) | 50,000 | 100 |
| Large (100-1000 files) | 100,000 | 200 |
| Huge (> 1000 files) | 250,000 | 500 |

---

## API Reference

### SymbolIndex

```rust
impl SymbolIndex {
    pub fn new() -> Self;
    pub fn index_document(&mut self, uri: &Url, text: &str, ast: Option<&Program>);
    pub fn remove_document(&mut self, uri: &Url);
    pub fn find_definitions(&self, name: &str, uri: &Url, position: Position) -> Vec<Location>;
    pub fn find_references(&self, name: &str, uri: &Url, position: Position, include_declaration: bool) -> Vec<Location>;
}
```

### WorkspaceIndex

```rust
impl WorkspaceIndex {
    pub fn new() -> Self;
    pub fn with_config(config: IndexConfig) -> Self;
    pub fn index_document(&mut self, uri: Url, text: &str, ast: &Program);
    pub fn index_documents_parallel(&mut self, documents: Vec<(Url, String, Program)>);
    pub fn remove_document(&mut self, uri: &Url);
    pub fn search(&self, query: &str, limit: usize, kind_filter: Option<SymbolKind>) -> Vec<SymbolInformation>;
    pub fn symbol_count(&self, uri: &Url) -> usize;
    pub fn total_symbols(&self) -> usize;
}
```

### CallHierarchy

```rust
pub fn prepare_call_hierarchy(uri: &Url, text: &str, ast: &Program, position: Position) -> Option<CallHierarchyItem>;
pub fn find_incoming_calls(uri: &Url, text: &str, ast: &Program, target: &str) -> Vec<CallHierarchyIncomingCall>;
pub fn find_outgoing_calls(uri: &Url, text: &str, ast: &Program, target: &str) -> Vec<CallHierarchyOutgoingCall>;
```

---

## Examples

### Example 1: Find All References

```atlas
// File: main.atl
fn helper() -> number {
    return 42;
}

fn main() -> number {
    let x = helper();  // Reference 1
    let y = helper();  // Reference 2
    return x + y;
}
```

**Request:** Find references to `helper` (position at line 1, col 3)

**Result:**
```json
[
  {"uri": "file:///main.atl", "range": {"start": {"line": 5, "character": 12}, ...}},
  {"uri": "file:///main.atl", "range": {"start": {"line": 6, "character": 12}, ...}}
]
```

### Example 2: Call Hierarchy

```atlas
// File: app.atl
fn validateInput(input: string) -> boolean {
    return len(input) > 0;
}

fn processData(data: string) -> string {
    if (!validateInput(data)) {
        return "";
    }
    return data;
}

fn main() -> string {
    return processData("hello");
}
```

**Request:** Incoming calls for `validateInput`

**Result:**
```json
[
  {
    "from": {
      "name": "processData",
      "kind": 12,
      "uri": "file:///app.atl",
      ...
    },
    "fromRanges": [
      {"start": {"line": 6, "character": 9}, ...}
    ]
  }
]
```

### Example 3: Workspace Symbol Search

```atlas
// File: utils.atl
fn calculateSum(a: number, b: number) -> number {
    return a + b;
}

fn calculateProduct(a: number, b: number) -> number {
    return a * b;
}

// File: helpers.atl
fn calculateAverage(values: number[]) -> number {
    return sum(values) / len(values);
}
```

**Query:** `"calc"`

**Results (ranked by relevance):**
1. `calculateSum` (exact prefix match)
2. `calculateProduct` (exact prefix match)
3. `calculateAverage` (exact prefix match)

**Query:** `"cs"` (CamelCase fuzzy)

**Results:**
1. `calculateSum` (matches "c" and "s")

---

## Integration with LSP Clients

### VS Code

The Atlas LSP server automatically provides:
- **Go to Definition**: Click on symbol → jump to definition
- **Find All References**: Right-click → "Find All References"
- **Call Hierarchy**: Right-click → "Peek Call Hierarchy" / "Show Call Hierarchy"
- **Workspace Symbol**: `Cmd+T` (Mac) or `Ctrl+T` (Windows/Linux) → type query

### Neovim (via nvim-lspconfig)

```lua
require('lspconfig').atlas_lsp.setup{
  on_attach = function(client, bufnr)
    -- Find references
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, {buffer=bufnr})

    -- Call hierarchy
    vim.keymap.set('n', '<leader>ci', vim.lsp.buf.incoming_calls, {buffer=bufnr})
    vim.keymap.set('n', '<leader>co', vim.lsp.buf.outgoing_calls, {buffer=bufnr})

    -- Workspace symbols
    vim.keymap.set('n', '<leader>fs', vim.lsp.buf.workspace_symbol, {buffer=bufnr})
  end
}
```

### Emacs (via lsp-mode)

```elisp
(use-package lsp-mode
  :hook (atlas-mode . lsp)
  :commands lsp
  :config
  ;; All navigation features work automatically:
  ;; - M-? : Find references
  ;; - M-. : Go to definition
  ;; - C-c l g i : Incoming calls
  ;; - C-c l g o : Outgoing calls
  ;; - C-c l g w : Workspace symbols
  )
```

---

## Troubleshooting

### Symbols Not Found

**Symptom:** Workspace symbol search returns empty results.

**Causes:**
1. Files not opened/indexed yet
2. Parse errors preventing indexing
3. Memory limit exceeded (old files evicted)

**Solutions:**
- Open files to trigger indexing
- Check server logs for parse errors
- Increase `max_symbols` config

### Slow Search Performance

**Symptom:** Workspace symbol search takes > 1 second.

**Causes:**
1. Very large workspace (> 1M symbols)
2. Cache not warming up
3. Complex fuzzy queries

**Solutions:**
- Reduce workspace size or increase `max_symbols` limit
- Use more specific queries (prefix matching is faster)
- Increase `cache_size` for better hit rate

### Incorrect References

**Symptom:** Find references returns unrelated symbols with same name.

**Causes:**
1. Shadowing not handled correctly (known limitation)
2. Cross-file scope resolution

**Solutions:**
- This is a known limitation (DR-LSP-006)
- Future enhancement: full scope analysis with symbol table

---

## Future Enhancements

Potential improvements for future phases:

1. **Type-Aware Navigation**
   - Find references by type, not just name
   - Jump to type definition
   - Find implementations

2. **Advanced Filtering**
   - Filter by file path pattern
   - Filter by symbol visibility (public/private)
   - Filter by module

3. **Performance**
   - Parallel symbol extraction (requires AST refactor for Send+Sync)
   - Partial cache invalidation (invalidate only affected queries)
   - Bloom filters for negative lookups

4. **Scope Analysis**
   - Proper shadowing handling
   - Control flow analysis for insertion points
   - Dataflow analysis for dead code detection

---

## Related Documentation

- [LSP Status](lsp-status.md) - Overall LSP implementation status
- [Atlas Specification](specification/) - Language specification
- [Testing Guide](../memory/testing-patterns.md) - LSP testing patterns

---

**Last Updated:** 2026-02-20
**Phase:** 05C (Workspace Symbols & Polish)
**Maintainer:** Atlas LSP Team
