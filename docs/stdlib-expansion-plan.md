# Atlas Standard Library Expansion Plan

**Version:** 1.0
**Status:** Planning
**Last Updated:** 2026-02-12

---

## Overview

This document defines the roadmap for growing the Atlas standard library beyond v0.1 in a controlled, principled manner. The stdlib will expand in lockstep with language features, maintaining Atlas's core values of simplicity, predictability, and AI-friendliness.

---

## Current State (v0.1)

**Implemented Functions:**
- `print(value: string|number|bool|null) -> void` - Output to stdout
- `len(value: string|T[]) -> number` - Length of strings/arrays
- `str(value: number|bool|null) -> string` - Convert to string

**Characteristics:**
- Pure functions except `print`
- Type-safe with runtime validation
- Error code `AT0102` for invalid arguments
- All errors include span information

---

## Expansion Roadmap

### Phase 1: v0.1 Completion (Current)
**Timeline:** Ongoing
**Goal:** Complete foundational stdlib with testing and documentation

**Deliverables:**
- ✅ Core functions (`print`, `len`, `str`)
- ✅ Comprehensive tests
- ✅ Documentation sync
- ✅ Expansion plan (this document)
- ✅ Security model for I/O
- ✅ JSON stdlib plan

---

### Phase 2: v0.2 - String & Array Utilities
**Timeline:** Post v0.1 release
**Goal:** Essential string and array manipulation

**String Module (`string::*`):**
- `split(s: string, sep: string) -> string[]` - Split string by separator
- `join(arr: string[], sep: string) -> string` - Join array into string
- `trim(s: string) -> string` - Remove leading/trailing whitespace
- `upper(s: string) -> string` - Convert to uppercase
- `lower(s: string) -> string` - Convert to lowercase
- `contains(s: string, substr: string) -> bool` - Substring check
- `replace(s: string, old: string, new: string) -> string` - Replace substring
- `charAt(s: string, index: number) -> string` - Get character at index
- `substring(s: string, start: number, end: number) -> string` - Extract substring

**Array Module (`array::*`):**
- `push(arr: T[], item: T) -> void` - Append to array (mutates)
- `pop(arr: T[]) -> T|null` - Remove last element (mutates)
- `slice(arr: T[], start: number, end: number) -> T[]` - Extract subarray
- `concat(arr1: T[], arr2: T[]) -> T[]` - Combine arrays
- `reverse(arr: T[]) -> T[]` - Reverse array (new copy)
- `indexOf(arr: T[], item: T) -> number` - Find index of item (-1 if not found)

**Rationale:**
- No external dependencies required
- Foundational for most programs
- Low security risk (pure functions)
- TypeScript/Python familiarity

**Exit Criteria:**
- All functions implemented with type validation
- Test coverage >95%
- Documentation complete
- No breaking changes to v0.1 API

---

### Phase 3: v0.5 - File I/O & JSON
**Timeline:** After module system (v1.0 dependency)
**Goal:** Basic file operations and JSON parsing

**File Module (`file::*`):**
- `read(path: string) -> string` - Read entire file as string
- `write(path: string, content: string) -> void` - Write string to file
- `append(path: string, content: string) -> void` - Append to file
- `exists(path: string) -> bool` - Check if file exists
- `delete(path: string) -> void` - Remove file
- `readLines(path: string) -> string[]` - Read file as lines

**JSON Module (`json::*`):**
- `parse(s: string) -> any` - Parse JSON string (requires `any` type or dynamic values)
- `stringify(value: any) -> string` - Convert value to JSON

**Security Considerations:**
- Sandbox mode by default (see `docs/io-security-model.md`)
- Explicit path allowlist via CLI flag
- No network access in this phase
- Audit logging for file operations

**Rationale:**
- Most-requested feature for scripting
- Enables real-world automation
- JSON critical for AI agent workflows
- Security model must be designed first (Phase 05)

**Exit Criteria:**
- Security model documented and implemented
- File operations restricted to allowlist
- Error messages include file paths
- Tests include permission denied scenarios
- Documentation warns about security model

---

### Phase 4: v1.0 - Path, Time & Collections
**Timeline:** v1.0 stable release
**Goal:** Complete core stdlib for production use

