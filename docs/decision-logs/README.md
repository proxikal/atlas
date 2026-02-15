# Decision Logs

**Purpose:** Track irreversible or high-impact technical decisions made during Atlas development.

**AI-first design:** Structured markdown optimized for Grep searches and LLM parsing.

---

## Structure

```
decision-logs/
├── README.md              # This file (template + search guide)
├── language/              # Language semantics, syntax design
├── parser/                # Parsing, grammar, AST decisions
├── typechecker/           # Type system, inference, checking
├── vm/                    # Bytecode, VM execution, optimization
├── stdlib/                # Standard library APIs, functions
├── runtime/               # Runtime model, value representation
└── tooling/               # CLI, REPL, LSP, compiler tools
```

---

## Decision File Format

**Naming:** `DR-NNN-short-title.md` (e.g., `DR-001-strict-typing.md`)

**Numbering:** Sequential per component (parser/DR-001, parser/DR-002, etc.)

**Template:**

```markdown
# DR-NNN: Decision Title

**Date:** YYYY-MM-DD
**Status:** Accepted | Superseded | Deprecated
**Component:** [Component Name]
**Supersedes:** DR-XXX (if applicable)
**Superseded By:** DR-YYY (if applicable)

## Context
Why this decision was needed (1-3 sentences)

## Decision
What was decided (clear, actionable statement)

## Rationale
Why this choice over alternatives (technical reasoning)

## Alternatives Considered
- **Option A:** Description → Why rejected
- **Option B:** Description → Why rejected

## Consequences
- ✅ **Benefits:** Positive impacts
- ⚠️  **Trade-offs:** Known compromises
- ❌ **Costs:** Negative impacts or limitations

## Implementation Notes
Key technical details for implementation (optional)

## References
- Spec: `docs/specification/X.md`
- Related: DR-XXX, DR-YYY
- External: Links to RFCs, blog posts, etc.
```

---

## Status Values

- **Accepted:** Active decision guiding current development
- **Superseded:** Replaced by newer decision (link to successor)
- **Deprecated:** No longer applicable (explain why)

---

## Search Guide (for AI)

**Find decisions by keyword:**
```bash
Grep pattern="strict typing" path="docs/decision-logs/" output_mode="files_with_matches"
```

**Find decisions by component:**
```bash
Glob pattern="docs/decision-logs/parser/*.md"
```

**Find superseded decisions:**
```bash
Grep pattern="Status: Superseded" path="docs/decision-logs/" output_mode="content"
```

**Find decisions by date range:**
```bash
Grep pattern="Date: 2026-02" path="docs/decision-logs/" output_mode="content"
```

---

## When to Log a Decision

**DO log:**
- Language semantics changes (type rules, operators, behavior)
- AST or grammar modifications
- Runtime model changes (value representation, memory)
- Security or permission model decisions
- Breaking changes requiring migration
- Architectural choices with long-term impact
- Rejected alternatives that might be reconsidered

**DON'T log:**
- Trivial implementation details (variable names, minor refactors)
- Bug fixes (unless they reveal design flaw)
- Test-only changes
- Documentation updates
- Temporary decisions or experiments

---

## Creating New Decisions

**Quick workflow:**

1. **Determine component:** Which folder? (language, parser, vm, etc.)
2. **Get next number:** Check folder for highest DR-NNN, increment
3. **Copy template:** Use format above
4. **Fill sections:** Be concise but complete
5. **Link related decisions:** Cross-reference if relevant

**Example:**

```bash
# Creating parser/DR-003-error-recovery.md
cd docs/decision-logs/parser
# Check existing: ls DR-*.md → see DR-001, DR-002
# Create DR-003
```

---

## Migrating Old Decisions

Old `docs/reference/decision-log.md` has been split into this structure.

**Migration map:**
- Language Semantics → `language/DR-001-type-system.md`
- Number Literals → `language/DR-002-scientific-notation.md`
- Runtime Model → `runtime/DR-001-value-representation.md`
- Security Architecture → `runtime/DR-002-security-context.md`
- Diagnostics → `language/DR-003-diagnostic-format.md`
- Warning Implementation → `typechecker/DR-001-unused-tracking.md`
- Prelude → `language/DR-004-prelude-design.md`
- Bytecode → `vm/DR-001-bytecode-format.md`
- JSON Support → `stdlib/DR-001-json-value.md`
- Method Call Syntax → `language/DR-005-method-syntax.md`
- Generic Types → `typechecker/DR-002-monomorphization.md`
- Array API → `stdlib/DR-002-array-intrinsics.md`

---

## Dashboard (Future)

When needed, AI can generate a dashboard:
- Parse all DR-*.md files
- Extract frontmatter (Date, Status, Component)
- Generate HTML with filters/search
- Show decision timeline and relationships

**No tooling needed upfront.** Grep + Glob work perfectly.

---

**Remember:** Decisions are permanent records. Be clear, concise, and honest about trade-offs.
