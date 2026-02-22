# Atlas Module System Test Plan (v1.0)

## Goals
Validate module resolution, imports, exports, and error handling.

## Test Categories
- Import resolution (relative paths)
- Alias imports (`import math as m`)
- Missing module errors
- Circular dependency detection
- Export visibility across modules

## Examples
- `tests/modules/basic_import/`:
  - `main.atl` imports `math.atl`
- `tests/modules/cycle/`:
  - `a.atl` imports `b.atl`, `b.atl` imports `a.atl`
- `tests/modules/missing/`:
  - import non-existent module

## Expected Outputs
- Diagnostics for missing or circular imports.
- Successful runs for valid imports.
