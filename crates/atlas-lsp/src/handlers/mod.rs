//! LSP request and notification handlers
//!
//! This module organizes specialized handlers for LSP features:
//!
//! ## Document Sync
//! - `did_open`, `did_change`, `did_close` in server.rs
//!
//! ## Navigation
//! - `document_symbol`, `goto_definition`, `references` in navigation.rs
//!
//! ## Code Intelligence
//! - `hover` in hover.rs - Type info, documentation, builtin help
//! - `completion` in completion.rs - Code completions
//! - `code_action` in actions.rs - Quick fixes and refactorings
//!
//! ## Syntax
//! - `semantic_tokens_full`, `semantic_tokens_range` in semantic_tokens.rs
//! - `formatting`, `range_formatting` in formatting.rs
//!
//! ## Diagnostics
//! - Diagnostic publishing in server.rs via convert.rs
