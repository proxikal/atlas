# Phase 10a: FFI Core - Extern Types + Type Marshaling

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Type system from v0.1 for FFI extension.

**Verification Steps:**
1. Verify type system exists:
   ```bash
   ls crates/atlas-runtime/src/types.rs
   grep -n "pub enum Type" crates/atlas-runtime/src/types.rs
   ```
2. Verify value model exists:
   ```bash
   ls crates/atlas-runtime/src/value.rs
   ```
3. Check if extern types already exist:
   ```bash
   grep -n "Extern" crates/atlas-runtime/src/types.rs
   ```

**Expected State:**
- ✅ Type enum exists in `types.rs`
- ✅ Value enum exists in `value.rs`
- ❌ No extern types yet (we're creating them)

**Decision:** Extern types are NEW for v0.2. Design minimal C-compatible types.

---

## Objective

Implement core FFI type system enabling type-safe marshaling between Atlas and C. Adds extern types to Atlas type system and implements bidirectional type conversion (Atlas↔C) without any actual library loading or function calling (those come in phase-10b).

This phase lays the foundation for FFI by establishing the type bridge between Atlas and C.

---

## Files

**Create:** `crates/atlas-runtime/src/ffi/mod.rs` (~200 lines)
**Create:** `crates/atlas-runtime/src/ffi/types.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/ffi/marshal.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/types.rs` (~50 lines - add Extern variant)
**Update:** `crates/atlas-runtime/src/lib.rs` (~5 lines - pub mod ffi)
**Tests:** `crates/atlas-runtime/tests/ffi_types_tests.rs` (~400 lines)

**Total:** ~1355 lines (type system foundation)

---

## Implementation

### 1. Extern Type Variant

Add `Type::Extern` variant to the type system representing C-compatible foreign types.

**File:** `crates/atlas-runtime/src/types.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    // ... existing variants ...

    /// Extern type for FFI (Foreign Function Interface)
    Extern(ExternType),
}

/// C-compatible extern types for FFI
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExternType {
    /// C int (platform-specific, typically i32)
    CInt,
    /// C long (platform-specific, i32 or i64)
    CLong,
    /// C double (f64)
    CDouble,
    /// C char* (null-terminated string pointer)
    CCharPtr,
    /// C void (for void* or void return)
    CVoid,
    /// C bool (u8: 0 or 1)
    CBool,
}
```

Update `Type::display_name()` and `Type::is_assignable_to()` to handle `Extern` variant.

---

### 2. FFI Module Structure

**File:** `crates/atlas-runtime/src/ffi/mod.rs`

```rust
//! Foreign Function Interface (FFI) infrastructure
//!
//! Enables Atlas to interoperate with C code via:
//! - Type marshaling (Atlas ↔ C conversions)
//! - Dynamic library loading (phase-10b)
//! - Extern function calls (phase-10b)
//! - Callbacks from C to Atlas (phase-10c)

pub mod types;
pub mod marshal;

pub use types::ExternType;
pub use marshal::{MarshalContext, MarshalError};
```

---

### 3. FFI Type Definitions

**File:** `crates/atlas-runtime/src/ffi/types.rs`

Define C type representations and conversions:

```rust
use crate::types::{Type, ExternType};
use crate::value::Value;

/// C type representation for FFI boundary
#[derive(Debug, Clone)]
pub enum CType {
    Int(i32),
    Long(i64),
    Double(f64),
    CharPtr(*const i8),  // null-terminated C string
    Void,
    Bool(u8),  // C bool: 0 or 1
}

impl ExternType {
    /// Check if Atlas type can be marshaled to this extern type
    pub fn accepts_atlas_type(&self, atlas_type: &Type) -> bool {
        match (self, atlas_type) {
            (ExternType::CInt, Type::Number) => true,
            (ExternType::CLong, Type::Number) => true,
            (ExternType::CDouble, Type::Number) => true,
            (ExternType::CCharPtr, Type::String) => true,
            (ExternType::CVoid, Type::Void) => true,
            (ExternType::CBool, Type::Bool) => true,
            _ => false,
        }
    }

    /// Get the Atlas type this extern type maps to
    pub fn to_atlas_type(&self) -> Type {
        match self {
            ExternType::CInt | ExternType::CLong | ExternType::CDouble => Type::Number,
            ExternType::CCharPtr => Type::String,
            ExternType::CVoid => Type::Void,
            ExternType::CBool => Type::Bool,
        }
    }
}
```

---

### 4. Type Marshaling

**File:** `crates/atlas-runtime/src/ffi/marshal.rs`

Implement bidirectional Atlas↔C type conversion:

```rust
use crate::value::Value;
use crate::types::ExternType;
use crate::ffi::types::CType;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

#[derive(Debug, Clone)]
pub enum MarshalError {
    TypeMismatch { expected: String, got: String },
    NullPointer,
    InvalidString(String),
    NumberOutOfRange { value: f64, target: String },
}

pub struct MarshalContext {
    /// Track allocated C strings for cleanup
    allocated_strings: Vec<CString>,
}

impl MarshalContext {
    pub fn new() -> Self {
        Self {
            allocated_strings: Vec::new(),
        }
    }

    /// Marshal Atlas value to C type
    pub fn atlas_to_c(&mut self, value: &Value, target: &ExternType) -> Result<CType, MarshalError> {
        match (value, target) {
            (Value::Number(n), ExternType::CInt) => {
                if *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                    Ok(CType::Int(*n as i32))
                } else {
                    Err(MarshalError::NumberOutOfRange {
                        value: *n,
                        target: "c_int".to_string(),
                    })
                }
            }

            (Value::Number(n), ExternType::CLong) => {
                Ok(CType::Long(*n as i64))
            }

            (Value::Number(n), ExternType::CDouble) => {
                Ok(CType::Double(*n))
            }

            (Value::String(s), ExternType::CCharPtr) => {
                let c_string = CString::new(s.as_str())
                    .map_err(|e| MarshalError::InvalidString(e.to_string()))?;
                let ptr = c_string.as_ptr();
                self.allocated_strings.push(c_string);
                Ok(CType::CharPtr(ptr))
            }

            (Value::Bool(b), ExternType::CBool) => {
                Ok(CType::Bool(if *b { 1 } else { 0 }))
            }

            (Value::Void, ExternType::CVoid) => {
                Ok(CType::Void)
            }

            _ => Err(MarshalError::TypeMismatch {
                expected: format!("{:?}", target),
                got: format!("{:?}", value),
            }),
        }
    }

    /// Marshal C type to Atlas value
    pub fn c_to_atlas(&self, c_value: &CType) -> Result<Value, MarshalError> {
        match c_value {
            CType::Int(i) => Ok(Value::Number(*i as f64)),
            CType::Long(l) => Ok(Value::Number(*l as f64)),
            CType::Double(d) => Ok(Value::Number(*d)),

            CType::CharPtr(ptr) => {
                if ptr.is_null() {
                    return Err(MarshalError::NullPointer);
                }
                unsafe {
                    let c_str = CStr::from_ptr(*ptr);
                    let s = c_str.to_str()
                        .map_err(|e| MarshalError::InvalidString(e.to_string()))?;
                    Ok(Value::String(s.to_string()))
                }
            }

            CType::Bool(b) => Ok(Value::Bool(*b != 0)),
            CType::Void => Ok(Value::Void),
        }
    }
}

impl Drop for MarshalContext {
    fn drop(&mut self) {
        // CStrings automatically cleaned up
        self.allocated_strings.clear();
    }
}
```

---

## Tests (TDD - Use rstest)

**File:** `crates/atlas-runtime/tests/ffi_types_tests.rs`

### Extern Type Tests (8 tests)
1. `test_extern_type_accepts_atlas_type_valid` - CInt accepts Number
2. `test_extern_type_accepts_atlas_type_invalid` - CInt rejects String
3. `test_extern_type_to_atlas_type_mapping` - All extern types map correctly
4. `test_extern_type_display_names` - Display names correct
5. `test_type_enum_extern_variant` - Type::Extern serialization
6. `test_type_assignability_with_extern` - Type compatibility checks
7. `test_extern_type_equality` - ExternType comparison
8. `test_all_extern_types_exist` - All 6 extern types defined

### Atlas→C Marshaling Tests (10 tests)
1. `test_marshal_number_to_cint` - 42.0 → CInt(42)
2. `test_marshal_number_to_clong` - 1000.0 → CLong(1000)
3. `test_marshal_number_to_cdouble` - 3.14 → CDouble(3.14)
4. `test_marshal_string_to_char_ptr` - "hello" → CCharPtr
5. `test_marshal_bool_to_cbool_true` - true → CBool(1)
6. `test_marshal_bool_to_cbool_false` - false → CBool(0)
7. `test_marshal_void_to_cvoid` - Void → CVoid
8. `test_marshal_type_mismatch` - String to CInt → Error
9. `test_marshal_number_out_of_range` - 1e100 to CInt → Error
10. `test_marshal_string_with_null_byte` - "hello\0world" → Error

### C→Atlas Marshaling Tests (7 tests)
1. `test_unmarshal_cint_to_number` - CInt(42) → 42.0
2. `test_unmarshal_clong_to_number` - CLong(1000) → 1000.0
3. `test_unmarshal_cdouble_to_number` - CDouble(3.14) → 3.14
4. `test_unmarshal_char_ptr_to_string` - CCharPtr → "hello"
5. `test_unmarshal_cbool_to_bool` - CBool(1) → true, CBool(0) → false
6. `test_unmarshal_cvoid_to_void` - CVoid → Void
7. `test_unmarshal_null_pointer` - null CCharPtr → Error

### MarshalContext Tests (5 tests)
1. `test_marshal_context_tracks_strings` - Verify string tracking
2. `test_marshal_context_cleanup` - Drop cleans up allocations
3. `test_marshal_context_multiple_strings` - Multiple string conversions
4. `test_marshal_context_reuse` - Context can be reused
5. `test_marshal_context_concurrent_conversions` - Multiple conversions work

**Minimum test count:** 30 tests (all must pass)

---

## Integration Points

- **Uses:** Type system from `types.rs`
- **Uses:** Value model from `value.rs`
- **Creates:** FFI type system foundation
- **Creates:** Type marshaling infrastructure
- **Blocks:** Phase-10b (library loading) depends on this
- **Output:** C-compatible type bridge

---

## Acceptance Criteria

- [ ] `Type::Extern` variant added to type system
- [ ] All 6 extern types defined (CInt, CLong, CDouble, CCharPtr, CVoid, CBool)
- [ ] `ExternType::accepts_atlas_type()` validates conversions
- [ ] `ExternType::to_atlas_type()` maps extern types to Atlas types
- [ ] `MarshalContext::atlas_to_c()` converts all Atlas→C types
- [ ] `MarshalContext::c_to_atlas()` converts all C→Atlas types
- [ ] Type mismatch errors handled correctly
- [ ] Number range validation for CInt
- [ ] Null pointer detection for CCharPtr
- [ ] String allocation tracking and cleanup
- [ ] 30+ tests pass
- [ ] No clippy warnings
- [ ] `cargo fmt` applied
- [ ] `cargo test -p atlas-runtime test_ffi -- --exact` passes
- [ ] Type system remains backward compatible

---

## Notes

**Memory Safety:**
- All `unsafe` code isolated in `marshal.rs`
- CString allocation tracked in MarshalContext
- Automatic cleanup on Drop

**Platform Compatibility:**
- CInt/CLong use platform-specific sizes via std::os::raw
- Tests run on all platforms (Linux/macOS/Windows)

**Design Decisions:**
- Minimal C type set (6 types) sufficient for v0.2
- No struct marshaling yet (future enhancement)
- No array marshaling yet (phase-10b will add)
- Explicit error types over panics

**Next Phase:**
Phase-10b will add library loading, extern function declarations, and actual function calling using this marshaling foundation.
