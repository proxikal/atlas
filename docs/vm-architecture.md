# Atlas VM Architecture

## Overview

The Atlas Virtual Machine is a stack-based bytecode interpreter that executes compiled Atlas programs. It operates alongside the tree-walking interpreter, maintaining 100% semantic parity.

## Architecture Layers

```
┌─────────────────────────────────────────┐
│            Atlas Source Code             │
├─────────────────────────────────────────┤
│  Lexer → Parser → Binder → TypeChecker │
├──────────────────┬──────────────────────┤
│   Interpreter    │      Compiler        │
│  (tree-walking)  │    ↓ Bytecode        │
│                  │   [Optimizer]         │
│                  │    ↓ Optimized BC     │
│                  │      VM Engine        │
│                  │   [Profiler]          │
│                  │   [Debugger]          │
└──────────────────┴──────────────────────┘
```

## Bytecode Format

### Instruction Encoding

Instructions are variable-length, consisting of a 1-byte opcode followed by 0 or more operand bytes.

### Core Opcodes

| Category | Opcodes | Description |
|----------|---------|-------------|
| Constants | `Constant` | Push constant from pool (u16 index) |
| Literals | `True`, `False`, `Null` | Push literal values |
| Arithmetic | `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate` | Stack-based arithmetic |
| Comparison | `Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual` | Push boolean result |
| Logic | `Not`, `And`, `Or` | Boolean operations |
| Variables | `GetLocal`, `SetLocal`, `GetGlobal`, `SetGlobal` | Variable access (u16 slot) |
| Control | `Jump`, `JumpIfFalse`, `JumpIfTrue` | Unconditional/conditional jumps (i16 offset) |
| Functions | `Call`, `Return` | Function call/return |
| Arrays | `NewArray`, `GetIndex`, `SetIndex` | Array operations |
| Stack | `Pop`, `Dup` | Stack management |
| Control | `Halt` | Stop execution |

### Constant Pool

Constants are stored in a vector and referenced by u16 index. Supports:
- Numbers (f64)
- Strings (Arc<String>)
- Booleans
- Null
- Function references

### Debug Information

Each instruction records its source `Span` for:
- Error reporting with source locations
- Debugger source mapping
- Profiler hotspot identification

## Execution Model

### Stack Machine

The VM uses a value stack for all computation:

```
// Computing: (1 + 2) * 3
Constant 1    → stack: [1]
Constant 2    → stack: [1, 2]
Add           → stack: [3]
Constant 3    → stack: [3, 3]
Mul           → stack: [9]
```

### Call Frames

Function calls create call frames on a separate frame stack:

```
struct CallFrame {
    return_ip: usize,        // Return address
    base_pointer: usize,     // Stack base for locals
    function_ref: FunctionRef // Function metadata
}
```

### Local Variables

Locals are stored on the value stack, addressed relative to the current frame's base pointer:
- `GetLocal(n)` → push `stack[base_pointer + n]`
- `SetLocal(n)` → `stack[base_pointer + n] = pop()`

## Optimizer

### Optimization Passes

The optimizer operates on compiled bytecode before execution:

1. **Constant Folding**: Evaluates compile-time constant expressions
   - `Constant(2) Constant(3) Add` → `Constant(5)`

2. **Dead Code Elimination**: Removes unreachable code after unconditional jumps

3. **Peephole Optimization**: Pattern-based local optimizations
   - Redundant push/pop elimination
   - Jump chain simplification

### Optimization Levels

| Level | Passes | Use Case |
|-------|--------|----------|
| 0 | None | Debugging (unmodified bytecode) |
| 1 | Constant folding | Default |
| 2 | All passes | Production |

### Statistics

The optimizer tracks:
- Original vs optimized bytecode size
- Number of optimizations applied per pass
- Bytes saved (reduction percentage)

## Profiler

### Data Collection

The profiler records during execution:
- **Instruction counts**: Per-opcode execution frequency
- **Hotspot detection**: Instructions executed most frequently (by IP)
- **Stack depth**: Maximum call frame and value stack depth
- **Function calls**: Named function call counts
- **Timing**: Wall-clock execution time

### Reports

```rust
let report = profiler.generate_report(0.1); // 10% hotspot threshold
// Returns: ProfileReport {
//   total_instructions, elapsed_secs,
//   max_frame_depth, max_value_stack_depth,
//   hotspots, top_opcodes
// }
```

## Debugger

### Protocol

The debugger uses a request/response protocol:

| Request | Response | Description |
|---------|----------|-------------|
| `SetBreakpoint { location }` | `BreakpointSet { breakpoint }` | Set breakpoint at source location |
| `RemoveBreakpoint { id }` | `BreakpointRemoved` | Remove breakpoint |
| `Continue` | `Paused` / `Stopped` | Continue execution |
| `StepInto` | `Paused` | Step into function calls |
| `StepOver` | `Paused` | Step over function calls |
| `StepOut` | `Paused` | Step out of current function |

### Source Mapping

Bidirectional mapping between bytecode offsets and source locations:
- Forward: bytecode offset → (file, line, column)
- Reverse: (file, line) → bytecode offset (for breakpoints)

### State Management

```
ExecutionMode: Running | Paused | Stopped
StepMode: Into | Over | Out | None
PauseReason: Breakpoint | Step | UserRequest
```

## Performance Optimizations (Phase 06)

### Dispatch Table

Static O(1) opcode dispatch using a lookup table instead of match-based dispatch.

### Hot Path Inlining

`#[inline(always)]` on frequently called helpers:
- Stack push/pop
- Local variable access
- Arithmetic operations

### Stack Management

- Pre-allocated stack capacity (1024 slots)
- `truncate()` for O(1) stack cleanup on function return
- Reusable string buffer for concatenation

## Testing Strategy

### Test Categories

1. **Unit tests**: Individual component tests (optimizer passes, profiler collectors)
2. **Integration tests**: Cross-component interaction (optimizer + debugger)
3. **Parity tests**: Interpreter-VM result comparison
4. **Regression tests**: v0.1 program compatibility
5. **Performance tests**: Execution time bounds
6. **Complex programs**: Real-world algorithm implementations

### Running Tests

```bash
# VM test domain file (post-consolidation: all VM tests are in tests/vm.rs)
cargo nextest run -p atlas-runtime --test vm

# Specific test
cargo nextest run -p atlas-runtime -E 'test(test_name)'
```

## File Layout

```
crates/atlas-runtime/src/
├── vm/
│   ├── mod.rs        # VM engine, execution loop
│   ├── frame.rs      # Call frame management
│   ├── dispatch.rs   # Opcode dispatch table
│   └── profiler.rs   # VM-integrated profiling
├── compiler/
│   └── mod.rs        # AST → bytecode compilation
├── optimizer/
│   └── mod.rs        # Bytecode optimization passes
├── profiler/
│   ├── mod.rs        # Profiler facade
│   ├── collector.rs  # Data collection
│   └── report.rs     # Report generation
├── debugger/
│   ├── mod.rs        # DebuggerSession
│   ├── protocol.rs   # Request/Response types
│   ├── breakpoints.rs # Breakpoint management
│   ├── source_map.rs # Offset ↔ location mapping
│   ├── state.rs      # Execution state
│   ├── stepping.rs   # Step tracking
│   └── inspection.rs # Variable inspection
└── bytecode/
    └── mod.rs        # Bytecode types, validation
```
