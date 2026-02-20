//! Atlas LSP Server implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::DocumentState;
use crate::index::SymbolIndex;
use crate::inlay_hints::InlayHintConfig;
use crate::semantic_tokens;
use crate::symbols::WorkspaceIndex;

/// Atlas Language Server
pub struct AtlasLspServer {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
    workspace_index: Arc<Mutex<WorkspaceIndex>>,
    symbol_index: Arc<Mutex<SymbolIndex>>,
    inlay_config: InlayHintConfig,
}

impl AtlasLspServer {
    /// Create a new Atlas LSP server
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
            workspace_index: Arc::new(Mutex::new(WorkspaceIndex::new())),
            symbol_index: Arc::new(Mutex::new(SymbolIndex::new())),
            inlay_config: InlayHintConfig::default(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for AtlasLspServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("atlas".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    completion_item: None,
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::REFACTOR,
                            CodeActionKind::REFACTOR_EXTRACT,
                            CodeActionKind::REFACTOR_INLINE,
                            CodeActionKind::REFACTOR_REWRITE,
                            CodeActionKind::SOURCE,
                            CodeActionKind::SOURCE_ORGANIZE_IMPORTS,
                        ]),
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        resolve_provider: Some(false),
                    },
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_tokens::get_legend(),
                            range: Some(true),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                        },
                    ),
                ),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(
                    InlayHintOptions {
                        resolve_provider: Some(false),
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                ))),
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Create and analyze document
        let doc = DocumentState::new(uri.clone(), text, version);

        // Collect diagnostics before storing
        let diagnostics: Vec<_> = doc
            .diagnostics
            .iter()
            .map(crate::convert::diagnostic_to_lsp)
            .collect();

        // Store document and get AST for indexing
        let ast_clone = doc.ast.clone();
        let text_clone = doc.text.clone();
        {
            let mut documents = self.documents.lock().await;
            documents.insert(uri.clone(), doc);
        }

        // Update workspace index with cloned AST
        if let Some(ast) = ast_clone {
            let mut index = self.workspace_index.lock().await;
            index.index_document(uri.clone(), &text_clone, &ast);

            // Also update symbol index for references
            let mut symbol_index = self.symbol_index.lock().await;
            symbol_index.index_document(&uri, &text_clone, Some(&ast));
        }

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Collect diagnostics and clone AST for indexing
        let (diagnostics, ast_clone, text_clone) = {
            let mut documents = self.documents.lock().await;
            if let Some(doc) = documents.get_mut(&uri) {
                // Update document text (full sync)
                for change in params.content_changes {
                    doc.update(change.text, version);
                }

                // Collect diagnostics and clone AST
                let diags: Vec<_> = doc
                    .diagnostics
                    .iter()
                    .map(crate::convert::diagnostic_to_lsp)
                    .collect();

                (Some(diags), doc.ast.clone(), doc.text.clone())
            } else {
                (None, None, String::new())
            }
        };

        // Update workspace index with cloned AST
        if let Some(ast) = ast_clone {
            let mut index = self.workspace_index.lock().await;
            index.index_document(uri.clone(), &text_clone, &ast);

            // Also update symbol index for references
            let mut symbol_index = self.symbol_index.lock().await;
            symbol_index.index_document(&uri, &text_clone, Some(&ast));
        }

        // Publish diagnostics after releasing locks
        if let Some(diags) = diagnostics {
            self.client
                .publish_diagnostics(uri, diags, Some(version))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from state
        {
            let mut documents = self.documents.lock().await;
            documents.remove(&uri);
        }

        // Remove from symbol index
        {
            let mut symbol_index = self.symbol_index.lock().await;
            symbol_index.remove_document(&uri);
        }

        // Clear diagnostics
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            if let Some(ast) = &doc.ast {
                // Use the enhanced symbol extractor with proper ranges
                let symbols = crate::symbols::extract_document_symbols(&doc.text, ast);
                return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
            }
        }

        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query;
        let index = self.workspace_index.lock().await;
        let symbols = index.search(&query, 100);

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(symbols))
        }
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let ranges = crate::folding::generate_folding_ranges(&doc.text, doc.ast.as_ref());
            if ranges.is_empty() {
                return Ok(None);
            }
            return Ok(Some(ranges));
        }

        Ok(None)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;
        let range = params.range;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let hints = crate::inlay_hints::generate_inlay_hints(
                &doc.text,
                range,
                doc.ast.as_ref(),
                doc.symbols.as_ref(),
                &self.inlay_config,
            );
            if hints.is_empty() {
                return Ok(None);
            }
            return Ok(Some(hints));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            if let (Some(_ast), Some(_symbols)) = (&doc.ast, &doc.symbols) {
                if let Some(_identifier) =
                    crate::navigation::find_identifier_at_position(&doc.text, position)
                {
                    // TODO: Implement actual go-to-definition once we have position info in symbol table
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let documents = self.documents.lock().await;
        let symbol_index = self.symbol_index.lock().await;

        if let Some(doc) = documents.get(&uri) {
            let locations = crate::references::find_all_references(
                &uri,
                &doc.text,
                position,
                doc.ast.as_ref(),
                doc.symbols.as_ref(),
                &symbol_index,
                include_declaration,
            );
            return Ok(locations);
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            // Use the enhanced hover provider
            return Ok(crate::hover::generate_hover(
                &doc.text,
                position,
                doc.ast.as_ref(),
                doc.symbols.as_ref(),
            ));
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;
        let context = params.context;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let actions = crate::actions::generate_code_actions(
                &uri,
                range,
                &context,
                &doc.text,
                doc.ast.as_ref(),
                doc.symbols.as_ref(),
                &doc.diagnostics,
            );

            if actions.is_empty() {
                return Ok(None);
            }

            return Ok(Some(actions));
        }

        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let result = semantic_tokens::generate_semantic_tokens(
                &doc.text,
                doc.ast.as_ref(),
                doc.symbols.as_ref(),
            );
            return Ok(Some(result));
        }

        Ok(None)
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = params.text_document.uri;
        let range = params.range;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let result = semantic_tokens::generate_semantic_tokens_range(
                &doc.text,
                range,
                doc.ast.as_ref(),
                doc.symbols.as_ref(),
            );
            return Ok(Some(result));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let completions =
                crate::completion::generate_completions(doc.ast.as_ref(), doc.symbols.as_ref());
            return Ok(Some(CompletionResponse::Array(completions)));
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let edits = crate::formatting::format_document(&doc.text);
            return Ok(Some(edits));
        }

        Ok(None)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let range = params.range;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            let edits = crate::formatting::format_range(&doc.text, range);
            return Ok(Some(edits));
        }

        Ok(None)
    }
}
