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
//! - **Phase 10a (Complete):** Core types + marshaling
//! - **Phase 10b (Complete):** Library loading + extern calls
//! - **Phase 10c (Current):** Callbacks + integration
//!
//! # Safety
//!
//! FFI operations involve `unsafe` code and careful memory management.
//! All unsafe code is isolated in this module with safe wrappers.

pub mod callbacks;
pub mod caller;
pub mod loader;
pub mod marshal;
pub mod safety;
pub mod types;

pub use callbacks::{create_callback, CallbackError, CallbackHandle};
pub use caller::{CallError, ExternFunction};
pub use loader::{LibraryLoader, LoadError};
pub use marshal::{MarshalContext, MarshalError};
pub use safety::{check_null, check_null_mut, BoundedBuffer, SafeCString, SafeMarshalContext};
pub use types::{CType, ExternType};
