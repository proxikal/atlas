# Phase 03: LSP Integration Tests & Editor Verification

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** All previous LSP phases must be complete.

**Verification:**
```bash
ls crates/atlas-lsp/src/hover.rs
ls crates/atlas-lsp/src/symbols.rs
cargo nextest run -p atlas-lsp
atlas lsp --help
```

**What's needed:**
- LSP phases 01-02 complete with all features (3 more phases remain after this: 04-05)
- All LSP unit tests passing
- LSP server launches successfully
- CLI integration from cli/phase-03

**If missing:** Complete phases lsp/phase-01 and lsp/phase-02 first

---

## Objective
Comprehensive integration testing of all LSP features ensuring they work together correctly plus real editor integration verification and setup documentation establishing LSP as production-ready for v0.2.

## Files
**Create:** `crates/atlas-lsp/tests/lsp_integration_tests.rs` (~800 lines)
**Create:** `crates/atlas-lsp/tests/lsp_performance_tests.rs` (~300 lines)
**Create:** `crates/atlas-lsp/tests/lsp_protocol_tests.rs` (~400 lines)
**Create:** `docs/editor-setup/vscode.md` (~400 lines)
**Create:** `docs/editor-setup/neovim.md` (~400 lines)
**Create:** `docs/editor-setup/emacs.md` (~300 lines)
**Create:** `docs/lsp-features.md` (~600 lines)
**Create:** `docs/lsp-troubleshooting.md` (~400 lines)
**Create:** `docs/lsp-status.md` (~300 lines)
**Update:** `STATUS.md` (~50 lines mark LSP complete)

## Dependencies
- All LSP phases 01-02 complete
- All LSP features implemented
- CLI LSP launcher from cli/phase-03
- Real editors for integration testing

## Implementation

### LSP Feature Integration Testing
Test all LSP features working together. Test hover with semantic tokens ensuring consistency. Test code actions with diagnostics verifying fixes. Test symbols with folding ensuring structure alignment. Test inlay hints with hover checking type consistency. Test workspace symbols with document symbols checking index consistency. Verify features don't interfere with each other. Test feature combinations used in real workflows.

### Multi-Feature Workflows
Test realistic editor workflows using multiple features. Test navigation workflow symbols hover definition. Test editing workflow diagnostics actions semantic tokens. Test refactoring workflow symbols actions edits. Test debugging workflow hover variables evaluation. Test exploration workflow workspace symbols fuzzy search. Test documentation workflow hover doc comments. Verify smooth interaction between features.

### LSP Protocol Compliance
Test full LSP protocol compliance. Verify initialization handshake correct. Test capability negotiation. Verify all implemented methods conform to spec. Test notification handling. Test request-response patterns. Verify error handling per protocol. Test lifecycle management initialization shutdown exit. Validate message format JSON-RPC. Test protocol version compatibility.

### Performance Testing
Measure and verify LSP performance targets. Test hover response time under 100ms. Test completion response time under 50ms. Test semantic tokens under 200ms for large files. Test symbol search under 100ms. Test code actions under 150ms. Test diagnostics publishing under 300ms. Test large file handling performance. Test workspace indexing time. Profile memory usage. Ensure responsive user experience.

### Editor Integration Verification
Test LSP server with real editors. Test VS Code extension integration. Test Neovim LSP client integration. Test Emacs LSP mode integration. Verify all features work in each editor. Test feature behavior matches expectations. Verify UI integration hover popups actions menus. Test performance in real usage. Identify editor-specific quirks. Document any limitations per editor.

### VS Code Setup Documentation
Write comprehensive VS Code setup guide. Document extension installation from marketplace or manual. Configure extension settings for Atlas. Set up file associations for .at files. Configure syntax highlighting. Document feature usage hover actions symbols. Include screenshots showing features. Provide troubleshooting steps. Test setup on fresh VS Code installation.

