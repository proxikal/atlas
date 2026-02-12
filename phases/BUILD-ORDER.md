# Atlas Build Order

This is the canonical build order for AI agents. Execute phases in order. Do not skip ahead without completing exit criteria.

## 0. Research
1. `phases/research/phase-01-references.md`
2. `phases/research/phase-02-constraints.md`
3. `phases/research/phase-05-language-comparison.md`
4. `phases/research/phase-03-module-scaffolding.md`
5. `phases/research/phase-04-module-compiler-hooks.md`
6. Review `docs/decision-log.md`, `docs/coverage-matrix.md`, `docs/phase-gates.md` before starting implementation.

## 1. Foundation
1. `phases/foundation/phase-01-overview.md`
2. `phases/foundation/phase-02-workspace-layout.md`
3. `phases/foundation/phase-03-tooling-baseline.md`
4. `phases/foundation/phase-04-dependency-lock.md`
5. `phases/foundation/phase-05-ci-baseline.md`
6. `phases/foundation/phase-06-contributing.md`
7. `phases/foundation/phase-07-project-metadata.md`
8. `phases/foundation/phase-08-release-packaging-plan.md`
9. `phases/foundation/phase-09-runtime-api-scaffold.md`
10. `phases/foundation/phase-10-runtime-api-tests.md`
11. `phases/foundation/phase-11-runtime-api-evolution.md`

## 2. Diagnostics Core
1. `phases/typing/phase-03-diagnostics-pipeline.md`
2. `phases/typing/phase-04-diagnostic-normalization.md`
3. `phases/typing/phase-08-diagnostics-versioning.md`
4. `phases/typing/phase-09-diagnostics-snapshots.md`

## 3. Frontend
1. `phases/frontend/phase-03-ast-build.md`
2. `phases/frontend/phase-01-lexer.md`
3. `phases/frontend/phase-02-parser.md`
4. `phases/frontend/phase-04-parser-errors.md`
5. `phases/frontend/phase-05-grammar-conformance.md`
6. `phases/frontend/phase-06-parser-recovery-strategy.md`
7. `phases/frontend/phase-07-lexer-edge-cases.md`
8. `phases/frontend/phase-08-ast-dump-versioning.md`
9. `phases/frontend/phase-09-keyword-policy-tests.md`
10. `phases/frontend/phase-10-keyword-enforcement.md`

## 4. Typing & Binding
1. `phases/typing/phase-01-binder.md`
2. `phases/typing/phase-02-typechecker.md`
3. `phases/typing/phase-06-scopes-shadowing.md`
4. `phases/typing/phase-07-nullability.md`
5. `phases/typing/phase-10-function-returns.md`
6. `phases/typing/phase-14-warnings.md`
7. `phases/typing/phase-13-diagnostics.md`
8. `phases/typing/phase-18-semantic-edge-cases.md`
9. `phases/typing/phase-11-typecheck-stability.md`

## 5. Runtime Values
1. `phases/interpreter/phase-03-runtime-values.md`
2. `phases/interpreter/phase-07-value-model-tests.md`

## 6. Interpreter
1. `phases/interpreter/phase-01-interpreter-core.md`
2. `phases/interpreter/phase-04-arrays-mutation.md`
3. `phases/interpreter/phase-05-function-calls.md`
4. `phases/interpreter/phase-06-control-flow.md`
5. `phases/interpreter/phase-08-runtime-errors.md`
6. `phases/interpreter/phase-09-array-aliasing-tests.md`
7. `phases/interpreter/phase-10-numeric-semantics.md`
8. `phases/interpreter/phase-11-repl-state-tests.md`

## 7. REPL
1. `phases/interpreter/phase-02-repl.md`

## 8. Bytecode & VM
1. `phases/bytecode-vm/phase-03-bytecode-format.md`
2. `phases/bytecode-vm/phase-01-bytecode-compiler.md`
3. `phases/bytecode-vm/phase-02-vm.md`
4. `phases/bytecode-vm/phase-06-constants-pool.md`
5. `phases/bytecode-vm/phase-07-stack-frames.md`
6. `phases/bytecode-vm/phase-08-branching.md`
7. `phases/bytecode-vm/phase-09-vm-errors.md`
8. `phases/bytecode-vm/phase-10-bytecode-serialization.md`
9. `phases/bytecode-vm/phase-11-bytecode-versioning.md`
10. `phases/bytecode-vm/phase-04-disassembler.md`
11. `phases/bytecode-vm/phase-05-optimizer-hooks.md`
12. `phases/bytecode-vm/phase-12-profiling-hooks.md`
13. `phases/bytecode-vm/phase-13-debugger-hooks.md`
14. `phases/bytecode-vm/phase-14-debug-info.md`
15. `phases/bytecode-vm/phase-15-debug-info-defaults.md`
16. `phases/bytecode-vm/phase-16-bytecode-format-tests.md`
17. `phases/bytecode-vm/phase-17-runtime-numeric-errors.md`

## 9. Standard Library
1. `phases/stdlib/phase-01-stdlib.md`
2. `phases/stdlib/phase-02-stdlib-tests.md`
3. `phases/stdlib/phase-03-stdlib-doc-sync.md`
4. `phases/stdlib/phase-04-stdlib-expansion-plan.md`
5. `phases/stdlib/phase-05-io-security-model.md`
6. `phases/stdlib/phase-06-json-stdlib-plan.md`
7. `phases/stdlib/phase-07-prelude-binding.md`
8. `phases/stdlib/phase-08-prelude-tests.md`

## 10. CLI
1. `phases/cli/phase-01-cli.md`
2. `phases/cli/phase-02-cli-diagnostics.md`
3. `phases/cli/phase-03-repl-modes.md`
4. `phases/cli/phase-04-build-output.md`
5. `phases/cli/phase-05-repl-history.md`
6. `phases/cli/phase-06-config-behavior.md`
7. `phases/cli/phase-07-ast-typecheck-dumps.md`
8. `phases/cli/phase-08-ast-typecheck-tests.md`
9. `phases/cli/phase-09-json-dump-stability-tests.md`
10. `phases/cli/phase-10-cli-e2e-tests.md`

## 11. LSP & Tooling
1. `phases/lsp/phase-01-lsp-foundation.md`
2. `phases/lsp/phase-02-lsp-diagnostics.md`
3. `phases/lsp/phase-03-lsp-navigation.md`
4. `phases/lsp/phase-04-lsp-completion.md`
5. `phases/lsp/phase-05-lsp-formatting.md`
6. `phases/lsp/phase-06-lsp-testing.md`

## 12. Polish
1. `phases/polish/phase-01-polish.md`
2. `phases/polish/phase-02-regression-suite.md`
3. `phases/polish/phase-03-docs-pass.md`
4. `phases/polish/phase-04-stability-audit.md`
5. `phases/polish/phase-05-release-checklist.md`
6. `phases/polish/phase-06-cross-platform-check.md`
7. `phases/polish/phase-07-interpreter-vm-parity-tests.md`
