# When to Use atlas-dev

**atlas-dev is for AUTOMATION and TRACKING, not code development.**

---

## ✅ USE atlas-dev for:

### Phase Tracking
```bash
atlas-dev phase complete "phases/stdlib/phase-07b.md" -d "HashSet, 25 tests" -c
atlas-dev phase current     # Get current phase context
atlas-dev phase next        # Find next phase
```

**Why:** Eliminates manual STATUS.md editing, prevents sync errors

---

### Decision Logs
```bash
atlas-dev decision create --component stdlib --title "Iterator design"
atlas-dev decision list
atlas-dev decision search "hash"
```

**Why:** Auto-generates DR-XXX IDs, follows template format

---

### Progress Tracking
```bash
atlas-dev summary           # Overall progress
atlas-dev validate          # Check STATUS.md sync
atlas-dev context current   # Get everything for current phase
```

**Why:** Instant analytics, no manual counting

---

### Documentation Management
```bash
atlas-dev doc search "Result type"
atlas-dev doc index
atlas-dev feature list
```

**Why:** Fast search, structured output (JSON)

---

### Parity Validation
```bash
atlas-dev validate parity   # Check code ↔ spec ↔ docs
atlas-dev validate tests    # Verify test counts
```

**Why:** Catches drift between docs and implementation

---

## ❌ DO NOT use atlas-dev for:

### Writing Code
```bash
# WRONG: atlas-dev create-function "foo"
# RIGHT: Use Write/Edit tools directly
```

### Editing Source Files
```bash
# WRONG: atlas-dev edit src/value.rs
# RIGHT: Use Edit tool
```

### Running Tests
```bash
# WRONG: atlas-dev test
# RIGHT: cargo test -p atlas-runtime
```

### Building Project
```bash
# WRONG: atlas-dev build
# RIGHT: cargo build
```

### Git Operations (except phase completion)
```bash
# WRONG: atlas-dev commit -m "msg"
# RIGHT: git commit -m "msg"
```

---

## 🎯 Decision Tree

```
Need to complete phase?
  ├─ YES → atlas-dev phase complete
  └─ NO  → Continue

Need to track progress?
  ├─ YES → atlas-dev context current OR atlas-dev summary
  └─ NO  → Continue

Need to create decision log?
  ├─ YES → atlas-dev decision create
  └─ NO  → Continue

Need to validate docs/spec/code sync?
  ├─ YES → atlas-dev validate parity
  └─ NO  → Continue

Need to write/edit code?
  ├─ YES → Use Write/Edit tools (NOT atlas-dev)
  └─ NO  → Continue
```

---

## 💡 Common Workflows

### Completing a Phase
```bash
# 1. Get context
atlas-dev context current

# 2. [Do implementation work with Write/Edit tools]

# 3. Run tests
cargo test -p atlas-runtime

# 4. Mark complete
atlas-dev phase complete "phases/stdlib/phase-07b.md" \
  --desc "HashSet with 25 tests, 100% parity" \
  --commit
```

### Creating a Decision Log
```bash
# 1. Check next ID
atlas-dev decision next-id stdlib

# 2. Create log
atlas-dev decision create \
  --component stdlib \
  --title "Hash function design" \
  --context "Need consistent hashing for HashMap"
```

### Validating Everything
```bash
# 1. Validate STATUS.md sync
atlas-dev validate

# 2. Validate parity
atlas-dev validate parity

# 3. Validate test counts
atlas-dev validate tests
```

---

## 🚀 Key Principle

**atlas-dev handles BOOKKEEPING, you handle CODING.**

- Phase tracking → atlas-dev
- Decision logs → atlas-dev
- Progress stats → atlas-dev
- Parity validation → atlas-dev
- **Code implementation → Write/Edit/Read/Grep tools**

---

## 📊 Token Efficiency

| Task | Manual (tokens) | atlas-dev (tokens) | Savings |
|------|-----------------|--------------------| --------|
| Complete phase | ~350 | ~120 | 66% |
| Get context | ~200 | ~80 | 60% |
| Create decision log | ~180 | ~100 | 44% |
| Validate sync | ~150 | ~60 | 60% |

**Over 78 phases: ~18,000 tokens saved**

---

## ⚡ Quick Reference

```bash
# Most common commands
atlas-dev phase complete <path> -d "..." -c   # Complete phase
atlas-dev context current                      # Get phase context
atlas-dev validate                             # Validate sync
atlas-dev decision create                      # Create decision log

# Full command list
atlas-dev --help
```