### Neovim Setup Documentation
Write Neovim LSP setup guide. Document LSP client configuration. Show lua config for Atlas LSP. Configure keybindings for LSP actions. Set up completion integration. Document treesitter if applicable. Show feature usage commands. Provide troubleshooting for common issues. Test setup on fresh Neovim installation.

### Emacs Setup Documentation
Write Emacs LSP setup guide. Document lsp-mode or eglot configuration. Show elisp config for Atlas LSP. Configure keybindings for features. Set up company-mode or similar. Document feature usage. Provide troubleshooting steps. Test setup on fresh Emacs installation.

### LSP Features Documentation
Document all LSP features comprehensively. Describe hover feature with examples. Document code actions quick fixes and refactorings. Explain semantic tokens and syntax highlighting. Describe document and workspace symbols. Document folding ranges. Explain inlay hints configuration. Show feature usage in different editors. Include screenshots and examples.

### Troubleshooting Guide
Create troubleshooting guide for LSP issues. Document common server startup issues. Explain connection problems stdio vs TCP. Troubleshoot feature not working scenarios. Debug performance issues. Provide logging configuration. Show how to file bug reports with logs. Include FAQ section. Document known limitations.

### LSP Status Documentation
Write comprehensive LSP status report. Document implementation status of all three LSP phases. List all implemented features with protocol method names. Describe editor compatibility. Show performance benchmarks. List verification checklist with test coverage. Document known limitations. Propose future enhancements completion refactoring improvements. Conclude LSP is complete and production-ready.

### STATUS.md Update
Update STATUS.md marking LSP category as 3/5 complete with phases 01-03 checked off. Update overall progress percentage.

## Tests (TDD - Use rstest)

**Feature integration tests:**
1. Hover with semantic tokens consistency
2. Code actions with diagnostics
3. Symbols with folding alignment
4. Inlay hints with hover types
5. Workspace and document symbols
6. All features simultaneously
7. Feature interaction safety
8. Complex workflow scenarios

**Protocol compliance tests:**
1. Initialization handshake
2. Capability negotiation
3. Method conformance
4. Notification handling
5. Request-response patterns
6. Error handling protocol
7. Lifecycle management
8. Message format validation
9. Protocol version support
10. Edge case handling

**Performance tests:**
1. Hover response time under 100ms
2. Completion under 50ms
3. Semantic tokens under 200ms
4. Symbol search under 100ms
5. Code actions under 150ms
6. Diagnostics under 300ms
7. Large file handling
8. Memory usage acceptable
9. Workspace indexing time
10. Concurrent request handling

**Editor integration tests:**
1. VS Code features work
2. Neovim features work
3. Emacs features work
4. Feature behavior correct
5. UI integration proper
6. Performance in editors
7. Keybinding responsiveness
8. Error display correct

**Minimum test count:** 100 tests (30 integration, 30 protocol, 30 performance, 10 editors)

## Integration Points
- Uses: All LSP features from phases 01-02
- Uses: CLI LSP launcher from cli/phase-03
- Tests: Complete LSP implementation
- Verifies: Protocol compliance
- Validates: Editor integration
- Creates: Editor setup guides
- Updates: STATUS.md and lsp-status.md
- Output: Production-ready LSP for v0.2

## Acceptance
- All 100+ integration tests pass 30 features 30 protocol 30 performance 10 editors
- Protocol compliance verified fully
- Performance targets met all operations responsive
- Hover under 100ms
- Completion under 50ms
- Semantic tokens under 200ms for large files
- LSP works in VS Code with extension
- LSP works in Neovim with LSP client
- LSP works in Emacs with lsp-mode
- All features functional in all editors
- Editor setup guides complete and tested
- Feature documentation comprehensive
- Troubleshooting guide helpful
- LSP status documentation complete
- STATUS.md updated LSP marked 3/3 complete
- Total LSP test count 250+
- No clippy warnings
- cargo nextest run -p atlas-lsp passes
- LSP production-ready for v0.2
