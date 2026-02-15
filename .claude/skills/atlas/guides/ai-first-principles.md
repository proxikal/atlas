# AI-First Language Principles (Atlas's Competitive Edge)

**Atlas is built AFTER AI, optimized FOR AI. This is our differentiator.**

---

## Design for AI Agents

- **Token-efficient error messages** - LLMs pay per token (be concise but helpful)
- **Predictable syntax** - AI can generate correct code without trial-and-error
- **Explicit over implicit** - No magic; AI must reason about behavior
- **Clear diagnostic format** - JSON for AI consumption (already implemented)
- **Consistent patterns** - AI learns once, applies everywhere

---

## Error Recovery for AI

- **Errors suggest fixes** - Not just "syntax error"
- **Error messages parseable by LLMs** - Structured, actionable
- **Recovery makes sense to AI code generators** - Predictable patterns
- **Span information precise** - AI can fix exact location

---

## When Designing Features

**Ask these questions:**

- How will AI agents use this? (code generation perspective)
- Can AI understand the error messages? (parse and fix)
- Is the syntax predictable? (minimal ambiguity)
- Does it compose well? (AI can combine features)
- Is it explicit? (no hidden behavior AI must discover)

---

## Examples of AI-First Decisions

- **Explicit types** - AI doesn't guess
- **No truthiness** - AI knows exactly what's bool
- **Strict equality** - AI knows exactly what's comparable
- **Function types explicit** - AI sees signatures clearly
- **Error codes** - AI can categorize and handle systematically

---

**This is what makes Atlas special - built for the AI-native development era.**
