# Atlas Documentation

**Welcome to the Atlas programming language documentation.**

This documentation is organized for both AI agents and human developers working on Atlas.

---

## 📁 Documentation Structure

### [`specification/`](specification/) - Language Specification
Core language definition and semantics
- `grammar-conformance.md` - Parser conformance requirements
- `language-semantics.md` - Type rules, operators, execution semantics
- `runtime-spec.md` - Value model, bytecode format, execution model
- `diagnostic-system.md` - Error codes, warnings, diagnostic format
- `json-formats.md` - AST/typecheck dump formats

### [`philosophy/`](philosophy/) - Project Philosophy
Why Atlas exists and how we approach development
- `ai-manifesto.md` - AI-first language design principles
- `documentation-philosophy.md` - How we document Atlas
- `why-strict.md` - Type system philosophy
- `ai-principles.md` - AI agent development principles

### [`implementation/`](implementation/) - Implementation Guides
How to implement each compiler component (**detailed, comprehensive**)
- `01-project-structure.md` - Codebase organization
- `02-core-types.md` through `16-lsp.md` - Component guides
- `13-stdlib.md` - Standard library implementation

### [`guides/`](guides/) - Development Guides
Practical guides for working on Atlas
- `ai-workflow.md` - AI agent development workflow
- `ai-agent-checklist.md` - Checklist for AI agents
- `testing-guide.md` - Test infrastructure and patterns
- `code-quality-standards.md` - Code standards and phase gates

### [`api/`](api/) - API Reference
Function signatures and usage
- `stdlib.md` - Standard library API reference
- `runtime-api.md` - Runtime embedding API

### [`decision-logs/`](decision-logs/) - Architecture Decisions
Permanent record of high-impact technical decisions (AI-optimized structure)
- `README.md` - Template, format, search guide
- `language/` - Type system, syntax, semantics decisions
- `parser/` - Parsing and AST decisions
- `typechecker/` - Type checking and inference decisions
- `vm/` - Bytecode and VM execution decisions
- `stdlib/` - Standard library API decisions
- `runtime/` - Runtime model and security decisions
- `tooling/` - CLI, REPL, LSP decisions

### [`reference/`](reference/) - Technical Reference
Policies and technical details
- `parser-recovery-policy.md` - Error recovery strategy
- `io-security-model.md` - I/O security and sandboxing
- `code-organization.md` - Project structure and conventions
- `versioning.md` - Version numbering and compatibility
- `decision-log.md` - Architecture decisions (DEPRECATED - see `decision-logs/`)

### [`config/`](config/) - Configuration
Configuration formats and options
- `cli-config.md` - CLI configuration
- `repl-modes.md` - REPL behavior modes

### [`features/`](features/) - Feature Documentation
**Populated by phases as features are implemented**

---

## 🎯 Quick Start

**For AI Agents:**
1. Read `Atlas-SPEC.md` (root) for language overview
2. Read `STATUS.md` (root) for current phase and progress
3. Reference `implementation/` guides for component details
4. Follow phase files in `phases/` for implementation tasks

**For Humans:**
1. Start with `philosophy/ai-manifesto.md` to understand Atlas
2. Read `Atlas-SPEC.md` for language specification
3. Reference `implementation/` for technical details
4. See `STATUS.md` for current development state

---

## 📖 Root Documentation Files

- **`Atlas-SPEC.md`** - Complete language specification
- **`STATUS.md`** - Current development status and progress
- **`CONTRIBUTING.md`** - Contribution guidelines (if exists)

---

## 🤖 For AI Agents

**Phase Execution:**
- Start with `STATUS.md` for current phase
- Read phase file from `phases/` directory
- Reference docs from appropriate subdirectories
- Follow BLOCKERS → Implementation → Tests → Acceptance

**Documentation Pattern:**
- `specification/` = WHAT the language is
- `implementation/` = HOW to build it
- `guides/` = PRACTICES for building it
- `api/` = USING what was built

**Development Workflow (Atlas Skill):**
- **Skill Definition:** `/.claude/skills/atlas/skill.md` - Project overview, roles, rules
- **Gate System:** `/.claude/skills/atlas/gates/` - Workflow gates (-1 through 6)
- **Workflows:** `/.claude/skills/atlas/workflows/` - Structured dev, bug-fix, refactoring, enhancement, debugging, dead-code cleanup
- **Gate Index:** `/.claude/skills/atlas/gates/README.md` - Complete gate reference

**Key Rules:**
- **Always check STATUS.md first** - Shows current phase and doc map
- **Run GATE -1 before any work** - Communication & sanity check
- **Reference implementation/ guides** - Detailed component architecture
- **Maintain interpreter/VM parity** - Critical requirement
- **Follow phase file guidance** - Phases specify which docs to use

---

## 📊 Documentation Philosophy

Atlas documentation follows these principles:
1. **AI-first** - Optimized for AI agent consumption
2. **Token-efficient** - Concise but complete
3. **No code dumps** - Guidance and patterns, not full implementations
4. **Evolves with project** - Docs grow as features are added
5. **Single source of truth** - No duplication between docs

See `philosophy/documentation-philosophy.md` for complete philosophy.

---

## 🔍 Finding Documentation

**Looking for...**
- Language syntax/semantics → `specification/`
- How to implement a feature → `implementation/`
- Testing/quality standards → `guides/`
- Function signatures → `api/`
- Project decisions → `decision-logs/` (organized by component)
- Configuration options → `config/`

---

**Atlas is an AI-first programming language. Our documentation reflects this principle.**
