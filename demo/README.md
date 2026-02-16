# Atlas Demo (v0.2 scope)

This `demo/` project exercises the language features that are currently implemented in Atlas (as of 2026-02-16). Each module focuses on a different part of the stdlib or language surface.

## Modules
- `main.atl` — orchestrates all sections
- `strings.atl` — trim/split/join/replace/pad/substring/repeat
- `arrays.atl` — map/filter/reduce/findIndex/some/every/flatMap/sort/slice
- `math_demo.atl` — abs/floor/ceil/sqrt/pow/log/sin/clamp/sign/random
- `options_results.atl` — Option/Result constructors, match expressions, map/map_err/unwrap helpers
- `regex_demo.atl` — regexNew, captures, match indices, replaceAll, split
- `datetime_demo.atl` — now/parse/fromComponents/add/diff/compare/toIso/toTimestamp
- `json_demo.atl` — parseJSON, indexing, jsonAs*, prettify/minify/validate
- `collections.atl` — HashMap/HashSet/Queue/Stack operations
- `io_demo.atl` — readFile/writeFile/appendFile/readDir/pathJoin (requires permissions)
- `reflection.atl` — reflect_typeof, reflect_is_callable, reflect_type_describe, clone, deep_equals, value_to_string

## Running

The module system and file I/O require the runtime API (imports are skipped in the CLI `run` command). Use the bundled helper that grants permissive permissions for the demo:

```bash
cargo run -p atlas-runtime --example run_demo_allow_all
```

This will:
- enable filesystem access for `demo/data/*`
- load `demo/main.atl` with module resolution
- print each demo section to stdout

## Data files
- `demo/data/sample.txt` — sample text for file I/O
- `demo/data/sample.json` — JSON payload for parsing

## Notes
- Namespace imports (`import * as name`) are not implemented yet; only named imports are used.
- `print` accepts only string/number/bool/null; complex values are stringified via `toJSON` or reflection helpers before printing.
- File I/O will fail under the default deny-all security context; the provided runner uses `SecurityContext::allow_all()` for convenience.
