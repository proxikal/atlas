# GATE 1: Size Estimation (Compiler-Aware)

**Condition:** Dependencies verified

---

## Action

1. List all functions/features/components needed
2. Estimate lines per item
3. Sum total per file
4. Add 20% buffer
5. **DECLARE estimate in output**

---

## Estimation Format (MANDATORY)

```
File: src/compiler/parser.rs
- Feature 1: ~X lines
- Feature 2: ~Y lines
- Error handling: ~Z lines
Total: ~N lines
Buffered (×1.2): ~M lines
```

---

## Decision (Compiler-Aware)

- <800 buffered → Single implementation → GATE 1.5
- 800-1000 buffered → Acceptable, proceed → GATE 1.5
- 1000-1500 buffered → Justify complexity OR plan split → GATE 1.5
- >1500 buffered → MUST justify OR split → GATE 1.5

---

## Compiler Reality

- Simple modules: 200-600 lines
- Standard modules: 600-1000 lines
- Complex modules: 1000-2000 lines (VM, bytecode, parser)

---

## Split Planning (if needed)

1. Define module boundaries
2. List what goes in each file
3. Estimate lines per file
4. Document split plan

---

**CRITICAL:** Quality over line counts. NEVER simplify compiler logic for arbitrary limits.

**Next:** GATE 1.5
