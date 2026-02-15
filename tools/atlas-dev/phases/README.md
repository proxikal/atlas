# atlas-dev - Implementation Phases

**Purpose:** Break atlas-dev implementation into manageable, sequential phases.

**Tool Name:** `atlas-dev` (unified development tool, no conflict with `atlas` compiler)

**Each phase:**
- Complete, deliverable unit
- ~2-8 hours of work
- Builds on previous phases
- Has clear acceptance criteria

---

## Phase Sequence

| Phase | Focus | Priority |
|-------|-------|----------|
| [01](phase-01-core-infrastructure.md) | Core Infrastructure | CRITICAL |
| [02](phase-02-phase-management.md) | Phase Management | CRITICAL |
| [03](phase-03-decision-log-integration.md) | Decision Logs | HIGH |
| [04](phase-04-progress-analytics.md) | Analytics & Validation | HIGH |
| [05](phase-05-documentation-context.md) | Docs & Context | MEDIUM |
| [06](phase-06-polish-advanced.md) | Polish & Advanced | LOW |
| [07](phase-07-feature-management.md) | Feature Management | HIGH |
| [08](phase-08-spec-api-management.md) | Spec & API Management | HIGH |
| [09](phase-09-parity-validation.md) | Parity Validation | CRITICAL |
| [10](phase-10-composability.md) | Composability & Piping | MEDIUM |

**Note:** Complete all phases sequentially. No BS timeline estimates - just build it.

---

## Prerequisites

**Before starting:**
- Go 1.22+ installed
- Git configured
- Atlas repository at known location
- Familiarity with Go CLI development (cobra)

---

## Workflow

### 1. Read Phase File
```bash
cat tools/atlas-dev/phases/phase-01-core-infrastructure.md
```

### 2. Implement Phase
Follow the implementation steps in the phase file.

### 3. Test Phase
Run tests specified in acceptance criteria.

### 4. Mark Complete
Update this README with completion status.

---

## Completion Tracking

### Phase 1: Core Infrastructure
- [ ] Project structure created
- [ ] CLI framework (cobra) integrated
- [ ] Config system implemented
- [ ] Error handling framework
- [ ] Test infrastructure setup
- [ ] `atlas-dev version` works

### Phase 2: Phase Management
- [ ] Phase path parser implemented
- [ ] Tracker file reader/writer
- [ ] STATUS.md reader/writer
- [ ] Percentage calculator
- [ ] Sync validator
- [ ] Git commit automation
- [ ] `atlas-dev phase complete` works end-to-end

### Phase 3: Decision Log Integration
- [ ] Decision log parser
- [ ] Next ID calculator
- [ ] Template generator
- [ ] Search indexer
- [ ] `atlas-dev decision create` works
- [ ] `atlas-dev decision list` works

### Phase 4: Progress Analytics
- [ ] Progress calculator
- [ ] Blocker analyzer
- [ ] Test coverage tracker
- [ ] `atlas-dev summary` works
- [ ] `atlas-dev validate` comprehensive

### Phase 5: Documentation & Context
- [ ] Doc indexer
- [ ] Doc search
- [ ] Context aggregator
- [ ] `atlas-dev context current` works

### Phase 6: Polish & Advanced
- [ ] Undo/redo system
- [ ] Export functionality
- [ ] Cache system
- [ ] Pre-commit hooks
- [ ] Human mode output

---

## Completion Tracking (NEW - Expanded)

### Phase 7: Feature Management
- [ ] Feature doc parser implemented
- [ ] Feature CRUD operations
- [ ] Feature validation (against code/spec)
- [ ] Feature sync (auto-update from code)
- [ ] `feature create/list/read/update/validate` work

### Phase 8: Spec & API Management
- [ ] Spec parser (markdown + EBNF)
- [ ] API doc parser
- [ ] Grammar validator (EBNF)
- [ ] API validator (against code)
- [ ] API generator (from code)
- [ ] `spec` and `api` commands work

### Phase 9: Parity Validation
- [ ] Code analyzer (parse Rust)
- [ ] Spec matcher (spec → code)
- [ ] API matcher (API docs → code)
- [ ] Test analyzer (coverage)
- [ ] Cross-reference validator
- [ ] `validate parity` works (KILLER FEATURE)
- [ ] `validate all` comprehensive

### Phase 10: Composability
- [ ] Stdin input support (--stdin)
- [ ] JSON streaming (piping)
- [ ] Batch operations (xargs)
- [ ] Parallel execution
- [ ] Progress reporting
- [ ] All commands composable

---

## Current Status

**Last Updated:** 2026-02-15
**Completed:** 0/10 phases
**In Progress:** None
**Next:** Phase 1 (Core Infrastructure)

---

## Build Strategy

### MVP (Phases 1-2): Core Phase Tracking
**Delivers:**
- Phase tracking (`phase complete`, `phase current`, `phase next`)
- STATUS.md/tracker automation
- Validation (`validate`)
- 99.8% success rate on phase completions

**Use immediately for remaining Atlas phases.**

### Full Feature Set (Phases 1-6): Original Vision
**Adds:**
- Decision logs (`decision create`, `decision list`, `decision search`)
- Analytics, stats, blockers
- Context system (`context current`)
- Undo, export, cache
- Human mode output

**Complete original automation vision.**

### Unified Platform (Phases 1-10): World-Class
**Adds:**
- Feature management (docs/features/ CRUD)
- Spec/API management (docs/specification/, docs/api/)
- Parity validation (code ↔ spec ↔ docs ↔ tests) - **KILLER FEATURE**
- Composability (piping, chaining, batch operations)

**World-class unified development platform.**

---

## Notes

- Phases must be done in order (dependencies)
- Each phase is independently testable
- Phases 1-2 = MVP (6-9h)
- Phases 3-6 = Original features (12-16h)
- Phases 7-10 = Docs management + parity (18-22h)
- Can defer Phases 7-10 until after v0.2 if time-constrained
- Focus on Phases 1-2 first for immediate ROI
