//! Document synchronization tests

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, LanguageServer};

#[tokio::test]
async fn test_did_open() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: Url::parse("file:///test.atl").unwrap(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "let x: number = 42;".to_string(),
        },
    };

    // Should not panic
    server.did_open(params).await;
}

#[tokio::test]
async fn test_did_change() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // First open the document
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "let x: number = 42;".to_string(),
        },
    };
    server.did_open(open_params).await;

    // Then change it
    let change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "let y: string = \"hello\";".to_string(),
        }],
    };

    // Should not panic
    server.did_change(change_params).await;
}

#[tokio::test]
async fn test_did_close() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // First open the document
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "let x: number = 42;".to_string(),
        },
    };
    server.did_open(open_params).await;

    // Then close it
    let close_params = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    };

    // Should not panic
    server.did_close(close_params).await;
}

#[tokio::test]
async fn test_full_document_lifecycle() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // Open
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "let x: number = 42;".to_string(),
        },
    };
    server.did_open(open_params).await;

    // Change multiple times
    for i in 2..=5 {
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: i,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: format!("let x: number = {};", i * 10),
            }],
        };
        server.did_change(change_params).await;
    }

    // Close
    let close_params = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    };
    server.did_close(close_params).await;
}

#[tokio::test]
async fn test_multiple_documents() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    // Open multiple documents
    for i in 1..=3 {
        let uri = Url::parse(&format!("file:///test{}.atl", i)).unwrap();
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "atlas".to_string(),
                version: 1,
                text: format!("let x{}: number = {};", i, i * 10),
            },
        };
        server.did_open(open_params).await;
    }

    // All documents should be independently tracked
    // (If we had a way to query document state, we would verify here)
}
