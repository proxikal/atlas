# DR-002: Raw Pointer Threading for SecurityContext

**Date:** 2024-02-05
**Status:** Accepted
**Component:** Runtime Security

## Context
I/O operations need permission checks, but threading `&SecurityContext` through all runtime calls creates lifetime complexity.

## Decision
`SecurityContext` threaded through runtime via raw pointer:
- `Interpreter`/`VM` store `current_security: Option<*const SecurityContext>`
- Set during `eval()`/`run()` calls
- Stdlib functions accept `&SecurityContext` parameter
- Access via unsafe dereference in builtin calls

## Rationale
**Simplicity:** Avoids lifetime complexity while maintaining security checks.

**Safety:** SecurityContext lifetime guaranteed valid for duration of `eval()`/`run()` execution - pointer is always valid when dereferenced.

**Performance:** Zero overhead compared to reference passing.

## Alternatives Considered
- **Thread `&SecurityContext` through all calls:** Rejected - lifetime annotations explode, makes codebase unmaintainable
- **Global static SecurityContext:** Rejected - not thread-safe, prevents multiple concurrent executions
- **Arc<SecurityContext> clone everywhere:** Rejected - unnecessary allocation overhead

## Consequences
- ✅ **Benefits:** Simple, clean API for stdlib functions
- ✅ **Benefits:** Zero runtime overhead
- ✅ **Benefits:** Safe in practice (controlled lifetime scope)
- ⚠️  **Trade-offs:** Uses `unsafe` block (but with clear safety invariant)
- ❌ **Costs:** Requires careful documentation of safety contract

## Implementation Notes
Pattern:
```rust
// In Interpreter/VM
current_security: Option<*const SecurityContext>

// Set during eval/run
self.current_security = Some(security_ctx as *const _);

// Access in stdlib
let ctx = unsafe { &*self.current_security.unwrap() };
call_builtin(ctx, ...);
```

**Safety invariant:** SecurityContext pointer valid for entire `eval()`/`run()` scope.

## References
- Spec: `docs/reference/io-security-model.md`
- Related: DR-001 (Value Representation)
