//! Test runner infrastructure for Atlas
//!
//! Provides test discovery, execution, and reporting following
//! the Rust (cargo test) and Go (go test) model.

pub mod discovery;
pub mod reporter;
pub mod runner;

pub use discovery::TestSuite;
pub use reporter::TestReporter;
pub use runner::TestRunner;
