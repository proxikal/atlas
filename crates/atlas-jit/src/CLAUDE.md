# atlas-jit/src/

Cranelift-based JIT compiler. Wires into the VM's hotspot profiler.
Block 7 (v0.3) completes this crate — adds control flow + wires to VM.

## File Map

| File | What it does |
|------|-------------|
| `lib.rs` | `JitEngine`, `JitConfig`, `JitError`, `JitStats` — public API |
| `hotspot.rs` | `HotspotTracker` — counts function calls, identifies compilation candidates |
| `codegen.rs` | Cranelift IR generation — translates Atlas bytecode → native via Cranelift |
| `cache.rs` | Compiled function cache — maps bytecode offset → native function pointer |
| `backend.rs` | Cranelift backend setup, module configuration |

## Current State (Block 1 complete, Block 7 pending)

**Supported opcodes** (already implemented in codegen.rs):
`Constant`, `True`, `False`, `Null`, `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate`,
`Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`, `Not`,
`Pop`, `Return`

**Unsupported opcodes** (bail out to interpreter — Block 7 adds these):
`GetGlobal`, `SetGlobal`, `Jump`, `JumpIfFalse`, `Loop`, `Call`, `And`, `Or`,
`GetLocal`, `SetLocal` and all collection/closure opcodes

**Threshold:** Default 1000 invocations → compilation triggered.
**Not wired to VM yet** — `JitEngine` exists but VM doesn't call it. Block 7 wires it.

## Block 7 Scope (what gets added)

1. `Jump`, `JumpIfFalse`, `Loop` opcodes in `codegen.rs` — enables loop compilation
2. `Call` opcode — indirect dispatch to compiled or interpreted functions
3. `GetGlobal`/`SetGlobal` — access VM's global value array via pointer
4. `And`/`Or` short-circuit via Cranelift conditional blocks
5. Wire `hotspot.rs` threshold check into VM execution loop
6. Replace interpreter loop for hot functions with native function pointer
7. JIT cache invalidation on bytecode change (REPL support)

## Key Types

- `JitEngine` — top-level, holds tracker + cache + config
- `JitConfig` — `compilation_threshold: u64` (default 1000)
- `HotspotTracker` — `record_call(offset)` → `should_compile(offset) -> bool`
- `JitResult<T>` = `Result<T, JitError>`
- `JitError::UnsupportedOpcode(Opcode)` — graceful fallback signal

## Critical Rules

**Graceful fallback is required.** Any unsupported opcode must return
`Err(JitError::UnsupportedOpcode(...))` — never panic. The VM falls back to interpreted
execution on this error. This invariant must hold after every Block 7 phase.

**Parity with interpreter.** JIT output must be identical to interpreter output for all
supported opcodes. JIT is an optimization — it must never change observable behavior.

**No JIT in tests by default.** atlas-runtime tests run interpreted. JIT-specific tests
live in `crates/atlas-jit/` and test the JIT engine directly.

## Tests

Tests live in `crates/atlas-jit/` (no separate tests/ dir — inline in src or adjacent).
Run with: `cargo nextest run -p atlas-jit`
