# Phase 14: Documentation Generator

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Parser with doc comment support (frontend/phase-02) and Module system (foundation/phase-06).

**Verification Steps:**
1. Check STATUS.md:
   - Frontend section, phase-02 (Formatter) should be ✅
   - Foundation section, phase-06 (Module System) should be ✅

2. Check spec for doc comment syntax:
   - Read `docs/specification/syntax.md` for comment syntax
   - Check if doc comments (`///` or `/** */`) are defined

3. Verify parser preserves comments (if frontend/phase-02 complete):
   ```bash
   grep -n "Comment\|comment\|doc" crates/atlas-runtime/src/ast.rs | head -10
   cargo test parser 2>&1 | grep "test result"
   ```

4. Verify module system exists:
   ```bash
   ls crates/atlas-runtime/src/modules/mod.rs
   cargo test modules 2>&1 | grep "test result"
   ```

**Frontend/phase-02 dependency:**
- Phase-02 (Formatter) should preserve and expose doc comments
- Parser should include comment nodes in AST
- If phase-02 incomplete, doc comments may not be accessible

**Foundation/phase-06 dependency:**
- Module system provides structure for documentation
- Need module exports/imports for cross-references
- Need module resolution for linking

**Decision Tree:**

a) If both frontend/phase-02 AND foundation/phase-06 complete:
   → Proceed with phase-14
   → Extract doc comments from AST
   → Use module system for documentation structure

b) If frontend/phase-02 incomplete:
   → STOP immediately
   → Report: "Frontend phase-02 required for doc comment support"
   → Complete frontend/phase-02 first
   → Then return to phase-14

c) If foundation/phase-06 incomplete:
   → STOP immediately
   → Report: "Foundation phase-06 required for module documentation"
   → Complete foundation/phase-06 first
   → Then return to phase-14

d) If spec doesn't define doc comments:
   → Check frontend/phase-02 for doc comment syntax it added
   → Use triple-slash (`///`) convention (standard for doc comments)
   → Document doc comment syntax in this phase

e) If parser doesn't preserve comments yet:
   → Extend parser to preserve doc comments
   → Add Comment AST node if needed
   → This is enhancement to parser, not full rewrite

**No user questions needed:** Prerequisites are verifiable via STATUS.md and file checks. Doc comment syntax follows standard conventions if spec silent.

---

## Objective
Implement documentation generator extracting doc comments from source code and producing comprehensive HTML documentation with API references, examples, and cross-references - enabling professional documentation for Atlas projects and packages.

## Files
**Create:** `crates/atlas-doc/` (new crate ~1500 lines total)
**Create:** `crates/atlas-doc/src/lib.rs` (~200 lines)
**Create:** `crates/atlas-doc/src/extractor.rs` (~500 lines)
**Create:** `crates/atlas-doc/src/generator.rs` (~400 lines)
**Create:** `crates/atlas-doc/src/templates.rs` (~300 lines)
**Create:** `crates/atlas-doc/src/markdown.rs` (~100 lines)
**Update:** `crates/atlas-cli/src/commands/doc.rs` (~300 lines)
**Create:** `docs/documentation-guide.md` (~700 lines)
**Create:** `templates/doc/` (HTML templates directory)
**Tests:** `crates/atlas-doc/tests/doc_gen_tests.rs` (~500 lines)

## Dependencies
- Parser with doc comment extraction
- Module system for package structure
- Markdown parser (pulldown-cmark)
- HTML templating (tera or similar)
- Reflection API for runtime info

## Implementation

### Documentation Comment Format
Define doc comment syntax using triple-slash or block style. Triple-slash for line comments preceding declarations. Block comments with doc prefix for multi-line. Markdown formatting in doc comments. Code examples in fenced code blocks. Link syntax for cross-references. Parameter documentation with param tag. Return value documentation with returns tag. Example sections with examples tag. Deprecation warnings with deprecated tag. Since version with since tag.

