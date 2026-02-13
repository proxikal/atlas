//! Atlas LSP Server implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::DocumentState;

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
        self.client
            .publish_diagnostics(uri, Vec::new(), None)
            .await;
    }
}
