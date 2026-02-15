# Structured Development Workflow

**When to use:** Following documented development plan (e.g., from STATUS.md)

**Approach:** Gate-based, systematic development (implementation-driven, NOT strict TDD)

**Reference:** See `gates/README.md` for detailed gate definitions

---

## Workflow Gates

Structured development uses **GATE 0, 0.5, 1, 1.5, 2-6** from central gate workflow.

| Gate | Action | Reference |
|------|--------|-----------|
| 0 | **Read Docs** | See gates/gate-0-read-docs.md |
| 0.5 | **Check Dependencies** | See gates/gate-0.5-dependencies.md |
| 1 | **Size Estimation** | See gates/gate-1-sizing.md |
| 1.5 | **Foundation Check** | See gates/gate-1.5-foundation.md (CRITICAL) |
| 2 | **Implement + Test** | See gates/gate-2-implement.md (implementation-driven) |
| 3 | **Verify Parity** | See gates/gate-3-parity.md |
| 4 | **Quality Gates** | See gates/gate-4-quality.md |
| 5 | **Doc Update** | See gates/gate-5-docs.md |
| 6 | **Update Status** | See gates/gate-6-status.md |

---

## Structured Development Specifics

### GATE 0 Additions
**Read complete development plan:**
- Objective (what are we building?)
- Files (what to create/update?)
- Dependencies (what must exist first?)
- Implementation details (how to build?)
- Tests (what to test?)
- Acceptance criteria (how to verify?)

**Source:** STATUS.md points to current phase/plan location.

---

### GATE 0.5 Verification
**Development plan requirements:**
- All prerequisites from plan exist
- Required components implemented
- Dependencies satisfied
- No blockers

**BLOCKING:** If plan dependencies missing, STOP and report.

---

### GATE 2: Implementation-Driven Approach
**NOT strict TDD for features** - Mirrors real compilers (rustc, Go, TypeScript, Clang)

**Approach:**
1. Implement feature first (exploratory)
2. Write tests alongside or after implementation
3. Iterate: implement → test → refine

**Why:**
- Compilers require exploratory implementation
- You discover edge cases WHILE building
- Difficult to write tests before understanding the algorithm

**Tests required:** Comprehensive coverage, but NOT before implementation.

---

### GATE 6 Status Update
**Record completion:**
- Update STATUS.md or phase tracking
- Mark work complete
- Note completion date
- Document next steps if applicable

**MANDATORY for structured development.**

---

## Emergency Procedures

**If tests fail:**
- Debug systematically
- Check parity (both engines?)
- Don't skip - fix the issue
- Max 2 retry attempts at GATE 4

**If dependencies aren't met:**
- STOP immediately at GATE 0.5
- Report what's missing
- Don't implement without dependencies

**If parity fails:**
- CRITICAL issue at GATE 3
- Debug both engines
- Don't proceed without parity

**If quality gates fail:**
- Fix issues at GATE 4
- All must pass: cargo test, clippy, fmt
- Max 2 retry attempts

---

## Notes

- **Gate-based:** Each gate must pass before proceeding
- **Implementation-driven:** Build first, test alongside/after (compiler approach)
- **NOT strict TDD:** Tests can come after implementation for features
- **Parity sacred:** Both engines must match (GATE 3)
- **Quality non-negotiable:** All gates must pass (GATE 4)
- **Doc-driven:** Reference gates/README.md for details
- **Status tracking:** GATE 6 mandatory for structured development
