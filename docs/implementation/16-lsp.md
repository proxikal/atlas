# LSP Implementation Guide

## Overview

The Atlas Language Server Protocol (LSP) implementation provides AI agents and human developers with real-time code intelligence. This guide covers the architecture and implementation of the `atlas-lsp` crate.

## Why LSP for Atlas?

**Atlas is AI-native.** While other languages were built before AI and retrofitted with tooling, Atlas treats AI agents as first-class consumers from day one. The LSP gives AI agents:

- **Structured access** to diagnostics, types, and symbols
- **Real-time feedback** without repeated compiler invocations
- **Navigation capabilities** (go-to-definition, find references)
- **Context-aware completion** for better code generation

Humans benefit too: the same LSP powers editor integration in VSCode, Neovim, Zed, and other editors.

## Architecture

### Crate Structure

```
atlas/
├── crates/
│   ├── atlas-runtime/      # Core language (existing)
│   ├── atlas-cli/           # CLI tool (existing)
│   └── atlas-lsp/           # LSP server (new)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs         # LSP server binary entry point
│           ├── server.rs       # LSP server implementation
│           ├── handlers/
│           │   ├── mod.rs
│           │   ├── diagnostics.rs   # publishDiagnostics
│           │   ├── symbols.rs       # documentSymbol, definition, references
│           │   ├── completion.rs    # completion, completionResolve
│           │   ├── hover.rs         # hover information
│           │   └── formatting.rs    # formatting, rangeFormatting
│           ├── document.rs     # Document state management
│           └── convert.rs      # Atlas <-> LSP type conversions
```

### Dependencies

```toml
[dependencies]
atlas-runtime = { path = "../atlas-runtime" }
tower-lsp = "0.20"      # LSP framework
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Core Components

### 1. LSP Server Lifecycle

The server follows the standard LSP lifecycle:

```rust
use tower_lsp::{LspService, Server, LanguageServer};

pub struct AtlasLspServer {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

impl AtlasLspServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for AtlasLspServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions::default()
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "atlas-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Atlas LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
```

### 2. Document State Management

Track open documents and their analysis results:

```rust
pub struct DocumentState {
    pub uri: Url,
    pub text: String,
    pub version: i32,
    pub ast: Option<ast::Program>,
    pub symbols: Option<symbol::SymbolTable>,
    pub types: Option<HashMap<NodeId, types::Type>>,
    pub diagnostics: Vec<diagnostic::Diagnostic>,
}

impl DocumentState {
    pub fn new(uri: Url, text: String, version: i32) -> Self {
        let mut state = Self {
            uri,
            text,
            version,
            ast: None,
            symbols: None,
            types: None,
            diagnostics: Vec::new(),
        };
        state.analyze();
        state
    }

    pub fn analyze(&mut self) {
        self.diagnostics.clear();

        // Lex and parse
        let lexer = lexer::Lexer::new(&self.text);
        let tokens: Vec<_> = lexer.collect();
        let mut parser = parser::Parser::new(tokens);

        match parser.parse() {
            Ok(ast) => {
                self.ast = Some(ast.clone());

                // Bind symbols
                let mut binder = symbol::Binder::new();
                match binder.bind(&ast) {
                    Ok(symbols) => {
                        self.symbols = Some(symbols.clone());

                        // Typecheck
                        let mut typechecker = typechecker::TypeChecker::new(symbols);
                        match typechecker.check(&ast) {
                            Ok(types) => {
                                self.types = Some(types);
                                self.diagnostics.extend(typechecker.diagnostics());
                            }
                            Err(diag) => {
                                self.diagnostics.push(diag);
                            }
                        }
                    }
                    Err(diag) => {
                        self.diagnostics.push(diag);
                    }
                }
            }
            Err(diag) => {
                self.diagnostics.push(diag);
            }
        }

        self.diagnostics.extend(parser.diagnostics());
    }
}
```

### 3. Type Conversions

Convert between Atlas and LSP types:

```rust
// Atlas Span -> LSP Range
pub fn span_to_range(span: &span::Span) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            line: span.start.line as u32,
            character: span.start.column as u32,
        },
        end: lsp_types::Position {
            line: span.end.line as u32,
            character: span.end.column as u32,
        },
    }
}

