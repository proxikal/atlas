//! Workspace Symbol Search Tests
//!
//! Tests for workspace-wide symbol search with filtering, fuzzy matching,
//! relevance ranking, and performance optimizations.

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

/// Helper to create test URI
fn test_uri(name: &str) -> Url {
    Url::parse(&format!("file:///{}.atl", name)).unwrap()
}

// ============================================================================
// Workspace Symbol Search Tests
// ============================================================================

#[tokio::test]
async fn test_workspace_symbol_exact_match() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn calculateSum(a: number, b: number) -> number {
    return a + b;
}

fn calculateProduct(a: number, b: number) -> number {
    return a * b;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Search for exact function name
    let params = WorkspaceSymbolParams {
        query: "calculateSum".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    assert!(result.is_some());

    let symbols = result.unwrap();
    assert!(!symbols.is_empty());
    assert!(symbols.iter().any(|s| s.name == "calculateSum"));
}

#[tokio::test]
async fn test_workspace_symbol_fuzzy_match() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn myVariableName() -> string {
    return "test";
}

fn myValueNotice() -> number {
    return 42;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Fuzzy search with abbreviated query
    let params = WorkspaceSymbolParams {
        query: "mvn".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    assert!(result.is_some());

    // Should match "myVariableName" and possibly "myValueNotice"
    let symbols = result.unwrap();
    assert!(!symbols.is_empty());
}

#[tokio::test]
async fn test_workspace_symbol_prefix_match() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn getUserData() -> string {
    return "data";
}

fn getUserInfo() -> string {
    return "info";
}

fn getProductList() -> number {
    return 0;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Search with prefix
    let params = WorkspaceSymbolParams {
        query: "getUser".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    assert!(result.is_some());

    let symbols = result.unwrap();
    // Should match getUserData and getUserInfo, but not getProductList
    assert!(symbols.len() >= 2);
    assert!(symbols.iter().any(|s| s.name == "getUserData"));
    assert!(symbols.iter().any(|s| s.name == "getUserInfo"));
}

#[tokio::test]
async fn test_workspace_symbol_across_multiple_files() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    // File 1
    let uri1 = test_uri("file1");
    let source1 = r#"
fn helperFunction() -> number {
    return 1;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri1.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source1.to_string(),
            },
        })
        .await;

    // File 2
    let uri2 = test_uri("file2");
    let source2 = r#"
fn helperUtility() -> number {
    return 2;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri2.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source2.to_string(),
            },
        })
        .await;

    // Search across both files
    let params = WorkspaceSymbolParams {
        query: "helper".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    assert!(result.is_some());

    let symbols = result.unwrap();
    assert!(symbols.len() >= 2);
    assert!(symbols.iter().any(|s| s.name == "helperFunction"));
    assert!(symbols.iter().any(|s| s.name == "helperUtility"));
}

#[tokio::test]
async fn test_workspace_symbol_relevance_ranking() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn testFunction() -> number {
    return 1;
}

fn myTestHelper() -> number {
    return 2;
}

fn anotherTest() -> number {
    return 3;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Search for "test" - exact prefix matches should come first
    let params = WorkspaceSymbolParams {
        query: "test".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    assert!(result.is_some());

    let symbols = result.unwrap();
    assert!(symbols.len() >= 2);

    // First result should be "testFunction" (exact prefix match)
    // rather than "myTestHelper" or "anotherTest" (substring matches)
    assert_eq!(symbols[0].name, "testFunction");
}

#[tokio::test]
async fn test_workspace_symbol_partial_query() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn processData() -> number {
    return 1;
}

fn processRequest() -> number {
    return 2;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Partial query
    let params = WorkspaceSymbolParams {
        query: "proc".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    assert!(result.is_some());

    let symbols = result.unwrap();
    assert!(symbols.len() >= 2);
}

#[tokio::test]
async fn test_workspace_symbol_no_results() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn someFunction() -> number {
    return 1;
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Search for nonexistent symbol
    let params = WorkspaceSymbolParams {
        query: "nonExistentSymbol".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    // Should return None or empty list for no matches
    if let Some(symbols) = result {
        assert!(symbols.is_empty());
    }
}

#[tokio::test]
async fn test_workspace_symbol_large_workspace() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    // Create multiple files with many symbols
    for i in 0..10 {
        let uri = test_uri(&format!("file{}", i));
        let mut source = String::new();

        // Add 10 functions per file
        for j in 0..10 {
            source.push_str(&format!(
                "fn function_{}_{} () -> number {{ return {}; }}\n",
                i, j, j
            ));
        }

        server
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "atlas".to_string(),
                    version: 1,
                    text: source,
                },
            })
            .await;
    }

    // Search in large workspace
    let params = WorkspaceSymbolParams {
        query: "function".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.symbol(params).await.unwrap();
    assert!(result.is_some());

    let symbols = result.unwrap();
    // Should find many symbols but respect the limit (100)
    assert!(!symbols.is_empty());
    assert!(symbols.len() <= 100);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_combined_navigation_workflow() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn targetFunction() -> number {
    return 42;
}

fn caller() -> number {
    return targetFunction();
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // 1. Workspace symbol search to find function
    let symbol_params = WorkspaceSymbolParams {
        query: "target".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let symbol_result = server.symbol(symbol_params).await.unwrap();
    assert!(symbol_result.is_some());
    let symbols = symbol_result.unwrap();
    assert!(symbols.iter().any(|s| s.name == "targetFunction"));

    // 2. Find references to the function
    let ref_params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: ReferenceContext {
            include_declaration: true,
        },
    };

    let ref_result = server.references(ref_params).await.unwrap();
    assert!(ref_result.is_some());

    // 3. Call hierarchy for the function
    let call_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let call_result = server.prepare_call_hierarchy(call_params).await.unwrap();
    assert!(call_result.is_some());
}

#[tokio::test]
async fn test_index_consistency_across_operations() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn myFunction() -> number {
    return 1;
}
"#;

    // Open document
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Search for symbol
    let params1 = WorkspaceSymbolParams {
        query: "myFunction".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result1 = server.symbol(params1).await.unwrap();
    assert!(result1.is_some());

    // Update document
    server
        .did_change(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: r#"
fn myFunction() -> number {
    return 1;
}

fn anotherFunction() -> number {
    return 2;
}
"#
                .to_string(),
            }],
        })
        .await;

    // Search again - should find both functions
    let params2 = WorkspaceSymbolParams {
        query: "Function".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result2 = server.symbol(params2).await.unwrap();
    assert!(result2.is_some());

    let symbols = result2.unwrap();
    assert!(symbols.len() >= 2);
}

#[tokio::test]
async fn test_navigation_full_workflow() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn helper() -> number {
    return 1;
}

fn main() -> number {
    return helper();
}
"#;

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: source.to_string(),
            },
        })
        .await;

    // Test workspace symbol, references, and call hierarchy all work together
    let symbol_result = server
        .symbol(WorkspaceSymbolParams {
            query: "helper".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .await
        .unwrap();

    assert!(symbol_result.is_some());

    let ref_result = server
        .references(ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 3,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: false,
            },
        })
        .await
        .unwrap();

    assert!(ref_result.is_some());

    let call_result = server
        .prepare_call_hierarchy(CallHierarchyPrepareParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: 1,
                    character: 3,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .await
        .unwrap();

    assert!(call_result.is_some());
}
