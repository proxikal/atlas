//! LSP request and notification handlers
//!
//! This module will contain specialized handlers for different LSP features:
//! - diagnostics.rs: Diagnostic publishing
//! - symbols.rs: Symbol navigation (go-to-definition, find references)
//! - completion.rs: Code completion
//! - hover.rs: Hover information
//! - formatting.rs: Code formatting
//!
//! For now, basic handlers are implemented directly in server.rs.
//! These will be moved here as the LSP implementation grows.