// Atlas Diagnostic -> LSP Diagnostic
pub fn diagnostic_to_lsp(diag: &diagnostic::Diagnostic) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range: span_to_range(&diag.span),
        severity: Some(match diag.level {
            diagnostic::DiagnosticLevel::Error => DiagnosticSeverity::ERROR,
            diagnostic::DiagnosticLevel::Warning => DiagnosticSeverity::WARNING,
        }),
        source: Some("atlas".to_string()),
        message: diag.message.clone(),
        related_information: diag.related.as_ref().map(|related| {
            related.iter().map(|r| DiagnosticRelatedInformation {
                location: Location {
                    uri: /* derive from span */,
                    range: span_to_range(&r.span),
                },
                message: r.message.clone(),
            }).collect()
        }),
        ..Default::default()
    }
}
```

## LSP Feature Implementation

### Diagnostics (Phase 02)

Hook into document changes to provide real-time feedback:

```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    let version = params.text_document.version;

    let mut documents = self.documents.lock().await;
    if let Some(doc) = documents.get_mut(&uri) {
        // Update document text
        for change in params.content_changes {
            doc.text = change.text;
        }
        doc.version = version;

        // Re-analyze
        doc.analyze();

        // Publish diagnostics
        let diagnostics = doc.diagnostics.iter()
            .map(diagnostic_to_lsp)
            .collect();

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;
    }
}
```

### Navigation (Phase 03)

Implement go-to-definition using symbol bindings:

```rust
async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let documents = self.documents.lock().await;
    let doc = documents.get(&uri)?;

    // Find symbol at position
    let symbol_name = find_symbol_at_position(&doc.ast?, position)?;

    // Lookup definition in symbol table
    if let Some(def_span) = doc.symbols?.get_definition(symbol_name) {
        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: span_to_range(def_span),
        })));
    }

    Ok(None)
}
```

### Completion (Phase 04)

Provide context-aware completions:

```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    let documents = self.documents.lock().await;
    let doc = documents.get(&uri)?;

    let mut items = Vec::new();

    // Keywords
    for keyword in &["let", "if", "else", "while", "func", "return"] {
        items.push(CompletionItem {
            label: keyword.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    // Symbols in scope
    if let Some(symbols) = &doc.symbols {
        for (name, symbol) in symbols.visible_at_position(position) {
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(match symbol.kind {
                    SymbolKind::Function => CompletionItemKind::FUNCTION,
                    SymbolKind::Variable => CompletionItemKind::VARIABLE,
                }),
                detail: Some(format!("{:?}", symbol.ty)),
                ..Default::default()
            });
        }
    }

    Ok(Some(CompletionResponse::Array(items)))
}
```

## Editor Integration

### VSCode Extension

Create a minimal extension in `editors/vscode/`:

```typescript
// extension.ts
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    const serverOptions = {
        command: 'atlas-lsp',  // Assumes in PATH
        args: []
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'atlas' }],
    };

    const client = new LanguageClient(
        'atlasLsp',
        'Atlas Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}
```

### Neovim Configuration

```lua
-- In ~/.config/nvim/after/ftplugin/atlas.lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.atlas_lsp then
    configs.atlas_lsp = {
        default_config = {
            cmd = { 'atlas-lsp' },
            filetypes = { 'atlas' },
            root_dir = lspconfig.util.root_pattern('.git'),
        },
    }
end

lspconfig.atlas_lsp.setup{}
```

## Performance Considerations

**Target Latencies:**
- Diagnostics: <200ms after change
- Completion: <100ms
- Hover: <100ms
- Go-to-definition: <50ms

**Optimization Strategies:**
1. **Incremental parsing** (future): Only re-parse changed regions
2. **Caching**: Cache symbol tables and type information
3. **Debouncing**: Wait 300ms after typing before re-analyzing
4. **Background threads**: Run analysis off the main LSP thread

## Testing Strategy

### Protocol Tests
Test LSP protocol conformance with mock clients:
```rust
#[tokio::test]
async fn test_initialization() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Send initialize request
    // Verify capabilities in response
}
```

### Integration Tests
Test with real editors using automated scripts.

### Performance Tests
Benchmark with large files (>1000 lines) and rapid changes.

## Future Enhancements

- **Semantic highlighting**: Color based on semantic meaning
- **Rename refactoring**: Rename symbol across files
- **Call hierarchy**: Show caller/callee relationships
- **Inlay hints**: Show inferred types inline
- **Code lens**: Show references count, run/debug actions

## References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - Rust LSP framework
- [vscode-languageclient](https://www.npmjs.com/package/vscode-languageclient) - VSCode LSP client
