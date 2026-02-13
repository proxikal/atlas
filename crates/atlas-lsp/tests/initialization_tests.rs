//! LSP server initialization tests

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, LanguageServer};

#[tokio::test]
async fn test_server_initialization() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let params = InitializeParams {
        process_id: Some(1),
        root_uri: None,
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None,
        ..Default::default()
    };

    let result = server.initialize(params).await.unwrap();

    // Verify server info
    assert!(result.server_info.is_some());
    let server_info = result.server_info.unwrap();
    assert_eq!(server_info.name, "atlas-lsp");
    assert!(server_info.version.is_some());

    // Verify capabilities
    assert!(result.capabilities.text_document_sync.is_some());
    assert!(result.capabilities.diagnostic_provider.is_some());
}

#[tokio::test]
async fn test_server_initialized() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    // Should not panic
    server.initialized(InitializedParams {}).await;
}

#[tokio::test]
async fn test_server_shutdown() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let result = server.shutdown().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_full_lifecycle() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    // Initialize
    let params = InitializeParams {
        process_id: Some(1),
        root_uri: None,
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None,
        ..Default::default()
    };

    let init_result = server.initialize(params).await;
    assert!(init_result.is_ok());

    // Initialized notification
    server.initialized(InitializedParams {}).await;

    // Shutdown
    let shutdown_result = server.shutdown().await;
    assert!(shutdown_result.is_ok());
}