**Path Module (`path::*`):**
- `join(...parts: string[]) -> string` - Combine path segments
- `dirname(path: string) -> string` - Get directory name
- `basename(path: string) -> string` - Get file name
- `extname(path: string) -> string` - Get file extension
- `isAbsolute(path: string) -> bool` - Check if path is absolute
- `resolve(...paths: string[]) -> string` - Resolve to absolute path

**Time Module (`time::*`):**
- `now() -> number` - Current Unix timestamp (milliseconds)
- `sleep(ms: number) -> void` - Pause execution
- `format(timestamp: number, fmt: string) -> string` - Format timestamp
- `parse(s: string, fmt: string) -> number` - Parse time string

**Collections Module (`collections::*`):**
- `map<T, U>(arr: T[], fn: (T) -> U) -> U[]` - Map array
- `filter<T>(arr: T[], fn: (T) -> bool) -> T[]` - Filter array
- `reduce<T, U>(arr: T[], fn: (U, T) -> U, init: U) -> U` - Reduce array
- `forEach<T>(arr: T[], fn: (T) -> void) -> void` - Iterate array
- `some<T>(arr: T[], fn: (T) -> bool) -> bool` - Test if any match
- `every<T>(arr: T[], fn: (T) -> bool) -> bool` - Test if all match
- `find<T>(arr: T[], fn: (T) -> bool) -> T|null` - Find first match

**Rationale:**
- Path operations are cross-platform necessities
- Time functions enable scheduling and benchmarking
- Higher-order functions unlock functional patterns
- Requires first-class function support

**Exit Criteria:**
- Cross-platform path handling (Windows/Unix)
- Time zone handling documented
- Higher-order functions type-check correctly
- Performance benchmarks meet targets

---

### Phase 5: v1.1+ - Advanced Features
**Timeline:** Post v1.0
**Goal:** Differentiating features for advanced use cases

**HTTP Module (`http::*`):**
- `get(url: string) -> string` - HTTP GET request
- `post(url: string, body: string) -> string` - HTTP POST request
- `fetch(req: Request) -> Response` - Full HTTP client

**Process Module (`process::*`):**
- `exec(cmd: string, args: string[]) -> string` - Execute command
- `env(key: string) -> string|null` - Get environment variable
- `exit(code: number) -> void` - Exit with status code
- `args() -> string[]` - Get command-line arguments

**Concurrency Module (`async::*`):**
- `spawn(fn: () -> T) -> Handle<T>` - Spawn lightweight task
- `await(handle: Handle<T>) -> T` - Wait for task completion
- `chan<T>() -> Channel<T>` - Create channel
- `send(ch: Channel<T>, value: T) -> void` - Send to channel
- `recv(ch: Channel<T>) -> T` - Receive from channel

**Rationale:**
- HTTP enables web scraping and API integration
- Process execution unlocks system automation
- Concurrency differentiates Atlas from similar languages
- Requires runtime support for green threads

**Security Model:**
- HTTP requests require `--allow-net` flag with domain allowlist
- Process execution requires `--allow-run` flag with command allowlist
- Channels are memory-safe and panic-free

**Exit Criteria:**
- Concurrency model does not deadlock
- Network requests timeout appropriately
- Process execution sandboxed properly
- Documentation includes security warnings

---

## Inclusion Criteria

All proposed stdlib functions must meet these requirements:

### 1. **Necessity**
- ✅ Solves a common problem (used in >20% of programs)
- ✅ Cannot be reasonably implemented in userland
- ✅ Benefits from tight runtime integration

### 2. **Safety**
- ✅ Type-safe with clear error messages
- ✅ Cannot cause memory corruption
- ✅ Security implications documented
- ✅ Fails explicitly (no silent errors)

### 3. **Portability**
- ✅ Works identically on macOS, Windows, Linux
- ✅ No platform-specific behavior without explicit opt-in
- ✅ Uses cross-platform abstractions (e.g., `std::path`)

### 4. **Performance**
- ✅ Does not introduce O(n²) or worse complexity unexpectedly
- ✅ Large data operations provide progress feedback or streaming
- ✅ No hidden I/O or network calls

### 5. **Consistency**
- ✅ Naming follows stdlib conventions (lowercase, descriptive)
- ✅ Error handling matches existing patterns (`AT01XX` codes)
- ✅ Purity documented (`pure` or `effectful`)

