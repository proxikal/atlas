# Domain Verification Queries

**Purpose:** Surgical grep patterns to verify codebase structure BEFORE writing code.
**Rule:** Run queries. Note patterns. Then write. Never assume.

---

## AST Domain

**When:** Phase touches parser, expressions, statements, AST nodes.

```bash
# Item variants (top-level declarations)
Grep pattern="^\s+[A-Z][a-z]+.*\{|^\s+[A-Z][a-z]+\(" path="crates/atlas-runtime/src/ast.rs" output_mode="content" -A=0

# Statement structs (field names)
Grep pattern="pub struct \w+Stmt" path="crates/atlas-runtime/src/ast.rs" output_mode="content" -A=5

# Expression enum (variant format: tuple vs struct)
Grep pattern="pub enum Expr" path="crates/atlas-runtime/src/ast.rs" output_mode="content" -A=50

# Literal types
Grep pattern="pub enum Literal" path="crates/atlas-runtime/src/ast.rs" output_mode="content" -A=10
```

**Key questions:**
- Tuple variant `Expr::X(A, B)` or struct variant `Expr::X { a, b }`?
- What are the exact field names on Stmt structs?
- What's the Span position (first, last, separate field)?

---

## Type System Domain

**When:** Phase touches types, type checker, type annotations.

```bash
# Type enum variants
Grep pattern="pub enum Type" path="crates/atlas-runtime/src/types.rs" output_mode="content" -A=30

# Type variant format (tuple vs struct)
Grep pattern="^\s+[A-Z]\w+\s*\{|^\s+[A-Z]\w+\s*\(" path="crates/atlas-runtime/src/types.rs" output_mode="content"
```

**Key questions:**
- Struct variant `Type::X { field }` or tuple variant `Type::X(inner)`?
- What variants exist? (Number, String, Array, Function, etc.)

---

## Value Domain

**When:** Phase touches runtime values, Value enum.

```bash
# Value enum variants
Grep pattern="pub enum Value" path="crates/atlas-runtime/src/value.rs" output_mode="content" -A=30

# Collection wrapper types
Grep pattern="Arc<Mutex<" path="crates/atlas-runtime/src/value.rs" output_mode="content"
```

**Key questions:**
- Which variants use `Arc<Mutex<>>`?
- Which are immutable (`Arc<String>`)?

---

## Stdlib Domain

**When:** Phase touches builtin functions.

```bash
# Registry pattern (is_builtin)
Grep pattern="fn is_builtin" path="crates/atlas-runtime/src/stdlib/mod.rs" output_mode="content" -A=20

# Call pattern (call_builtin signature)
Grep pattern="fn call_builtin" path="crates/atlas-runtime/src/stdlib/mod.rs" output_mode="content" -A=5

# Function implementation pattern (any stdlib file)
Grep pattern="pub fn \w+\(" path="crates/atlas-runtime/src/stdlib/string.rs" output_mode="content" -A=3 head_limit=5
```

---

## Error Domain

**When:** Phase touches error handling, diagnostics.

```bash
# RuntimeError variants
Grep pattern="pub enum RuntimeError" path="crates/atlas-runtime/src/errors.rs" output_mode="content" -A=20

# Error construction pattern
Grep pattern="RuntimeError::\w+\s*\{" path="crates/atlas-runtime/src/stdlib" output_mode="content" head_limit=5
```

**Key pattern:** `RuntimeError::TypeError { msg, span }` (struct variant, NOT `::new()`)

---

## LSP Domain

**When:** Phase touches language server.

```bash
# Exported modules
Grep pattern="pub mod" path="crates/atlas-lsp/src/lib.rs" output_mode="content"

# Server capabilities
Grep pattern="ServerCapabilities" path="crates/atlas-lsp/src/server.rs" output_mode="content" -A=30

# Handler signatures
Grep pattern="async fn \w+\(&self" path="crates/atlas-lsp/src/server.rs" output_mode="content" head_limit=10
```

---

## Interpreter/VM Domain

**When:** Phase touches execution engines.

```bash
# Interpreter eval methods
Grep pattern="fn eval_" path="crates/atlas-runtime/src/interpreter" output_mode="content" head_limit=10

# VM opcodes
Grep pattern="pub enum OpCode" path="crates/atlas-runtime/src/vm" output_mode="content" -A=30

# Compiler methods
Grep pattern="fn compile_" path="crates/atlas-runtime/src/vm/compiler" output_mode="content" head_limit=10
```

---

## Verification Protocol

1. **Identify domains** - Read phase file, list all domains touched
2. **Run queries** - Execute relevant grep patterns above
3. **Extract patterns** - Note exact variant names, field names, signatures
4. **Write code** - Use ONLY verified patterns
5. **If uncertain** - Grep more. Read the actual struct/enum. Never guess.

**Cost:** ~200 tokens per domain verification
**Savings:** Prevents 10,000+ token rewrite cycles

---

## Anti-Patterns (BANNED)

```rust
// WRONG: Assumed structure
Item::ExternFunction { ... }  // Did you verify this exists?
stmt.condition                // Did you verify field name?
Type::TypeParameter(name)     // Tuple or struct variant?

// RIGHT: Verified via grep
Item::Extern { ... }          // Grep showed this is the actual name
stmt.cond                     // Grep showed field is "cond" not "condition"
Type::TypeParameter { name }  // Grep showed struct variant syntax
```

---

## Quick Reference Commands

```bash
# "What items exist in AST?"
Grep pattern="^\s+[A-Z]\w+\(" path="crates/atlas-runtime/src/ast.rs" -A=0 head_limit=30

# "What does IfStmt look like?"
Grep pattern="pub struct IfStmt" path="crates/atlas-runtime/src/ast.rs" -A=10

# "What Type variants exist?"
Grep pattern="^\s+[A-Z]\w+" path="crates/atlas-runtime/src/types.rs" head_limit=30

# "How are errors constructed?"
Grep pattern="RuntimeError::" path="crates/atlas-runtime/src" head_limit=10
```
