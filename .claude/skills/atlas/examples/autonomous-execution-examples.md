# Autonomous Execution Examples

**These examples show CORRECT vs INCORRECT behavior in EXECUTION MODE.**

---

## ❌ BAD: Asking Permission When Direction Is Clear

```
User: [Pastes handoff: "Next: Phase-06c (Benchmarks + Docs)"]

AI: "⏺ Phase-06b completion acknowledged. Excellent work!

     Ready for direction. What's next?"
```

**Problem:** Handoff already said "Next: Phase-06c" - asking "what's next?" is redundant and confusing.

---

## ✅ GOOD: Immediate Execution

```
User: [Pastes handoff: "Next: Phase-06c (Benchmarks + Docs)"]

AI: "⏺ GATE -1: Spec-First Verification
     Phase-06c: Stdlib Benchmarks + Documentation

     Reading phases/stdlib/phase-06c-stdlib-performance-docs.md..."

[AI executes autonomously through all gates]

AI: "⏺ ✅ Phase-06c Complete: Stdlib Benchmarks + Documentation

     Delivered: [handoff following template]"
```

**Result:** Immediate execution, zero permission requests, seamless handoff.

---

## ✅ GOOD: Intelligent Oversight (Catching User Mistakes)

```
User: "Start Phase-06c"

AI: "⚠️ STATUS.md shows Phase-06c already complete (100% passing tests).

     Did you mean:
     - Phase-06d (next incomplete phase)
     - Re-run Phase-06c tests
     - Review Phase-06c implementation"
```

**Result:** AI catches user's mistake, provides options.

---

## ✅ GOOD: Making Intelligent Decision When Spec Silent

```
AI: "⏺ GATE 1.5: Foundation Check

     Spec doesn't define error message format for FFI boundary violations.

     Analyzing codebase patterns:
     - Runtime errors use: AT#### codes + context
     - VM errors use: detailed stack traces
     - FFI is runtime boundary → use AT#### pattern

     Decision: FFI errors will use AT9xxx codes with boundary context.

     Logging to docs/decision-logs/..."
```

**Result:** AI decides intelligently based on codebase patterns, documents decision, doesn't ask user.

---

## ❌ BAD: Asking When Spec Defines It

```
AI: "The spec says Result type should use Ok/Err variants.
     Should I implement it that way?"
```

**Problem:** Spec already defines it - just implement it. Don't ask.

---

## ✅ GOOD: Following Spec Exactly

```
AI: "⏺ GATE 2: Implementation

     Spec (docs/specification/types.md section 4.2) defines Result<T, E>:
     - Ok(value: T) variant
     - Err(error: E) variant
     - Explicit type parameters

     Implementing exactly as specified..."
```

**Result:** Follows spec without asking permission.

---

## Execution Triggers (All Mean START NOW)

| User Input | Mode | AI Response |
|------------|------|-------------|
| "Next: Phase-06c" | EXECUTION | Start GATE -1 immediately |
| "Start Phase-06c" | EXECUTION | Start GATE -1 immediately |
| "Do Phase-06c" | EXECUTION | Start GATE -1 immediately |
| [Pastes handoff] | EXECUTION | Start GATE -1 immediately |
| "Let's design Phase-20" | DESIGN | Collaborate on design |

---

## Key Takeaways

1. **Phase directive = execution order** (not request for permission)
2. **Handoff contains next phase** (don't ask "what's next?")
3. **Spec defines behavior** (implement it, don't ask)
4. **Spec silent** (analyze codebase, decide, log decision)
5. **User mistake** (catch it, provide options)
