//! Atlas LSP Server implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::DocumentState;
use crate::semantic_tokens;

/// Atlas Language Server
pub struct AtlasLspServer {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

impl AtlasLspServer {
    /// Create a new Atlas LSP server
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
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

        // Publish diagnostics
        let diagnostics = doc
            .diagnostics
            .iter()
            .map(crate::convert::diagnostic_to_lsp)
            .collect();

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;

        // Store document
        let mut documents = self.documents.lock().await;
        documents.insert(uri, doc);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        let mut documents = self.documents.lock().await;
        if let Some(doc) = documents.get_mut(&uri) {
            // Update document text (full sync)
            for change in params.content_changes {
                doc.update(change.text, version);
            }

            // Publish diagnostics
            let diagnostics = doc
                .diagnostics
                .iter()
                .map(crate::convert::diagnostic_to_lsp)
                .collect();

            self.client
                .publish_diagnostics(uri.clone(), diagnostics, Some(version))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from state
        let mut documents = self.documents.lock().await;
        documents.remove(&uri);

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
                let symbols = crate::navigation::extract_document_symbols(ast);
                return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
            }
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
        let _uri = params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;

        // TODO: Implement find references
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
