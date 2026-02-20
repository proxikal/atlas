//! Call Hierarchy feature tests
//!
//! Tests callHierarchy/* LSP functionality including:
//! - Incoming calls (who calls this function)
//! - Outgoing calls (what this function calls)
//! - Recursive calls
//! - Cross-file call resolution

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

/// Helper to create test URI
fn test_uri(name: &str) -> Url {
    Url::parse(&format!("file:///{}.atl", name)).unwrap()
}

// ============================================================================
// Incoming Calls Tests
// ============================================================================

#[tokio::test]
async fn test_incoming_calls_direct_caller() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn helper() -> number {
    return 42;
}

fn main() -> void {
    var x: number = helper();
    print(x);
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

    // Prepare call hierarchy for helper() at line 1 (0-indexed, fn helper() line)
    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3, // Position on "helper"
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server.prepare_call_hierarchy(prepare_params).await.unwrap();
    assert!(items.is_some());

    let items = items.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "helper");

    // Get incoming calls
    let incoming_params = CallHierarchyIncomingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let incoming = server.incoming_calls(incoming_params).await.unwrap();
    assert!(incoming.is_some());

    let calls = incoming.unwrap();
    assert!(
        !calls.is_empty(),
        "Expected incoming calls from main to helper"
    );

    // Verify the caller is main
    assert_eq!(calls[0].from.name, "main");
}

#[tokio::test]
async fn test_incoming_calls_multiple_callers() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn utility() -> number {
    return 123;
}

fn caller1() -> number {
    return utility() + 1;
}

fn caller2() -> number {
    return utility() * 2;
}

fn caller3() -> number {
    var x: number = utility();
    return x;
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

    // Prepare for utility()
    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let incoming_params = CallHierarchyIncomingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let incoming = server.incoming_calls(incoming_params).await.unwrap();
    assert!(incoming.is_some());

    let calls = incoming.unwrap();
    assert_eq!(calls.len(), 3, "Expected 3 callers of utility");

    // Check all callers are present
    let caller_names: Vec<String> = calls.iter().map(|c| c.from.name.clone()).collect();
    assert!(caller_names.contains(&"caller1".to_string()));
    assert!(caller_names.contains(&"caller2".to_string()));
    assert!(caller_names.contains(&"caller3".to_string()));
}

#[tokio::test]
async fn test_incoming_calls_no_callers() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn unused() -> number {
    return 999;
}

fn main() -> void {
    print("hello");
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let incoming_params = CallHierarchyIncomingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let incoming = server.incoming_calls(incoming_params).await.unwrap();
    // No incoming calls for unused function
    assert!(incoming.is_none() || incoming.unwrap().is_empty());
}

#[tokio::test]
async fn test_incoming_calls_method_style() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn process(data: number) -> number {
    return data * 2;
}

fn main() -> void {
    var result: number = process(42);
    print(result);
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let incoming_params = CallHierarchyIncomingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let incoming = server.incoming_calls(incoming_params).await.unwrap();
    assert!(incoming.is_some());
    assert_eq!(incoming.unwrap()[0].from.name, "main");
}

#[tokio::test]
async fn test_incoming_calls_exported_function() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn publicHelper() -> string {
    return "helper";
}

fn consumer() -> string {
    return publicHelper();
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3, // On "publicHelper"
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let incoming_params = CallHierarchyIncomingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let incoming = server.incoming_calls(incoming_params).await.unwrap();
    assert!(incoming.is_some());
    assert_eq!(incoming.unwrap()[0].from.name, "consumer");
}

#[tokio::test]
async fn test_incoming_calls_nested_in_control_flow() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn check() -> boolean {
    return true;
}

fn conditional() -> void {
    if (check()) {
        print("yes");
    }
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let incoming_params = CallHierarchyIncomingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let incoming = server.incoming_calls(incoming_params).await.unwrap();
    assert!(incoming.is_some());
    assert_eq!(incoming.unwrap()[0].from.name, "conditional");
}

// ============================================================================
// Outgoing Calls Tests
// ============================================================================

#[tokio::test]
async fn test_outgoing_calls_direct_callees() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn helper1() -> number {
    return 1;
}

fn helper2() -> number {
    return 2;
}

fn main() -> void {
    var a: number = helper1();
    var b: number = helper2();
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

    // Prepare for main() - on line 9 (fn main line)
    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 9,
                character: 3, // On "main"
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());

    let calls = outgoing.unwrap();
    assert_eq!(calls.len(), 2, "Expected 2 outgoing calls from main");

    let callee_names: Vec<String> = calls.iter().map(|c| c.to.name.clone()).collect();
    assert!(callee_names.contains(&"helper1".to_string()));
    assert!(callee_names.contains(&"helper2".to_string()));
}

