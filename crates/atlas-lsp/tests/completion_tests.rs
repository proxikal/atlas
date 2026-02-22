//! Code completion tests

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

#[tokio::test]
async fn test_keyword_completions() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // Open empty document
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

    // Request completions
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

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have keywords
        assert!(items.iter().any(|item| item.label == "let"));
        assert!(items.iter().any(|item| item.label == "fn"));
        assert!(items.iter().any(|item| item.label == "if"));
        assert!(items.iter().any(|item| item.label == "while"));
        assert!(items.iter().any(|item| item.label == "return"));
    }
}

#[tokio::test]
async fn test_builtin_function_completions() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

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

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have builtin functions
        assert!(items.iter().any(|item| item.label == "print"));
        assert!(items.iter().any(|item| item.label == "len"));
        assert!(items.iter().any(|item| item.label == "push"));
        assert!(items.iter().any(|item| item.label == "pop"));
    }
}

#[tokio::test]
async fn test_type_completions() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

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

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have type keywords
        assert!(items.iter().any(|item| item.label == "number"));
        assert!(items.iter().any(|item| item.label == "string"));
        assert!(items.iter().any(|item| item.label == "bool"));
    }
}

#[tokio::test]
async fn test_function_completions_from_document() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}

fn greet(name: string) -> string {
    return name;
}
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 8,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have user-defined functions
        assert!(items.iter().any(|item| item.label == "add"));
        assert!(items.iter().any(|item| item.label == "greet"));

        // Check function details
        let add_item = items.iter().find(|item| item.label == "add");
        assert!(add_item.is_some());
        assert_eq!(add_item.unwrap().kind, Some(CompletionItemKind::FUNCTION));
    }
}

#[tokio::test]
async fn test_variable_completions_from_document() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let source = r#"
var counter: number = 0;
var name: string = "test";
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 3,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have variables
        assert!(items.iter().any(|item| item.label == "counter"));
        assert!(items.iter().any(|item| item.label == "name"));
    }
}

#[tokio::test]
async fn test_snippet_completions() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

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

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Check that snippets have insert text
        let if_item = items.iter().find(|item| item.label == "if");
        assert!(if_item.is_some());
        assert!(if_item.unwrap().insert_text.is_some());
        assert_eq!(
            if_item.unwrap().insert_text_format,
            Some(InsertTextFormat::SNIPPET)
        );

        let fn_item = items.iter().find(|item| item.label == "fn");
        assert!(fn_item.is_some());
        assert!(fn_item.unwrap().insert_text.is_some());
    }
}

#[tokio::test]
async fn test_completions_with_errors() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // Document with syntax errors
    let source = "let x =";

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

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

    let result = server.completion(completion_params).await.unwrap();
    // Should still provide completions even with errors
    assert!(result.is_some());
}

// === Ownership Annotation Completion Tests ===

#[tokio::test]
async fn test_completion_suggests_own_in_param_position() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    // Cursor is inside the parameter list after 'fn f('
    let source = "fn f(own x: number) -> number { return x; }";
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

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                // Position: inside the param list, after '('
                position: Position {
                    line: 0,
                    character: 6,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    let items = match result.unwrap() {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    };

    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"own"),
        "Expected 'own' in completions at param position, got: {labels:?}"
    );
    assert!(
        labels.contains(&"borrow"),
        "Expected 'borrow' in completions at param position, got: {labels:?}"
    );
    assert!(
        labels.contains(&"shared"),
        "Expected 'shared' in completions at param position, got: {labels:?}"
    );

    // Verify KEYWORD kind
    let own_item = items.iter().find(|i| i.label == "own").unwrap();
    assert_eq!(own_item.kind, Some(CompletionItemKind::KEYWORD));
}

#[tokio::test]
async fn test_completion_no_ownership_in_expression_position() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    // Cursor is in an expression, NOT in a parameter list
    let source = "fn f() -> number { let x = 1; return x; }";
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

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                // Position: inside the function body expression
                position: Position {
                    line: 0,
                    character: 28,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    let items = match result.unwrap() {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    };

    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        !labels.contains(&"own"),
        "Did not expect 'own' in expression position completions, got: {labels:?}"
    );
    assert!(
        !labels.contains(&"borrow"),
        "Did not expect 'borrow' in expression position completions, got: {labels:?}"
    );
    assert!(
        !labels.contains(&"shared"),
        "Did not expect 'shared' in expression position completions, got: {labels:?}"
    );
}

#[tokio::test]
async fn test_function_completion_shows_ownership_in_params() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    // Use two functions so there's valid top-level code — request completion from inside f
    let source =
        "fn process(own data: number) -> number { return data; }\nfn f() -> number { return 1; }";
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

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                // Inside the second function body — expression position, not param
                position: Position {
                    line: 1,
                    character: 22,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    let items = match result.unwrap() {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    };

    let process_item = items.iter().find(|i| i.label == "process");
    assert!(
        process_item.is_some(),
        "Expected 'process' in completion list"
    );

    let detail = process_item.unwrap().detail.as_deref().unwrap_or("");
    assert!(
        detail.contains("own"),
        "Expected function completion detail to include ownership annotation 'own', got: {detail}"
    );
}

#[tokio::test]
async fn test_ownership_completion_items_have_documentation() {
    use atlas_lsp::completion::ownership_annotation_completions;

    let items = ownership_annotation_completions();
    assert_eq!(items.len(), 3);

    for item in &items {
        assert!(
            item.documentation.is_some(),
            "Item '{}' missing documentation",
            item.label
        );
        assert_eq!(item.kind, Some(CompletionItemKind::KEYWORD));
        assert!(
            item.insert_text.is_some(),
            "Item '{}' missing insert_text",
            item.label
        );
    }
}

