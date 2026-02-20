//! LSP command - Language Server Protocol server
//!
//! Starts the Atlas LSP server in either stdio mode (default)
//! or TCP mode for editor integration.

use anyhow::Result;
use atlas_lsp::server::AtlasLspServer;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_lsp::{LspService, Server};

/// Arguments for the LSP command
#[derive(Debug, Clone)]
pub struct LspArgs {
    /// Use TCP mode instead of stdio
    pub tcp: bool,
    /// Port for TCP mode (default: 9257)
    pub port: u16,
    /// Bind address for TCP mode
    pub host: String,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for LspArgs {
    fn default() -> Self {
        Self {
            tcp: false,
            port: 9257,
            host: "127.0.0.1".to_string(),
            verbose: false,
        }
    }
}

/// Run the LSP server
pub fn run(args: LspArgs) -> Result<()> {
    // Create tokio runtime
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        if args.tcp {
            run_tcp_server(args).await
        } else {
            run_stdio_server(args).await
        }
    })
}

/// Run LSP server in stdio mode
async fn run_stdio_server(args: LspArgs) -> Result<()> {
    if args.verbose {
        eprintln!("Starting Atlas LSP server (stdio mode)...");
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(AtlasLspServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// Run LSP server in TCP mode
async fn run_tcp_server(args: LspArgs) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    eprintln!(
        "\x1b[32mAtlas LSP server\x1b[0m listening on \x1b[33m{}\x1b[0m",
        addr
    );
    eprintln!("Press Ctrl+C to stop.");

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;

        if args.verbose {
            eprintln!("Client connected from {}", client_addr);
        }

        let (read, write) = tokio::io::split(stream);

        let (service, socket) = LspService::new(AtlasLspServer::new);

        // Spawn handler for this connection
        tokio::spawn(async move {
            Server::new(read, write, socket).serve(service).await;
            if args.verbose {
                eprintln!("Client {} disconnected", client_addr);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_args_default() {
        let args = LspArgs::default();
        assert!(!args.tcp);
        assert_eq!(args.port, 9257);
        assert_eq!(args.host, "127.0.0.1");
        assert!(!args.verbose);
    }

    #[test]
    fn test_lsp_args_tcp_mode() {
        let args = LspArgs {
            tcp: true,
            port: 8080,
            host: "0.0.0.0".to_string(),
            verbose: true,
        };

        assert!(args.tcp);
        assert_eq!(args.port, 8080);
        assert_eq!(args.host, "0.0.0.0");
        assert!(args.verbose);
    }

    #[tokio::test]
    async fn test_socket_addr_parsing() {
        let args = LspArgs::default();
        let addr: Result<SocketAddr, _> = format!("{}:{}", args.host, args.port).parse();
        assert!(addr.is_ok());
        assert_eq!(addr.unwrap().port(), 9257);
    }

    #[tokio::test]
    async fn test_tcp_bind_failure_on_invalid_port() {
        // Port 0 should be valid (OS assigns port)
        let args = LspArgs {
            tcp: true,
            port: 0,
            ..Default::default()
        };

        let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
        let listener = TcpListener::bind(addr).await;
        assert!(listener.is_ok());
    }
}
