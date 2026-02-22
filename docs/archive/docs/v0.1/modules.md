# Atlas Module System (Design Sketch)

## Status
- Not implemented in v0.1.
- Defined now to avoid future refactors.

## Goals
- Simple, explicit imports.
- File-based modules.
- No implicit global namespace.

## Design
- Each `.atl` file is a module.
- Module name is the file path without extension.
- Imports are explicit:
  - `import math` (module name)
  - `import math as m`

## Resolution
- Imports resolve relative to the current file directory.
- No package registry in v1.0; only local modules.

## Exports
- All top-level `fn` and `let/var` are exported by default.
- Later versions may allow `export` keyword.

## Cycles
- Circular imports are compile-time errors.
