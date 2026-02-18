# Phase Infra-05b: File-Based Test Corpus

## Blocker
**REQUIRED:** Infra-05a complete. `RuntimeConfig::with_output()` exists and works.

---

## Objective

Build the file-based test corpus: real `.atlas` programs that prove the language
works, written in Atlas itself rather than embedded in Rust strings. This is how
every serious compiler tests — rustc has `tests/ui/`, Go has `$GOROOT/test/`,
clang has `llvm-lit`. When a feature is expressed as a real `.atlas` file it
becomes readable documentation, survives harness refactors, and is verifiable by
anyone who knows the language.

The corpus harness runs each `.atlas` file through the Atlas runtime using the
`OutputWriter` from Infra-05a to capture `print()` output, then compares it
against a companion `.stdout` file (for pass tests) or `.stderr` file (for fail
and warn tests).

---

## Files Created

```
crates/atlas-runtime/tests/corpus.rs          # Rust harness (~200 lines)
crates/atlas-runtime/tests/corpus/
├── README.md                                 # How to add tests, naming conventions
├── pass/                                     # Programs that should run and produce output
│   ├── arithmetic/
│   ├── strings/
│   ├── functions/
│   ├── types/
│   ├── collections/
│   └── stdlib/
├── fail/                                     # Programs that should error
│   ├── type_errors/
│   ├── syntax_errors/
│   └── runtime_errors/
└── warn/                                     # Programs that produce warnings
```

Each test case is a pair:
- `foo.atlas` — the Atlas program
- `foo.stdout` — exact expected output (for pass/), OR
- `foo.stderr` — exact expected error message (for fail/, warn/)

---

## Implementation

### Step 1: Write tests/corpus.rs (the harness)

The harness discovers all `.atlas` files at test time using `std::fs::read_dir`.
It uses rstest to generate one named test per file so nextest shows individual
names (e.g., `corpus::pass::arithmetic::fibonacci`).

**Key design:**
- Uses `RuntimeConfig::with_output(buf.clone())` from Infra-05a — no subprocess,
  no stdout redirect tricks, no return-value comparison
- Every `pass/` file runs in BOTH interpreter and VM — parity enforced automatically
- If `UPDATE_CORPUS=1` env var is set, writes actual output to `.stdout`/`.stderr`
  instead of asserting — this is the workflow for generating new snapshots
- Missing companion file = test fails with clear message: create `foo.stdout`

```rust
// Pseudocode structure (implement fully):
fn run_corpus_file(path: &Path, mode: ExecutionMode) -> String {
    let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    // Wrap buf as OutputWriter (use std::io::Cursor or thin newtype)
    let output: OutputWriter = Arc::new(Mutex::new(Box::new(CaptureWriter(buf.clone()))));
    let config = RuntimeConfig::new()
        .with_output(output)
        .with_io_allowed(false)   // corpus tests are pure computation
        .with_network_allowed(false);
    let runtime = Runtime::with_config(mode, config);
    let source = std::fs::read_to_string(path).unwrap();
    runtime.eval(&source).ok(); // eval may fail for fail/ tests
    String::from_utf8(buf.lock().unwrap().clone()).unwrap()
}
```

For `fail/` tests: capture the `EvalError` message (formatted) into the buffer
or compare against `.stderr` using the error's `Display` impl. The harness must
capture both stdout AND the error message.

### Step 2: Pass corpus — 30+ files

Stick to **proven-working** features only. Do not write corpus files for features
with known failures in the existing test suite.

**`pass/arithmetic/`** (5 files minimum):
- `basic_ops.atlas` — `+`, `-`, `*`, `/`, `%`, operator precedence
- `comparisons.atlas` — `<`, `>`, `<=`, `>=`, `==`, `!=`, truthiness
- `fibonacci.atlas` — recursive function, demonstrates recursion works
- `factorial.atlas` — another recursion pattern
- `number_formatting.atlas` — integer vs float display

**`pass/strings/`** (5 files minimum):
- `concatenation.atlas` — string `+` and `len()`
- `string_functions.atlas` — `split`, `trim`, `to_upper`, `to_lower`, `contains`
- `string_conversion.atlas` — `str(42)`, `str(true)`, `str(null)`
- `multiline.atlas` — multi-statement programs
- `unicode_basics.atlas` — basic unicode string handling

