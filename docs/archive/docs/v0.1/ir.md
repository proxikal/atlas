# Atlas IR (v0.1 Decision)

## Status
- v0.1 compiles AST directly to bytecode.
- No separate typed IR in v0.1.

## Rationale
- Keeps compiler smaller and easier to implement.
- Maintains spans from AST to diagnostics.

## Future
- A typed IR may be introduced in v1.1+ for optimization.