### 6. **Documentation**
- ✅ Signature with types documented
- ✅ Behavior described with examples
- ✅ Edge cases and errors listed
- ✅ Security considerations noted (for I/O functions)

### 7. **Testing**
- ✅ Unit tests for happy path and edge cases
- ✅ Integration tests with REPL and VM
- ✅ Cross-platform tests for file/path operations

### 8. **Dependencies**
- ✅ Prefer zero external dependencies
- ✅ External crates must be: well-maintained, security-audited, minimal
- ✅ Dependency list must be reviewed and approved

---

## Stability Policy

### Versioning
- **Stdlib version matches language version** (e.g., stdlib v1.0 ships with Atlas v1.0)
- **Semantic versioning applies to stdlib API:**
  - Major version: breaking changes (signature changes, removals)
  - Minor version: new functions, backward-compatible enhancements
  - Patch version: bug fixes only

### Deprecation Process
1. **Mark as deprecated** in docs (min 1 minor version warning period)
2. **Emit runtime warning** when function is called (next minor version)
3. **Remove function** (next major version)

**Example:**
- v1.0: Function `foo()` exists
- v1.1: Docs say `foo()` is deprecated, recommend `bar()` instead
- v1.2: Calling `foo()` prints deprecation warning to stderr
- v2.0: Function `foo()` removed

### Breaking Changes
- **Allowed in major versions only** (v1.0 → v2.0)
- **Require migration guide** in release notes
- **Should be rare** (avoid churn)

### Experimental Functions
- Functions in preview may be prefixed with `experimental_*`
- No stability guarantees
- Can change or be removed in minor versions
- Must graduate to stable or be removed before next major version

---

## Module Organization

### Namespace Strategy

**v0.1-v0.9 (Pre-module system):**
- All functions in global namespace: `print()`, `len()`, `str()`

**v1.0+ (With module system):**
- Functions organized in namespaced modules:
  ```atlas
  import string;
  import file;
  import json;

  let data = file::read("data.json");
  let obj = json::parse(data);
  let name = string::trim(obj.name);
  ```

**Prelude (Auto-imported):**
- Core functions available without import:
  - `print()`, `len()`, `str()` (v0.1 functions)
  - `panic()` (v1.0)
  - Common type conversions

**Explicit Import Required:**
- All I/O functions (`file::*`, `http::*`, `process::*`)
- Specialized utilities (`collections::*`, `time::*`)

---

## Decision Framework

When evaluating new stdlib proposals, ask:

1. **Is this foundational?**
   Could most programs benefit from this, or is it niche?

2. **Can userland do it?**
   If yes, should it be a third-party library instead?

3. **Does it require runtime access?**
   Does it need VM internals, file system, or system calls?

4. **Is the API surface minimal?**
   Can we solve the problem with fewer functions?

5. **Does it align with Atlas values?**
   Is it simple, predictable, and AI-friendly?

6. **What are the security risks?**
   Does it introduce new attack vectors?

7. **Is it testable and portable?**
   Can we verify it works the same everywhere?

**Approval Process:**
- Proposals submitted as GitHub issues with template
- Community discussion period (min 2 weeks)
- Core team approval required
- Implementation phase tracked in roadmap
- Release in next minor version if backward-compatible

---

## Anti-Goals

**We will NOT include:**

### 1. **Overly Specific Functions**
- ❌ `isValidEmail(s: string) -> bool` - too many edge cases, userland can do it
- ❌ `fibonacci(n: number) -> number` - educational, not foundational

### 2. **External Service Integrations**
- ❌ `aws::*`, `stripe::*`, `openai::*` - belong in third-party libraries
- ✅ Generic HTTP client is acceptable (userland can wrap it)

### 3. **UI/Graphics Primitives**
- ❌ `draw()`, `window()`, `render()` - Atlas is not a GUI framework
- ✅ Terminal I/O (stdin/stdout) is acceptable

### 4. **Opinionated Algorithms**
- ❌ `sortBy()`, `groupBy()` - many valid implementations, userland choice
- ✅ Basic `sort()` is acceptable with documented behavior

### 5. **Stateful Globals**
- ❌ `random::seed()` with global state - prefer explicit RNG objects
- ✅ `random::random() -> number` with platform RNG is acceptable

### 6. **Locale-Specific Logic**
- ❌ `formatCurrency()`, `formatDate()` - too many locale variations
- ✅ Basic `format()` with explicit format strings is acceptable

