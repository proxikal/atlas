//! Atlas Language Server Protocol (LSP) server
//!
//! Provides real-time diagnostics, navigation, and code intelligence
//! for Atlas source files in editors like VSCode, Neovim, and Zed.

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Set up LSP service
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));

    // Start the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
