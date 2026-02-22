//! Document state management

use atlas_runtime::{
    ast::Program, symbol::SymbolTable, Binder, Diagnostic, Lexer, Parser, TypeChecker,
};
use tower_lsp::lsp_types::Url;

/// State of a single document in the LSP server
pub struct DocumentState {
    pub uri: Url,
    pub text: String,
    pub version: i32,
    pub diagnostics: Vec<Diagnostic>,
    pub ast: Option<Program>,
    pub symbols: Option<SymbolTable>,
}

impl DocumentState {
    /// Create a new document and analyze it
    pub fn new(uri: Url, text: String, version: i32) -> Self {
        let mut doc = Self {
            uri,
            text,
            version,
            diagnostics: Vec::new(),
            ast: None,
            symbols: None,
        };
        doc.analyze();
        doc
    }

    /// Update document text and re-analyze
    pub fn update(&mut self, text: String, version: i32) {
        self.text = text;
        self.version = version;
        self.analyze();
    }

    /// Analyze the document and update diagnostics
    fn analyze(&mut self) {
        self.diagnostics.clear();
        self.ast = None;
        self.symbols = None;

        // Lex the source code
        let mut lexer = Lexer::new(&self.text);
        let (tokens, lex_diagnostics) = lexer.tokenize();

        if !lex_diagnostics.is_empty() {
            self.diagnostics.extend(lex_diagnostics);
            return;
        }

        // Parse tokens into AST
        let mut parser = Parser::new(tokens);
        let (ast, parse_diagnostics) = parser.parse();

        // Store partial AST even on parse errors — enables LSP features (hover, completion)
        // for the portion of the document that parsed successfully.
        self.ast = Some(ast.clone());

        if !parse_diagnostics.is_empty() {
            self.diagnostics.extend(parse_diagnostics);
            return;
        }

        // Bind symbols
        let mut binder = Binder::new();
        let (mut symbol_table, bind_diagnostics) = binder.bind(&ast);

        if !bind_diagnostics.is_empty() {
            self.diagnostics.extend(bind_diagnostics);
            return;
        }

        // Type check
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let typecheck_diagnostics = typechecker.check(&ast);

        // Store symbols for navigation (after type checking updates them)
        self.symbols = Some(symbol_table.clone());

        if !typecheck_diagnostics.is_empty() {
            self.diagnostics.extend(typecheck_diagnostics);
        }
    }
}
