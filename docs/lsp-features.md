# Atlas LSP Features

**Version:** 0.2 | **Status:** Production Ready

The Atlas Language Server provides comprehensive IDE support for the Atlas programming language through the Language Server Protocol (LSP).

---

## Overview

Atlas LSP implements the Language Server Protocol to provide rich language features in any LSP-compatible editor, including VS Code, Neovim, Emacs, and others.

**Protocol Version:** LSP 3.17
**Transport:** stdio (default), TCP (optional)

---

## Implemented Features

### 1. Hover Information

**Capability:** `hoverProvider`
**Method:** `textDocument/hover`

Displays type information, documentation, and signature details when hovering over code elements.

**What you get:**
- Function signatures with parameter types and return types
- Variable types
- Inline documentation from comments
- Builtin function documentation

**Example:**
```atlas
fn calculate(x: number) -> number { return x * 2; }
//  ^hover here shows: fn calculate(x: number) -> number
```

**Performance:** < 100ms response time

---

### 2. Semantic Syntax Highlighting

**Capability:** `semanticTokensProvider`
**Method:** `textDocument/semanticTokens/full`

Provides accurate,Human: continue