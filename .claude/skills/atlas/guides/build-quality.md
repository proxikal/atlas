# Build & Quality Commands

## During Development (ALWAYS)

```bash
cargo clean && cargo check -p atlas-runtime       # Clean + verify compilation
cargo clippy -p atlas-runtime -- -D warnings      # Zero warnings
cargo fmt -p atlas-runtime                        # Format code
cargo test -p atlas-runtime test_exact_name -- --exact  # ONE test ONLY
```

---

## NEVER During Development

```bash
cargo test                    # NO - full suite (1400+ tests)
cargo test -p atlas-runtime   # NO - package suite
cargo test --test file_tests  # NO - full test file
```

---

## End of Phase Only (GATE 4)

**User will tell you when to run full suite:**

```bash
cargo test -p atlas-runtime   # Full suite ONE time
```

---

**See guides/testing-protocol.md for complete testing rules.**