### Documentation Extraction
Extract documentation from source files. Parse source files preserving doc comments. Associate comments with declarations. Extract function signatures with parameter names and types. Extract type definitions with fields. Extract module-level documentation. Collect examples from doc comments. Parse markdown in comments. Resolve cross-references. Build documentation tree mirroring code structure.

### HTML Generation
Generate static HTML documentation site. Index page with package overview. Module pages with exported symbols. Function pages with signatures and examples. Type pages with field documentation. Sidebar navigation tree. Search functionality for symbols. Syntax highlighting for code examples. Responsive design for mobile. Dark mode support. Anchor links for sections.

### Cross-Reference Resolution
Resolve links between documentation pages. Link function references to definition pages. Link type references to type pages. Link module references to module pages. External package links if available. Broken link detection and warnings. Generate cross-reference index. Symbol disambiguation for common names.

### Code Example Extraction and Testing
Extract code examples from documentation. Mark examples as tested or untested. Run tested examples as integration tests. Verify examples compile and run. Report example failures in documentation. Include example output in docs. Support hidden example code for setup. Doctest framework similar to Rust.

### API Documentation Templates
Create professional HTML templates. Clean modern design. Consistent styling across pages. Mobile-responsive layout. Accessibility compliance. Print-friendly styles. Customizable themes. Brand customization options. Footer with generator credit. Metadata for SEO.

### Search and Navigation
Implement client-side search for symbols. Search index generation from documentation. Fuzzy search for symbol names. Filter search by type function, module, type. Keyboard navigation support. Breadcrumb navigation. Table of contents for long pages. Previous and next page links.

### Documentation Configuration
Configure documentation generation via manifest. Include and exclude patterns. Custom theme selection. Output directory specification. Title and branding customization. External link configuration. Privacy settings for internal symbols. Version information display. Logo and favicon configuration.

## Tests (TDD - Use rstest)

**Extraction tests:**
1. Extract doc comment from function
2. Extract doc comment from type
3. Extract module-level documentation
4. Parse markdown in comments
5. Extract code examples
6. Extract parameter documentation
7. Extract return value docs
8. Associate comment with declaration
9. Multiple doc comments
10. Doc comment edge cases

**HTML generation tests:**
1. Generate index page
2. Generate module page
3. Generate function page
4. Generate type page
5. Sidebar navigation
6. Syntax highlighting
7. Responsive design validation
8. Dark mode styles
9. Anchor link generation
10. Valid HTML output

**Cross-reference tests:**
1. Resolve function link
2. Resolve type link
3. Resolve module link
4. Detect broken links
5. Symbol disambiguation
6. External links
7. Cross-reference index

**Code example tests:**
1. Extract code example
2. Run tested example
3. Verify example compiles
4. Report example failure
5. Include example output
6. Hidden setup code
7. Untested example marking

**Search tests:**
1. Generate search index
2. Search for symbol
3. Fuzzy search matching
4. Filter by type
5. Search result ranking

**Configuration tests:**
1. Load doc configuration
2. Include and exclude patterns
3. Custom theme application
4. Output directory setting
5. Title customization
6. Brand configuration

**Integration tests:**
1. Generate docs for test project
2. Multi-module documentation
3. Documentation with dependencies
4. Doctest execution
5. Complete documentation site

**Minimum test count:** 60 tests

## Integration Points
- Uses: Parser with doc comments
- Uses: Module system from phase-06
- Uses: Reflection API from phase-12
- Uses: CLI framework from v0.1
- Creates: atlas-doc crate
- Creates: Documentation generator
- Output: Professional HTML documentation

## Acceptance
- Extract doc comments from source
- Generate HTML documentation site
- Cross-references resolve correctly
- Search functionality works
- Code examples testable as doctests
- Templates produce valid HTML
- Responsive design on mobile
- Dark mode available
- 60+ tests pass
- Documentation for Atlas stdlib generated
- Configuration via manifest works
- CLI atlas doc command functional
- Documentation guide complete
- No clippy warnings
- cargo test passes
