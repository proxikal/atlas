# Atlas Test Corpus

The corpus is a collection of real `.atlas` programs that prove the language works.
Each program is readable documentation, survived harness refactors, and verifiable
by anyone who knows the language — just like `rustc`'s `tests/ui/` or Go's `$GOROOT/test/`.

---

## Directory layout

```
tests/corpus/
├── pass/          Programs that must run successfully and produce output
│   ├── arithmetic/
│   ├── strings/
│   ├── functions/
│   ├── types/
│   ├── collections/
│   └── stdlib/
├── fail/          Programs that must produce an error
│   ├── type_errors/
│   ├── syntax_errors/
│   └── runtime_errors/
└── warn/          Programs that succeed but produce compiler warnings
```

Each test case is a pair:
- `foo.atlas`  — the Atlas program
- `foo.stdout` — exact expected output for `pass/` tests
- `foo.stderr` — exact expected error or warning text for `fail/` and `warn/` tests

---

## Adding a pass test

1. Write the `.atlas` program in the appropriate `pass/` subdirectory.
2. Generate the `.stdout` snapshot:
   ```
   UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus
   ```
3. Inspect the generated `.stdout` file to verify it is correct.
4. Run the tests to confirm they pass:
   ```
   cargo nextest run -p atlas-runtime --test corpus
   ```

**Naming convention:** snake_case, descriptive, no `test_` prefix.
**Example:** `pass/arithmetic/fibonacci.atlas`

---

## Adding a fail test

1. Write the `.atlas` program in the appropriate `fail/` subdirectory.
   The program should contain exactly one intentional error.
2. Generate the `.stderr` snapshot:
   ```
   UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus
   ```
3. Inspect the generated `.stderr` file to verify the error message is correct.

---

## Adding a warn test

1. Write the `.atlas` program in `warn/`.
   The program should compile and run successfully but trigger compiler warnings.
2. Generate the `.stderr` snapshot:
   ```
   UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus
   ```
3. Inspect the generated `.stderr` file to verify the warning messages are correct.

---

## Updating expected output

When behavior intentionally changes (bug fix, new output format), regenerate snapshots:

```
UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus
```

Always review the diff in `git diff` before committing updated snapshots to confirm
the change is intentional.

---

## Running the corpus

```bash
# Run all corpus tests
cargo nextest run -p atlas-runtime --test corpus

# Run only pass tests
cargo nextest run -p atlas-runtime --test corpus -E 'test(pass_)'

# Run only fail tests
cargo nextest run -p atlas-runtime --test corpus -E 'test(fail_)'

# Run only warn tests
cargo nextest run -p atlas-runtime --test corpus -E 'test(warn_)'
```

---

## Parity enforcement

Every `pass/` program runs in **both** the Interpreter and the VM (`pass_interpreter` and
`pass_vm` test functions). Both compare against the same `.stdout` snapshot — so if the
engines disagree, the test fails automatically without any extra work.

---

## Rule: new features require a corpus file

Every new language feature MUST add at least one corpus file before merging.
This ensures the feature is tested as a real Atlas program, not just embedded strings
in Rust test code.
