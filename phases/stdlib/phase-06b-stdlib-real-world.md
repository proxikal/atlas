# Phase 06b: Standard Library Integration Tests - Real-World Patterns

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Phase 06a must be complete.

**Verification:**
```bash
# Verify phase 06a integration tests exist and pass
ls crates/atlas-runtime/tests/stdlib_integration_tests.rs
cargo test -p atlas-runtime stdlib_integration_tests -- --nocapture

# Verify all stdlib modules still working
cargo test -p atlas-runtime string -- --exact --nocapture
cargo test -p atlas-runtime array -- --exact --nocapture
```

**What's needed:**
- Phase 06a complete with all integration tests passing
- All unit tests still passing
- Interpreter/VM parity verified

**If missing:** Complete phase 06a first

---

## Objective

Create realistic Atlas programs that demonstrate real-world usage patterns of the standard library. These tests should read like actual applications someone would write - CSV processing, JSON API handling, log file analysis, data transformation pipelines, etc. Verify that the stdlib is practical and usable for real tasks.

## Files

**Create:** `crates/atlas-runtime/tests/stdlib_real_world_tests.rs` (~600 lines)

## Dependencies

- Phase 06a complete
- All stdlib modules functional
- Interpreter/VM parity maintained

## Implementation

### 1. CSV Processing

Realistic CSV reading, parsing, filtering, transforming, and writing:
- Read CSV file with headers
- Parse rows into structured data
- Filter rows by criteria
- Transform columns
- Calculate aggregates (sum, average, count)
- Write output CSV
- Handle malformed data

**Example program:**
```atlas
// Read CSV with sales data
let csv = readFile("sales.csv")
let lines = split(csv, "\n")
let header = first(lines)
let rows = rest(lines)

// Parse and filter high-value sales
let sales = map(rows, fn(line) {
    let fields = split(line, ",")
    jsonObject() // Build structured record
})
let highValue = filter(sales, fn(sale) {
    jsonGet(sale, "amount") > 1000
})

// Calculate total
let total = reduce(highValue, fn(sum, sale) {
    sum + jsonGet(sale, "amount")
}, 0)
```

### 2. JSON API Response Handling

Parse JSON responses, extract data, transform, validate:
- Parse JSON API response
- Navigate nested objects/arrays
- Extract specific fields
- Transform data structures
- Validate types
- Build response objects
- Error handling for missing fields

**Example program:**
```atlas
// Parse GitHub API-style response
let response = parseJson(readFile("api_response.json"))
let users = jsonGet(response, "users")

// Extract emails
let emails = map(users, fn(user) {
    let email = jsonGet(user, "email")
    let name = jsonGet(user, "name")
    concat(name, " <", email, ">")
})

// Filter valid emails
let valid = filter(emails, fn(email) {
    contains(email, "@")
})
```

### 3. Log File Analysis

Read logs, parse entries, filter by severity, aggregate statistics:
- Read multi-line log file
- Parse log entries (timestamp, level, message)
- Filter by log level (ERROR, WARN, INFO)
- Count occurrences
- Extract error messages
- Group by category
- Generate summary report

**Example program:**
```atlas
// Parse log file
let logs = readFile("app.log")
let lines = split(logs, "\n")

// Extract error lines
let errors = filter(lines, fn(line) {
    contains(line, "ERROR")
})

// Extract error messages
let messages = map(errors, fn(line) {
    let parts = split(line, "]")
    trim(last(parts))
})

// Count unique errors
let unique = reduce(messages, fn(acc, msg) {
    // Build frequency map
    acc
}, jsonObject())
```

### 4. Data Transformation Pipelines

Complex multi-step transformations:
- Read structured data
- Filter by multiple criteria
- Transform with multiple functions
- Aggregate results
- Format output
- Handle errors throughout

**Example program:**
```atlas
// Process product inventory
let data = parseJson(readFile("inventory.json"))
let products = jsonGet(data, "products")

// Find low stock items, transform, and sort
let lowStock = filter(products, fn(p) {
    jsonGet(p, "quantity") < 10
})
let withReorder = map(lowStock, fn(p) {
    let reorder = jsonGet(p, "min_stock") - jsonGet(p, "quantity")
    jsonSet(p, "reorder_amount", reorder)
    p
})
let sorted = sort(withReorder, fn(a, b) {
    jsonGet(b, "reorder_amount") - jsonGet(a, "reorder_amount")
})
```

### 5. Text Processing and Analysis

String-heavy operations:
- Word frequency counting
- Text statistics (word count, line count, avg length)
- Find and replace patterns
- Extract URLs/emails
- Case transformations
- Whitespace normalization

**Example program:**
```atlas
// Analyze markdown document
let markdown = readFile("README.md")
let lines = split(markdown, "\n")

// Count headers
let headers = filter(lines, fn(line) {
    startsWith(line, "#")
})
let headerCount = length(headers)

// Extract links [text](url)
let links = []  // Build list of URLs
let words = split(markdown, " ")
let linkWords = filter(words, fn(w) {
    contains(w, "](")
})
```

### 6. Configuration File Processing

Parse and validate config files:
- Read JSON/CSV config
- Validate required fields
- Apply defaults
- Transform settings
- Write updated config
- Error handling for invalid config

## Tests (TDD - Use rstest)

**Test categories (minimum 150 tests):**
1. CSV processing (30 tests)
   - Read/parse/filter/transform/write
   - Header handling
   - Malformed data handling
   - Empty files, single row, large files
   - Aggregation calculations

2. JSON API handling (30 tests)
   - Parse responses
   - Navigate nested structures
   - Extract fields
   - Transform data
   - Validate types
   - Error handling

3. Log file analysis (30 tests)
   - Parse log entries
   - Filter by severity
   - Extract messages
   - Count occurrences
   - Group by category
   - Multi-line entries

4. Data transformation pipelines (30 tests)
   - Multi-step filters
   - Chained transformations
   - Aggregations
   - Sorting
   - Error propagation
   - Edge cases

5. Text processing (20 tests)
   - Word frequency
   - Text statistics
   - Pattern extraction
   - Case transformations
   - Whitespace handling

6. Configuration processing (10 tests)
   - Parse configs
   - Validate fields
   - Apply defaults
   - Transform settings
   - Error handling

**All tests MUST verify interpreter/VM parity**

## Integration Points

- Uses: All stdlib modules from phases 01-05
- Uses: Phase 06a integration test infrastructure
- Uses: Both interpreter and VM for parity
- Uses: `tempfile` for creating test files
- Pattern: Realistic programs that users would write
- Output: Confidence that stdlib is practical and usable

## Acceptance Criteria

- [ ] All 150+ real-world tests pass
- [ ] Zero parity violations (100% identical output in both engines)
- [ ] CSV processing programs work correctly (30 tests)
- [ ] JSON API handling programs work correctly (30 tests)
- [ ] Log file analysis programs work correctly (30 tests)
- [ ] Data transformation pipelines work correctly (30 tests)
- [ ] Text processing programs work correctly (20 tests)
- [ ] Configuration processing programs work correctly (10 tests)
- [ ] cargo test passes
- [ ] No clippy warnings
- [ ] Interpreter/VM parity: 100%

## Notes

**Testing protocol:**
- Write test function for one scenario
- Run ONLY that test: `cargo test -p atlas-runtime test_function_name -- --exact`
- Iterate until passing
- Move to next test
- NEVER run full suite during development

**Real-world focus:**
- Tests should read like actual programs
- Use realistic data and scenarios
- Demonstrate practical stdlib usage
- Show common patterns users will need

**This phase demonstrates stdlib practicality. Phase 06c adds benchmarks and complete documentation.**