**`pass/functions/`** (5 files minimum):
- `first_class.atlas` — functions as values, passing to other functions
- `generics.atlas` — generic function with type parameter
- `recursion.atlas` — mutual recursion
- `higher_order.atlas` — map/filter patterns with function args
- `default_return.atlas` — implicit null return

**`pass/types/`** (5 files minimum):
- `option_some_none.atlas` — `Option<T>`, `Some(x)`, `None`, pattern match
- `result_ok_err.atlas` — `Result<T, E>`, `Ok(x)`, `Err(e)`, pattern match
- `union_types.atlas` — union type annotation and type guard
- `type_guards.atlas` — `is` keyword, narrowing
- `pattern_matching.atlas` — `match` expression with multiple arms

**`pass/collections/`** (5 files minimum):
- `array_ops.atlas` — create, index, `len()`, `push`, `pop`
- `hashmap_ops.atlas` — `HashMap.new()`, insert, get, contains_key
- `hashset_ops.atlas` — `HashSet.new()`, insert, contains
- `queue_ops.atlas` — `Queue.new()`, enqueue, dequeue
- `stack_ops.atlas` — `Stack.new()`, push, pop, peek

**`pass/stdlib/`** (5 files minimum):
- `math_functions.atlas` — `floor`, `ceil`, `sqrt`, `abs`, `pow`, `min`, `max`
- `string_stdlib.atlas` — stdlib string functions (replace, starts_with, ends_with)
- `array_stdlib.atlas` — sort, reverse, join, filter, map
- `type_checks.atlas` — `typeof`, `is_null`, conversion functions
- `json_basic.atlas` — `JSON.parse`, `JSON.stringify` for simple values

### Step 3: Fail corpus — 15+ files

Each program has exactly one intentional error. The `.stderr` companion contains
the exact error code and message format Atlas produces.

**`fail/type_errors/`** (5 files):
- `wrong_arg_type.atlas` — pass string where number expected
- `wrong_arg_count.atlas` — call function with wrong arity
- `undefined_variable.atlas` — use variable before declaration
- `type_annotation_mismatch.atlas` — `let x: int = "hello"`
- `operation_on_null.atlas` — arithmetic on null value

**`fail/syntax_errors/`** (5 files):
- `missing_closing_brace.atlas` — unclosed `{`
- `invalid_token.atlas` — `@` character
- `missing_semicolon.atlas` — statement without `;` where required
- `bad_string_escape.atlas` — `"\q"` invalid escape
- `unexpected_eof.atlas` — file truncated mid-expression

**`fail/runtime_errors/`** (5 files):
- `division_by_zero.atlas` — `10 / 0`
- `index_out_of_bounds.atlas` — `[][0]`
- `null_access.atlas` — calling method on null
- `stack_overflow.atlas` — infinite recursion
- `type_error_at_runtime.atlas` — runtime type mismatch

### Step 4: Warn corpus — 5+ files

```
warn/unused_variable.atlas
warn/unused_import.atlas
warn/variable_shadowing.atlas
warn/unreachable_code.atlas
warn/redundant_type_annotation.atlas
```

### Step 5: README.md

`tests/corpus/README.md` must explain:
1. How to add a pass test: write `.atlas` + run `UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus` to generate `.stdout`
2. How to add a fail test: write `.atlas` + create `.stderr` with expected error
3. Naming convention: snake_case, descriptive, no `test_` prefix
4. How to update expected output when behavior intentionally changes (re-run with `UPDATE_CORPUS=1`)
5. Rule: every new language feature MUST add at least one corpus file before merge

---

## Acceptance

- `cargo nextest run -p atlas-runtime --test corpus` runs and all pass
- 30+ pass corpus files, each with `.stdout` companion
- 15+ fail corpus files, each with `.stderr` companion
- 5+ warn corpus files, each with `.stderr` companion
- Every `pass/` file runs in both interpreter AND VM (parity auto-enforced)
- `UPDATE_CORPUS=1` causes harness to write snapshots instead of asserting
- `tests/corpus/README.md` complete
- `cargo nextest run -p atlas-runtime` full suite green (corpus included)
- Zero clippy warnings
- Commit: `feat(tests): Add file-based test corpus (50+ .atlas programs)`