#[tokio::test]
async fn test_outgoing_calls_no_callees() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn leaf() -> number {
    return 42;
}

fn main() -> void {
    print("done");
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    // Leaf function has no outgoing calls (except stdlib which may not be indexed)
    assert!(outgoing.is_none() || outgoing.unwrap().is_empty());
}

#[tokio::test]
async fn test_outgoing_calls_multiple_callees() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn fn1() -> number { return 1; }
fn fn2() -> number { return 2; }
fn fn3() -> number { return 3; }

fn orchestrator() -> number {
    return fn1() + fn2() + fn3();
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 5, // orchestrator is on line 5
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());

    let calls = outgoing.unwrap();
    assert_eq!(calls.len(), 3, "Expected 3 outgoing calls");

    let callee_names: Vec<String> = calls.iter().map(|c| c.to.name.clone()).collect();
    assert!(callee_names.contains(&"fn1".to_string()));
    assert!(callee_names.contains(&"fn2".to_string()));
    assert!(callee_names.contains(&"fn3".to_string()));
}

#[tokio::test]
async fn test_outgoing_calls_in_branches() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn trueBranch() -> void {}
fn falseBranch() -> void {}

fn conditional(flag: boolean) -> void {
    if (flag) {
        trueBranch();
    } else {
        falseBranch();
    }
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 4, // conditional is on line 4
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());

    let calls = outgoing.unwrap();
    assert_eq!(calls.len(), 2, "Expected calls from both branches");

    let callee_names: Vec<String> = calls.iter().map(|c| c.to.name.clone()).collect();
    assert!(callee_names.contains(&"trueBranch".to_string()));
    assert!(callee_names.contains(&"falseBranch".to_string()));
}

#[tokio::test]
async fn test_outgoing_calls_including_stdlib() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn customFunc() -> number {
    return 123;
}

fn mixed() -> void {
    var x: number = customFunc();
    print(x);
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 5, // mixed is on line 5
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());

    // Should find customFunc (print may or may not be indexed)
    let calls = outgoing.unwrap();
    let callee_names: Vec<String> = calls.iter().map(|c| c.to.name.clone()).collect();
    assert!(callee_names.contains(&"customFunc".to_string()));
}

// ============================================================================
// Recursive Calls Tests
// ============================================================================

#[tokio::test]
async fn test_recursive_calls_direct_recursion() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    // Check outgoing calls - should call itself
    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());

    let calls = outgoing.unwrap();
    assert!(calls.iter().any(|c| c.to.name == "factorial"));

    // Check incoming calls - should call itself
    let incoming_params = CallHierarchyIncomingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let incoming = server.incoming_calls(incoming_params).await.unwrap();
    assert!(incoming.is_some());

    let calls = incoming.unwrap();
    assert!(calls.iter().any(|c| c.from.name == "factorial"));
}

#[tokio::test]
async fn test_recursive_calls_mutual_recursion() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn isEven(n: number) -> boolean {
    if (n == 0) { return true; }
    return isOdd(n - 1);
}

fn isOdd(n: number) -> boolean {
    if (n == 0) { return false; }
    return isEven(n - 1);
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

    // Test isEven
    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());

    // isEven should call isOdd
    let calls = outgoing.unwrap();
    assert!(calls.iter().any(|c| c.to.name == "isOdd"));
}

#[tokio::test]
async fn test_recursive_call_tree_navigation() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn countdown(n: number) -> void {
    if (n <= 0) { return; }
    print(n);
    countdown(n - 1);
}

fn main() -> void {
    countdown(5);
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

    // Start from main (line 7)
    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 7,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    // main should call countdown
    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());
    assert!(outgoing.unwrap().iter().any(|c| c.to.name == "countdown"));
}

#[tokio::test]
async fn test_recursive_depth_handling() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = test_uri("test");
    let source = r#"
fn deepRecursion(n: number) -> number {
    if (n <= 0) { return 0; }
    return deepRecursion(n - 1) + 1;
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

    let prepare_params = CallHierarchyPrepareParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 3,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let items = server
        .prepare_call_hierarchy(prepare_params)
        .await
        .unwrap()
        .unwrap();

    let outgoing_params = CallHierarchyOutgoingCallsParams {
        item: items[0].clone(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let outgoing = server.outgoing_calls(outgoing_params).await.unwrap();
    assert!(outgoing.is_some());

    // Should find the recursive call
    let calls = outgoing.unwrap();
    assert!(calls.iter().any(|c| c.to.name == "deepRecursion"));
}
