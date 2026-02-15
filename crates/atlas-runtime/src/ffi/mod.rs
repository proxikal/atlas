//! Foreign Function Interface (FFI) infrastructure
//!
//! Enables Atlas to interoperate with C code via:
//! - Type marshaling (Atlas ↔ C conversions)
//! - Dynamic library loading (phase-10b)
//! - Extern function calls (phase-10b)
//! - Callbacks from C to Atlas (phase-10c)
//!
//! # Phase Status
//!
//! - **Phase 10a (Current):** Core types + marshaling
//! - **Phase 10b (Next):** Library loading + extern calls
//! - **Phase 10c (Future):** Callbacks + integration
//!
//! # Safety
//!
//! FFI operations involve `unsafe` code and careful memory management.
//! All unsafe code is isolated in this module with safe wrappers.

pub mod marshal;
pub mod types;

pub use marshal::{MarshalContext, MarshalError};
pub use types::{CType, ExternType};