---

## Testing Requirements

All stdlib functions must include:

### Unit Tests
- **Happy path:** Correct inputs produce expected outputs
- **Type validation:** Wrong types return `InvalidStdlibArgument` (AT0102)
- **Edge cases:** Empty strings, zero, negative numbers, null, etc.
- **Boundary conditions:** Max values, min values, overflow behavior

### Integration Tests
- **REPL:** Function works correctly in interactive mode
- **VM:** Bytecode execution matches interpreter
- **Error spans:** Error messages point to correct call site

### Cross-Platform Tests
- **File paths:** Windows (`C:\`) vs Unix (`/`)
- **Line endings:** CRLF vs LF
- **Character encodings:** UTF-8 handling

### Performance Tests
- **No regressions:** New functions do not slow down existing code
- **Scalability:** Functions handle large inputs gracefully
- **Memory:** No unbounded allocations

---

## Documentation Standards

Each stdlib function must be documented with:

### Signature
```atlas
function_name(param1: Type1, param2: Type2) -> ReturnType
```

### Description
One-sentence summary of what the function does.

### Parameters
- `param1` - Description of first parameter
- `param2` - Description of second parameter

### Return Value
What the function returns, including special cases (e.g., `-1` for "not found").

### Errors
- `AT0102` - Invalid argument type (with examples)
- `AT0201` - File not found (for I/O functions)
- etc.

### Purity
- `pure` - No side effects, same input always produces same output
- `effectful` - Has side effects (I/O, mutation, randomness)

### Examples
```atlas
// Example 1: Basic usage
let result = function_name("input", 42);
print(result); // "expected output"

// Example 2: Edge case
let empty = function_name("", 0);
print(empty); // ""
```

### Security Notes
(For I/O functions only)
- Requires `--allow-*` flag
- Path restrictions apply
- Audit logging enabled

---

## Implementation Checklist

For each new stdlib function:

- [ ] **Proposal approved** (GitHub issue with community consensus)
- [ ] **Signature designed** (types, nullability, return value)
- [ ] **Implementation written** (`crates/atlas-runtime/src/stdlib/*.rs`)
- [ ] **Unit tests added** (>95% coverage)
- [ ] **Integration tests added** (REPL + VM)
- [ ] **Cross-platform tests added** (if applicable)
- [ ] **Documentation written** (`docs/stdlib.md`)
- [ ] **Error codes assigned** (`docs/diagnostics.md`)
- [ ] **Security model reviewed** (if I/O function)
- [ ] **Code review completed** (core team approval)
- [ ] **Changelog updated** (`CHANGELOG.md`)
- [ ] **Release notes drafted** (for next minor version)

---

## Migration Path (v0.1 → v1.0)

### Breaking Changes
None planned. All v0.1 functions (`print`, `len`, `str`) will remain in prelude.

### New Namespaces
When module system lands in v1.0:
- v0.1 functions stay in global namespace (backward compatible)
- New functions live in namespaced modules
- Prelude auto-imports common functions

### Example
```atlas
// v0.1 code (still works in v1.0)
print("Hello");
let length = len("world");

// v1.0 code (new features)
import file;
import json;

let data = file::read("data.json");
let obj = json::parse(data);
print(obj.name);
```

---

## Summary

**Atlas stdlib will grow deliberately:**
1. **Start small** (v0.1: 3 functions)
2. **Add essentials** (v0.2: strings/arrays)
3. **Enable I/O** (v0.5: files/JSON)
4. **Complete core** (v1.0: path/time/collections)
5. **Differentiate** (v1.1+: concurrency/HTTP)

**Guided by principles:**
- Necessity, safety, portability
- Minimal API surface
- Zero external dependencies (where possible)
- Comprehensive testing
- Clear security model

**Next steps:**
- Complete Phase 05: I/O Security Model
- Complete Phase 06: JSON Stdlib Plan
- Implement v0.2 string/array utilities
- Design module system for v1.0

---

**References:**
- `docs/stdlib.md` - Current stdlib specification
- `docs/io-security-model.md` - Security boundaries (Phase 05)
- `docs/json-stdlib-plan.md` - JSON design (Phase 06)
- `Atlas-SPEC.md` - Language roadmap
- `PRD.md` - Project requirements
