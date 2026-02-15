# Phase 08: Spec & API Management

**Objective:** Manage specifications and API docs - read, validate, sync with code.

**Priority:** HIGH
**Depends On:** Phases 1-7

---

## Deliverables

### Spec Management
1. ✅ Spec parser (markdown + EBNF)
2. ✅ Spec reader (by section)
3. ✅ Spec search
4. ✅ Spec validator
5. ✅ Grammar validator (EBNF)
6. ✅ Spec sync with code

### API Management
7. ✅ API doc parser
8. ✅ API reader (by function)
9. ✅ API validator (against code)
10. ✅ API generator (from code)
11. ✅ API coverage tracker

---

## Implementation

### 1. Spec Parser

**File:** `internal/spec/parser.go`

```go
type Spec struct {
    Path     string
    Title    string
    Sections []Section
    Grammar  []GrammarRule  // If contains EBNF
}

type Section struct {
    Title   string
    Level   int
    Content string
    CodeBlocks []CodeBlock
}

type GrammarRule struct {
    Name       string
    Production string
    EBNF       string
}

func Parse(path string) (*Spec, error)
func ParseSection(path string, sectionName string) (*Section, error)
func ParseGrammar(path string) ([]GrammarRule, error)
```

### 2. Grammar Validator

**File:** `internal/spec/grammar.go`

Validate EBNF grammar definitions.

```go
func ValidateGrammar(rules []GrammarRule) (*GrammarValidation, error) {
    // Check:
    // 1. EBNF syntax is valid
    // 2. All referenced rules are defined
    // 3. No circular dependencies (or allowed)
    // 4. No ambiguities
}

func CompareGrammarToParser(grammarPath string, parserCode string) (*Comparison, error) {
    // Compare spec grammar to actual parser implementation
}
```

### 3. API Parser

**File:** `internal/api/parser.go`

```go
type APIDoc struct {
    Path      string
    Functions []APIFunction
    Types     []APIType
}

type APIFunction struct {
    Name      string
    Signature string
    Params    []Param
    Returns   string
    Errors    []string
    Example   string
    Notes     string
}

func Parse(path string) (*APIDoc, error)
func ParseFunction(path string, functionName string) (*APIFunction, error)
```

### 4. API Validator

**File:** `internal/api/validator.go`

Validate API docs against actual code.

```go
func Validate(apiDoc *APIDoc, codeDir string) (*APIValidation, error) {
    // For each function in API doc:
    // 1. Find function in code
    // 2. Compare signatures
    // 3. Verify return types match
    // 4. Check error types match
    // 5. Count: API says X functions, code has X functions?
}
```

### 5. API Generator

**File:** `internal/api/generator.go`

Auto-generate API docs from code comments.

```go
func Generate(codeDir string) (*APIDoc, error) {
    // Parse Rust code
    // Extract doc comments
    // Extract function signatures
    // Generate markdown API doc
}
```

---

## Commands

### Spec Commands

**`spec read docs/specification/types.md --section "Generic Types"`:**
```json
{
  "ok": true,
  "spec": "docs/specification/types.md",
  "section": "Generic Types",
  "content": "...",
  "code_blocks": [
    {"lang": "atlas", "code": "let x: Option<number> = Some(42)"}
  ]
}
```

**`spec search "pattern matching"`:**
```json
{
  "ok": true,
  "query": "pattern matching",
  "results": [
    {"spec": "types.md", "section": "Pattern Matching", "score": 95},
    {"spec": "language-semantics.md", "section": "Match Expressions", "score": 80}
  ]
}
```

**`spec validate docs/specification/syntax.md`:**
```json
{
  "ok": true,
  "grammar_valid": true,
  "refs_valid": true,
  "code_blocks_valid": true
}
```

**`spec check-grammar`:**
```json
{
  "ok": true,
  "rules": 47,
  "valid": true,
  "warnings": [
    {"rule": "Expression", "issue": "Potentially ambiguous with Statement"}
  ]
}
```

### API Commands

**`api read docs/api/stdlib.md`:**
```json
{
  "ok": true,
  "functions": 89,
  "by_category": {
    "string": 18,
    "array": 21,
    "math": 18,
    "json": 17,
    "io": 10,
    "collections": 5
  }
}
```

**`api validate`:**
```json
{
  "ok": false,
  "errors": [
    {
      "function": "HashMap.get",
      "api": "HashMap.get(map, key) -> Option<T>",
      "code": "pub fn get(args: &[Value]) -> Result<Value, RuntimeError>",
      "issue": "API signature doesn't match code implementation"
    }
  ],
  "coverage": {
    "documented": 89,
    "implemented": 91,
    "missing_docs": ["HashMap.entry", "HashMap.drain"]
  }
}
```

**`api generate crates/atlas-runtime/src/stdlib/`:**
```bash
# Parses code, generates API markdown
# Output: Generated docs/api/stdlib.md
```

**`api coverage`:**
```json
{
  "ok": true,
  "total_functions": 91,
  "documented": 89,
  "coverage": 98,
  "missing": [
    {"function": "HashMap.entry", "file": "collections/hashmap.rs:245"},
    {"function": "HashMap.drain", "file": "collections/hashmap.rs:280"}
  ]
}
```

---

## Testing

```bash
# Read spec section
atlas-dev spec read docs/specification/types.md --section "Generic Types"

# Search specs
atlas-dev spec search "generics" | jq '.results | length'
# Expected: 5+

# Validate grammar
atlas-dev spec check-grammar
# Expected: {"ok": true}

# Read API
atlas-dev api read docs/api/stdlib.md | jq '.functions'
# Expected: 89

# Validate API
atlas-dev api validate | jq '.ok'
# Expected: true (if no mismatches)

# Check coverage
atlas-dev api coverage | jq '.coverage'
# Expected: 95+
```

---

## Acceptance Criteria

- [x] Spec parser reads markdown and EBNF correctly
- [x] Grammar validator catches EBNF errors
- [x] API parser extracts function signatures
- [x] API validator compares docs to code
- [x] API generator creates docs from code comments
- [x] Coverage tracker finds undocumented functions
- [x] All commands return valid JSON
- [x] Validation catches doc/code mismatches

---

## Next Phase

**Phase 9:** Parity Validation (CRITICAL)
- Cross-system validation
- Code/spec/doc parity
- Comprehensive consistency checks