#[tokio::test]
async fn test_is_in_param_position_detection() {
    use atlas_lsp::completion::is_in_param_position;

    // Inside parameter list
    assert!(is_in_param_position(
        "fn f(x: number) -> void { }",
        Position {
            line: 0,
            character: 6
        }
    ));

    // After opening paren, multiple params
    assert!(is_in_param_position(
        "fn process(own data: number, y: string) -> void { }",
        Position {
            line: 0,
            character: 12
        }
    ));

    // In function body — NOT param position
    assert!(!is_in_param_position(
        "fn f() -> void { let x = 1; }",
        Position {
            line: 0,
            character: 20
        }
    ));

    // In a function call — NOT param position (no `fn` before `(`)
    assert!(!is_in_param_position(
        "print(x);",
        Position {
            line: 0,
            character: 7
        }
    ));
}

// === Trait/Impl Completion Tests ===

#[tokio::test]
async fn test_completion_builtin_traits_after_impl() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: "impl ".to_string(),
            },
        })
        .await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 5,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(CompletionResponse::Array(items)) = result {
        assert!(
            items.iter().any(|i| i.label == "Copy"),
            "Expected 'Copy' in completions after `impl`"
        );
        assert!(
            items.iter().any(|i| i.label == "Display"),
            "Expected 'Display' in completions after `impl`"
        );
        assert!(
            items.iter().any(|i| i.label == "Debug"),
            "Expected 'Debug' in completions after `impl`"
        );
    } else {
        panic!("Expected Array completion response");
    }
}

#[tokio::test]
async fn test_completion_user_trait_after_impl() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: "trait MyTrait { }\nimpl ".to_string(),
            },
        })
        .await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 1,
                    character: 5,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(CompletionResponse::Array(items)) = result {
        assert!(
            items.iter().any(|i| i.label == "MyTrait"),
            "Expected user-defined 'MyTrait' in completions after `impl`"
        );
    } else {
        panic!("Expected Array completion response");
    }
}

#[tokio::test]
async fn test_completion_trait_names_classified_as_interface() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: "impl ".to_string(),
            },
        })
        .await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 5,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let trait_items: Vec<_> = items
            .iter()
            .filter(|i| {
                matches!(
                    i.label.as_str(),
                    "Copy" | "Move" | "Drop" | "Display" | "Debug"
                )
            })
            .collect();
        assert!(!trait_items.is_empty(), "Expected trait completions");
        for item in &trait_items {
            assert_eq!(
                item.kind,
                Some(CompletionItemKind::INTERFACE),
                "Trait '{}' should have kind INTERFACE",
                item.label
            );
        }
    } else {
        panic!("Expected Array completion response");
    }
}

#[tokio::test]
async fn test_completion_builtin_traits_have_documentation() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: "impl ".to_string(),
            },
        })
        .await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 5,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let copy_item = items.iter().find(|i| i.label == "Copy");
        assert!(copy_item.is_some(), "Expected 'Copy' in trait completions");
        let copy = copy_item.unwrap();
        assert!(
            copy.documentation.is_some(),
            "'Copy' trait completion should have documentation"
        );
    } else {
        panic!("Expected Array completion response");
    }
}

#[tokio::test]
async fn test_completion_method_stubs_in_impl_body() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: "trait Display { fn display(self: Display) -> string; }\nimpl Display for number {\n    fn "
                    .to_string(),
            },
        })
        .await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 7,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(result.is_some());
    if let Some(CompletionResponse::Array(items)) = result {
        assert!(
            items.iter().any(|i| i.label == "display"),
            "Expected 'display' method stub in completions inside impl body"
        );
        let display_item = items.iter().find(|i| i.label == "display").unwrap();
        assert_eq!(
            display_item.kind,
            Some(CompletionItemKind::METHOD),
            "Method stub should have kind METHOD"
        );
        assert!(
            display_item
                .detail
                .as_deref()
                .map(|d| d.contains("Display"))
                .unwrap_or(false),
            "Method stub detail should mention the trait name"
        );
    } else {
        panic!("Expected Array completion response");
    }
}

#[tokio::test]
async fn test_completion_method_stubs_are_snippets() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    let uri = Url::parse("file:///test.atl").unwrap();

    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "atlas".to_string(),
                version: 1,
                text: "trait Math { fn double(self: Math) -> number; }\nimpl Math for number {\n    fn "
                    .to_string(),
            },
        })
        .await;

    let result = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 7,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .await
        .unwrap();

    if let Some(CompletionResponse::Array(items)) = result {
        let double_item = items.iter().find(|i| i.label == "double");
        assert!(
            double_item.is_some(),
            "Expected 'double' method stub completion"
        );
        let item = double_item.unwrap();
        assert_eq!(
            item.insert_text_format,
            Some(InsertTextFormat::SNIPPET),
            "Method stub should use snippet format"
        );
        assert!(
            item.insert_text.is_some(),
            "Method stub should have insert_text snippet"
        );
    } else {
        panic!("Expected Array completion response");
    }
}

#[test]
fn test_is_after_impl_keyword_basic() {
    use atlas_lsp::completion::is_after_impl_keyword;
    assert!(is_after_impl_keyword(
        "impl ",
        Position {
            line: 0,
            character: 5
        }
    ));
    assert!(is_after_impl_keyword(
        "impl",
        Position {
            line: 0,
            character: 4
        }
    ));
    assert!(!is_after_impl_keyword(
        "let x = 5;",
        Position {
            line: 0,
            character: 5
        }
    ));
    assert!(!is_after_impl_keyword(
        "fn foo() { impl ",
        Position {
            line: 0,
            character: 5
        }
    ));
}
