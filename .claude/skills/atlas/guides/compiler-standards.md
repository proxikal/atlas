# Compiler Industry Standards (NON-NEGOTIABLE)

**Atlas aims to rival Rust, Go, C#. These are table stakes:**

---

## AST & Grammar

- **AST stability** - Changes require migration plan and version bump
- **Grammar evolution** - Additive only, no breaking changes within version
- **Reference implementation = spec** - Code is source of truth

---

## Error Messages & Diagnostics

- **Helpful, actionable, span-accurate** error messages
- **Machine-readable format** - JSON dumps already implemented
- **Error recovery** - Must always produce valid AST or clear diagnostic
- **Diagnostic codes** - Every error/warning gets AT#### code

---

## Quality Bars

- **Zero platform-specific code** - Already enforced
- **Performance benchmarks** - For hot paths (parser, typechecker, VM)
- **Cross-platform guarantee** - Runs identically on macOS, Linux, Windows
- **Memory safety** - Rust guarantees this

---

## Development Standards

- **Code is documentation** - Prefer clarity over cleverness
- **Consistent patterns** - Across entire codebase
- **Follow established conventions** - Don't introduce new styles
- **Comment only complex algorithms** - Not obvious code

---

**These standards are NON-NEGOTIABLE for world-class compiler infrastructure.**
