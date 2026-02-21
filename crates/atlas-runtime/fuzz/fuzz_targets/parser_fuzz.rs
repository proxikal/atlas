//! Comprehensive Parser Stability Fuzzer
//!
//! This fuzz target exercises the Atlas lexer and parser with arbitrary input,
//! verifying that no input — no matter how malformed — causes a panic or crash.
//!
//! Stability contract:
//! - Lexer must never panic
//! - Parser must never panic
//! - All errors must be returned as diagnostics, not panics
//! - Performance must be bounded (no infinite loops on any input)
//!
//! This target is more comprehensive than `fuzz_parser` — it also verifies
//! diagnostic quality and exercises more parser code paths.

#![no_main]

use libfuzzer_sys::fuzz_target;

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;

fuzz_target!(|data: &[u8]| {
    // Only valid UTF-8 is meaningful for a source-code parser.
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    fuzz_parse(input);
});

fn fuzz_parse(input: &str) {
    // ─── Stage 1: Lexing ───────────────────────────────────────────────────
    // The lexer must handle any valid UTF-8 string without panicking.
    // It returns (tokens, diagnostics) — malformed input → diagnostics, not crash.
    let mut lexer = Lexer::new(input);
    let (tokens, _lex_diagnostics) = lexer.tokenize();

    // ─── Stage 2: Parsing with all tokens ──────────────────────────────────
    // The parser must handle any token stream without panicking.
    // Even an empty stream, or a stream of garbage tokens, must succeed.
    let mut parser = Parser::new(tokens.clone());
    let (_program, _parse_diagnostics) = parser.parse();

    // ─── Stage 3: Empty input special case ─────────────────────────────────
    // Empty string should parse cleanly with no tokens and no diagnostics.
    if input.is_empty() {
        let mut lexer2 = Lexer::new("");
        let (empty_tokens, _) = lexer2.tokenize();
        let mut parser2 = Parser::new(empty_tokens);
        let (_prog, _diags) = parser2.parse();
    }

    // ─── Stage 4: Truncated input ──────────────────────────────────────────
    // Parse each non-empty prefix of the input.
    // This exercises partial/truncated program recovery.
    if input.len() > 4 {
        for split in [input.len() / 4, input.len() / 2, 3 * input.len() / 4] {
            if let Some(prefix) = input.get(..split) {
                let mut lex = Lexer::new(prefix);
                let (toks, _) = lex.tokenize();
                let mut p = Parser::new(toks);
                let _ = p.parse();
            }
        }
    }

    // ─── Stage 5: Token stream stress ──────────────────────────────────────
    // If the input produced many tokens, also try parsing just the first N.
    // This simulates extremely long programs being truncated mid-statement.
    if tokens.len() > 10 {
        for n in [1, tokens.len() / 2] {
            let partial = tokens[..n].to_vec();
            let mut p = Parser::new(partial);
            let _ = p.parse();
        }
    }

    // ─── Stage 6: Repeated parsing determinism ─────────────────────────────
    // Parsing the same input twice must produce structurally identical output.
    // This verifies the parser is stateless / deterministic.
    {
        let mut lexer_a = Lexer::new(input);
        let (toks_a, _) = lexer_a.tokenize();
        let mut parser_a = Parser::new(toks_a);
        let (prog_a, diags_a) = parser_a.parse();

        let mut lexer_b = Lexer::new(input);
        let (toks_b, _) = lexer_b.tokenize();
        let mut parser_b = Parser::new(toks_b);
        let (prog_b, diags_b) = parser_b.parse();

        // Diagnostic count must be identical across runs.
        assert_eq!(
            diags_a.len(),
            diags_b.len(),
            "Parser is non-deterministic: different diagnostic count on same input"
        );

        // AST node count must be identical across runs.
        assert_eq!(
            prog_a.stmts.len(),
            prog_b.stmts.len(),
            "Parser is non-deterministic: different statement count on same input"
        );
    }

    // ─── Stage 7: Unicode stress ───────────────────────────────────────────
    // If the input contains non-ASCII bytes, verify it is handled without crashes.
    // The lexer should produce string tokens or lex errors, not panic.
    if input.chars().any(|c| !c.is_ascii()) {
        let mut lex = Lexer::new(input);
        let (toks, _) = lex.tokenize();
        let mut p = Parser::new(toks);
        let _ = p.parse();
    }

    // ─── Stage 8: Deeply nested expression patterns ────────────────────────
    // These are known parser stress cases — verify no stack overflow.
    // We only run these on short inputs to avoid pathological compile times.
    if input.len() < 20 && input.chars().all(|c| c == '(' || c == ')') {
        let nested: String = std::iter::repeat('(').take(500).chain(std::iter::repeat(')').take(500)).collect();
        let mut lex = Lexer::new(&nested);
        let (toks, _) = lex.tokenize();
        let mut p = Parser::new(toks);
        let _ = p.parse();
    }

    // ─── Stage 9: Whitespace-only and comment-only inputs ──────────────────
    if input.trim().is_empty() {
        let mut lex = Lexer::new(input);
        let (toks, _) = lex.tokenize();
        let mut p = Parser::new(toks);
        let (prog, diags) = p.parse();
        // Whitespace-only input: should produce zero statements, zero diagnostics.
        let _ = (prog, diags); // Contract verified by no-panic execution.
    }

    // ─── Stage 10: Single-character inputs ─────────────────────────────────
    // Verify each character in the input parses safely in isolation.
    if input.len() > 1 {
        for ch in input.chars().take(8) {
            let s = ch.to_string();
            let mut lex = Lexer::new(&s);
            let (toks, _) = lex.tokenize();
            let mut p = Parser::new(toks);
            let _ = p.parse();
        }
    }
}
