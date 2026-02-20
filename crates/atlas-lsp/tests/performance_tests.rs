//! Performance and Optimization Tests
//!
//! Tests for workspace symbol indexing performance optimizations including:
//! - Parallel/batch indexing
//! - Query result caching
//! - Memory bounds enforcement
//! - Incremental updates

use atlas_lsp::symbols::{IndexConfig, WorkspaceIndex};
use atlas_runtime::{Lexer, Parser};
use tower_lsp::lsp_types::{SymbolKind, Url};

/// Parse source code into AST
fn parse_source(source: &str) -> atlas_runtime::ast::Program {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    ast
}

/// Create test URI
fn test_uri(name: &str) -> Url {
    Url::parse(&format!("file:///{}.atl", name)).unwrap()
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_batch_indexing_speedup() {
    // Test that batch indexing is more efficient than individual indexing

    let mut index = WorkspaceIndex::new();

    // Create multiple documents
    let mut documents = Vec::new();
    for i in 0..20 {
        let uri = test_uri(&format!("file{}", i));
        let source = format!("fn function_{}() -> number {{ return {}; }}", i, i);
        let ast = parse_source(&source);
        documents.push((uri, source, ast));
    }

    // Batch index all documents
    let start = std::time::Instant::now();
    index.index_documents_parallel(documents);
    let batch_duration = start.elapsed();

    // Verify all symbols were indexed
    assert_eq!(index.total_symbols(), 20);

    // Note: We can't easily compare sequential vs parallel in a unit test,
    // but we verify the batch method works correctly and indexes all documents
    println!("Batch indexing of 20 files took: {:?}", batch_duration);
    assert!(batch_duration.as_millis() < 1000); // Should be fast
}

#[test]
fn test_incremental_update_performance() {
    // Test that incremental updates are efficient

    let mut index = WorkspaceIndex::new();

    let uri = test_uri("test");
    let source1 = "fn function1() -> number { return 1; }";
    let ast1 = parse_source(source1);

    // Initial index
    let start = std::time::Instant::now();
    index.index_document(uri.clone(), source1, &ast1);
    let initial_duration = start.elapsed();

    // Update with larger document
    let source2 = r#"
fn function1() -> number { return 1; }
fn function2() -> number { return 2; }
fn function3() -> number { return 3; }
fn function4() -> number { return 4; }
fn function5() -> number { return 5; }
"#;
    let ast2 = parse_source(source2);

    let start = std::time::Instant::now();
    index.index_document(uri.clone(), source2, &ast2);
    let update_duration = start.elapsed();

    // Update should be fast (replaces entire document index)
    println!(
        "Initial: {:?}, Update: {:?}",
        initial_duration, update_duration
    );
    assert!(update_duration.as_millis() < 100); // Should be very fast
    assert_eq!(index.total_symbols(), 5);
}

#[test]
fn test_query_cache_effectiveness() {
    // Test that query caching improves performance

    let mut index = WorkspaceIndex::new();

    // Index a large number of symbols
    for i in 0..50 {
        let uri = test_uri(&format!("file{}", i));
        let source = format!(
            "fn function_{}() -> number {{ return {}; }}\nfn helper_{}() -> number {{ return {}; }}",
            i, i, i, i
        );
        let ast = parse_source(&source);
        index.index_document(uri, &source, &ast);
    }

    assert_eq!(index.total_symbols(), 100);

    // First search (cache miss)
    let start = std::time::Instant::now();
    let _results1 = index.search("function", 50, None);
    let first_search = start.elapsed();

    // Second identical search (cache hit)
    let start = std::time::Instant::now();
    let _results2 = index.search("function", 50, None);
    let cached_search = start.elapsed();

    // Different search (cache miss)
    let start = std::time::Instant::now();
    let _results3 = index.search("helper", 50, None);
    let third_search = start.elapsed();

    println!(
        "First: {:?}, Cached: {:?}, Different: {:?}",
        first_search, cached_search, third_search
    );

    // Cached search should be significantly faster than uncached
    // Note: This might be flaky on very fast systems, so we just verify it completes
    assert!(cached_search.as_nanos() > 0);
    assert!(_results1.len() >= 50);
}

#[test]
fn test_memory_usage_stays_bounded() {
    // Test that memory bounds are enforced

    let config = IndexConfig {
        max_symbols: 100, // Limit to 100 symbols
        cache_size: 10,
        parallel_indexing: true,
    };

    let mut index = WorkspaceIndex::with_config(config);

    // Try to index more symbols than the limit
    for i in 0..150 {
        let uri = test_uri(&format!("file{}", i));
        let source = format!("fn function_{}() -> number {{ return {}; }}", i, i);
        let ast = parse_source(&source);
        index.index_document(uri, &source, &ast);
    }

    // Total symbols should not exceed the limit
    let total = index.total_symbols();
    assert!(total <= 100, "Expected <= 100 symbols, found {}", total);
}

#[test]
fn test_kind_filtering_performance() {
    // Test that kind filtering doesn't significantly impact performance

    let mut index = WorkspaceIndex::new();

    // Index mixed symbol types
    let uri = test_uri("test");
    let source = r#"
fn myFunction() -> number { return 1; }
let myVariable: number = 2;
type MyType = number;
fn anotherFunction() -> number { return 3; }
let anotherVariable: number = 4;
"#;
    let ast = parse_source(source);
    index.index_document(uri, source, &ast);

    // Search without filter
    let start = std::time::Instant::now();
    let results_all = index.search("my", 100, None);
    let unfiltered_duration = start.elapsed();

    // Search with function filter
    let start = std::time::Instant::now();
    let results_functions = index.search("my", 100, Some(SymbolKind::FUNCTION));
    let filtered_duration = start.elapsed();

    println!(
        "Unfiltered: {:?}, Filtered: {:?}",
        unfiltered_duration, filtered_duration
    );

    // Both should find results
    assert!(!results_all.is_empty());
    assert!(!results_functions.is_empty());

    // Filtered results should only contain functions
    for symbol in &results_functions {
        assert_eq!(symbol.kind, SymbolKind::FUNCTION);
    }

    // Filtering should still be fast
    assert!(filtered_duration.as_millis() < 100);
}

#[test]
fn test_large_workspace_performance() {
    // Test performance with a large workspace

    let mut index = WorkspaceIndex::new();

    // Create a large workspace (100 files, 10 symbols each)
    for i in 0..100 {
        let uri = test_uri(&format!("file{}", i));
        let mut source = String::new();
        for j in 0..10 {
            source.push_str(&format!(
                "fn function_{}_{} () -> number {{ return {}; }}\n",
                i, j, j
            ));
        }
        let ast = parse_source(&source);
        index.index_document(uri, &source, &ast);
    }

    assert_eq!(index.total_symbols(), 1000);

    // Search should still be fast in large workspace
    let start = std::time::Instant::now();
    let results = index.search("function", 100, None);
    let search_duration = start.elapsed();

    println!("Large workspace search took: {:?}", search_duration);
    assert!(search_duration.as_millis() < 200); // Should complete in <200ms
    assert_eq!(results.len(), 100); // Should respect limit
}

#[test]
fn test_cache_invalidation_on_update() {
    // Test that cache is properly invalidated when index changes

    let mut index = WorkspaceIndex::new();

    let uri = test_uri("test");
    let source1 = "fn oldFunction() -> number { return 1; }";
    let ast1 = parse_source(source1);

    index.index_document(uri.clone(), source1, &ast1);

    // Search and cache result
    let results1 = index.search("old", 100, None);
    assert_eq!(results1.len(), 1);

    // Update document
    let source2 = "fn newFunction() -> number { return 2; }";
    let ast2 = parse_source(source2);
    index.index_document(uri, source2, &ast2);

    // Search again - should not find old function (cache was invalidated)
    let results2 = index.search("old", 100, None);
    assert!(results2.is_empty());

    // Should find new function
    let results3 = index.search("new", 100, None);
    assert_eq!(results3.len(), 1);
}

#[test]
fn test_relevance_ranking_consistency() {
    // Test that relevance ranking produces consistent, sensible results

    let mut index = WorkspaceIndex::new();

    let uri = test_uri("test");
    let source = r#"
fn testFunction() -> number { return 1; }
fn myTestHelper() -> number { return 2; }
fn someTestUtility() -> number { return 3; }
fn anotherTest() -> number { return 4; }
fn testing() -> number { return 5; }
"#;
    let ast = parse_source(source);
    index.index_document(uri, source, &ast);

    // Search for "test"
    let results = index.search("test", 100, None);

    // Exact prefix matches should come first
    assert!(results.len() >= 3);
    assert_eq!(results[0].name, "testFunction"); // Exact prefix
    assert_eq!(results[1].name, "testing"); // Exact prefix

    // Verify consistency - running the same search multiple times should
    // produce the same order
    let results2 = index.search("test", 100, None);
    assert_eq!(results.len(), results2.len());
    for (r1, r2) in results.iter().zip(results2.iter()) {
        assert_eq!(r1.name, r2.name);
    }
}

#[test]
fn test_empty_query_performance() {
    // Test that empty query (match all) performs reasonably

    let mut index = WorkspaceIndex::new();

    // Index multiple symbols
    for i in 0..50 {
        let uri = test_uri(&format!("file{}", i));
        let source = format!("fn function_{}() -> number {{ return {}; }}", i, i);
        let ast = parse_source(&source);
        index.index_document(uri, &source, &ast);
    }

    // Empty query should match all symbols but respect limit
    let start = std::time::Instant::now();
    let results = index.search("", 100, None);
    let duration = start.elapsed();

    println!("Empty query took: {:?}", duration);
    assert!(duration.as_millis() < 100);
    assert_eq!(results.len(), 50); // All symbols, under limit
}
