# Atlas Runtime API Evolution Policy

## Goals
- Keep embedding APIs stable and predictable.
- Avoid breaking host applications without a major version bump.

## Breaking Changes
- Renaming or removing public API functions.
- Changing parameter types or return types.
- Changing error/diagnostic semantics exposed by the API.

## Non-Breaking Changes
- Adding new optional functions.
- Adding new error codes without changing existing ones.

## Deprecation Policy
- Deprecations must be documented and announced.
- Minimum deprecation window: 1 minor release.

## Checklist for API Changes
- Update `docs/runtime-api.md` and `docs/runtime-api-evolution.md`.
- Add tests for new behavior.
- Update versioning in `docs/versioning.md` if needed.
