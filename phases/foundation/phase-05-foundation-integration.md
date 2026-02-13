# Phase 05: Foundation Integration & Testing

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** All previous foundation phases must be complete.

**Verification:**
```bash
ls crates/atlas-runtime/src/api/runtime.rs
ls examples/embedding/
ls .github/workflows/ci.yml
ls crates/atlas-config/src/config.rs
cargo test api_tests
cargo test config_tests
cargo run --example 01_hello_world
grep "name: CI" .github/workflows/ci.yml
```

**What's needed:**
- Phase 01: Runtime embedding API complete
- Phase 02: Custom functions and examples working
- Phase 03: CI/CD workflows configured
- Phase 04: Configuration system functional

**If missing:** Complete phases foundation/phase-01 through phase-04 first

---

## Objective
Comprehensive integration testing of all foundation features verifying they work together correctly embedding API with configuration, custom functions with sandboxing, CI automation with complete test coverage, establishing Foundation as production-ready infrastructure.

## Files
**Create:** `crates/atlas-runtime/tests/foundation_integration_tests.rs` (~600 lines)
**Create:** `crates/atlas-runtime/tests/embedding_scenarios_tests.rs` (~500 lines)
**Create:** `tests/foundation_e2e_tests.rs` (~400 lines workspace-level)
**Create:** `docs/foundation-status.md` (~300 lines)
**Update:** `STATUS.md` (~50 lines mark foundation complete)

## Dependencies
- All foundation phases 01-04 complete
- All individual unit tests passing
- Examples compiling and running
- CI workflows configured

## Implementation

### Cross-Feature Integration Testing
Test scenarios combining multiple foundation features. Test runtime with configuration verifying config affects execution. Test custom functions with sandboxing ensuring sandbox restrictions work. Test full embedding pipeline loading config, creating runtime, registering natives, setting globals, executing code, getting results. Test multiple runtimes with different configurations ensuring isolation. Test configuration affects compilation with different optimization levels. Test error recovery across features ensuring runtime continues after parse, compile, and native function errors. Test persistent state across eval calls with configuration. Test large-scale integration with multiple custom functions and complex computations.

### Embedding Scenario Testing
Create real-world embedding scenarios. Game scripting scenario with sandboxed runtime and game API functions spawn enemy, get player health. Configuration processing scenario loading JSON config, transforming with Atlas, extracting results. Data transformation pipeline scenario converting Rust data to Atlas, filtering and mapping in Atlas code, extracting filtered results. Plugin system scenario with sandboxed runtime, host-provided API, untrusted plugin code execution. Expression evaluator scenario with context variables and user-provided expressions.

### End-to-End Workflow Testing
Test complete embedding workflow from config loading to result extraction. Verify all embedding examples compile successfully. Validate CI workflow files are valid YAML with required sections. Test workspace-level integration ensuring all crates work together.

### Foundation Status Documentation
Write comprehensive foundation status report. Document implementation status of all five phases with checkboxes. List verification checklist with testing coverage and code quality metrics. Declare API stability for v0.2 with promise of no breaking changes in minor versions. Document performance benchmarks runtime creation, eval timing, value conversion, config loading. List known limitations sandboxing, configuration merge granularity, arity checking. Propose future enhancements for v0.3 and beyond. Conclude Foundation is complete and production-ready.

### STATUS.md Update
Update STATUS.md marking Foundation category as 5/5 complete with all phases checked off. Update overall progress percentage. Add completion timestamp.

## Tests (TDD - Use rstest)

**Cross-feature integration tests:**
1. Runtime with configuration
2. Custom functions with sandboxing
3. Full embedding pipeline all steps
4. Multiple isolated runtimes
5. Configuration affects compilation
6. Error recovery parse compile runtime
7. Persistent state across evals
8. Large-scale integration

**Embedding scenario tests:**
1. Game scripting scenario
2. Configuration processing scenario
3. Data transformation pipeline scenario
4. Plugin system scenario
5. Expression evaluator scenario

**End-to-end tests:**
1. Full embedding workflow
2. All examples compile
3. CI workflows valid YAML

**Minimum test count:** 100 integration tests

## Integration Points
- Uses: All foundation phases 01-04
- Tests: Runtime API custom functions config CI
- Verifies: Cross-feature integration
- Validates: Real-world embedding scenarios
- Updates: STATUS.md and foundation-status.md
- Output: Production-ready foundation infrastructure

## Acceptance
- All 100+ integration tests pass
- 5 embedding scenarios work correctly
- All 6 examples compile and run without errors
- API and Config work together seamlessly
- Custom functions work in sandboxed environments
- Multiple runtimes coexist with different configs
- Error recovery works across all features
- CI workflows validated YAML structure correct
- Documentation complete foundation-status.md
- STATUS.md updated Foundation marked 5/5 complete
- Zero clippy warnings across foundation crates
- Code coverage above 80% for foundation modules
- Performance benchmarks meet targets
- All embedding examples verified
- Foundation is production-ready for v0.2
