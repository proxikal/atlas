//! LSP integration and edge case tests

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, LanguageServer};

#[tokio::test]
async fn test_full_workflow_with_diagnostics_and_completion() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///workflow.atl").unwrap();

    // 1. Initialize server
    let init_params = InitializeParams {
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
    let result = server.initialize(init_params).await.unwrap();
    assert!(result.capabilities.text_document_sync.is_some());

    server.initialized(InitializedParams {}).await;

    // 2. Open document with error
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "let x =".to_string(), // Syntax error
        },
    };
    server.did_open(open_params).await;

    // 3. Get completions (should still work with errors)
    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 7,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };
    let completions = server.completion(completion_params).await.unwrap();
    assert!(completions.is_some());

    // 4. Fix the error
    let change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
"#
            .to_string(),
        }],
    };
    server.did_change(change_params).await;

    // 5. Get document symbols
    let symbol_params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let symbols = server.document_symbol(symbol_params).await.unwrap();
    assert!(symbols.is_some());

    // 6. Get hover
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 4,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    let hover = server.hover(hover_params).await.unwrap();
    assert!(hover.is_some());

    // 7. Format document
    let format_params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        options: FormattingOptions {
            tab_size: 2,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    let edits = server.formatting(format_params).await.unwrap();
    assert!(edits.is_some());

    // 8. Close document
    let close_params = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    };
    server.did_close(close_params).await;

    // 9. Shutdown
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_multiple_documents_simultaneously() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    // Open 5 documents simultaneously
    for i in 1..=5 {
        let uri = Url::parse(&format!("file:///file{}.atl", i)).unwrap();
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "atlas".to_string(),
                version: 1,
                text: format!("var x{}: number = {};", i, i * 10),
            },
        };
        server.did_open(open_params).await;
    }

    // All should be tracked independently
    // Verify by getting completions from each
    for i in 1..=5 {
        let uri = Url::parse(&format!("file:///file{}.atl", i)).unwrap();
        let completion_params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };
        let result = server.completion(completion_params).await.unwrap();
        assert!(result.is_some());
    }
}

#[tokio::test]
async fn test_large_document_performance() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///large.atl").unwrap();

    // Create a large document (100 functions)
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!(
            "fn func{}(x: number) -> number {{ return x + {}; }}\n",
            i, i
        ));
    }

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source,
        },
    };

    // This should complete without timeout
    server.did_open(open_params).await;

    // Get document symbols (should handle large AST)
    let symbol_params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let symbols = server.document_symbol(symbol_params).await.unwrap();
    assert!(symbols.is_some());

    if let Some(DocumentSymbolResponse::Nested(symbols)) = symbols {
        assert_eq!(symbols.len(), 100); // Should have all 100 functions
    }
}

#[tokio::test]
async fn test_invalid_document_uri() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    // Try to get completions from non-existent document
    let uri = Url::parse("file:///nonexistent.atl").unwrap();
    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    // Should return None for non-existent document
    assert!(result.is_none());
}

#[tokio::test]
async fn test_out_of_bounds_positions() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "let x: number = 42;".to_string(),
        },
    };
    server.did_open(open_params).await;

    // Try to hover at an out-of-bounds position
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 100,    // Way past end of document
                character: 50,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.hover(hover_params).await.unwrap();
    // Should handle gracefully (return None or empty)
    // Should not panic
    let _result = result;
}

#[tokio::test]
async fn test_unicode_content() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///unicode.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: r#"var greeting: string = "Hello 世界 🌍";"#.to_string(),
        },
    };
    server.did_open(open_params).await;

    // Get completions (should handle unicode correctly)
    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 0,
                character: 5,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());
}

#[tokio::test]
async fn test_empty_file_all_operations() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///empty.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

    // All operations should work on empty file
    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };
    assert!(server.completion(completion_params).await.unwrap().is_some());

    let symbol_params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    server.document_symbol(symbol_params).await.unwrap();

    let format_params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri },
        options: FormattingOptions::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    server.formatting(format_params).await.unwrap();
}
