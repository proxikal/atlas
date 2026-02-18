# Build & Quality Commands

## During Development

```bash
cargo clean && cargo check -p atlas-runtime                          # Verify
cargo clippy -p atlas-runtime -- -D warnings                        # Zero warnings
cargo fmt -p atlas-runtime                                          # Format
cargo nextest run -p atlas-runtime -E 'test(exact_name)'            # ONE test
cargo nextest run -p atlas-runtime --test <domain_file>             # Domain file
```

## Before Handoff (GATE 6)

```bash
cargo nextest run -p atlas-runtime                                   # Full suite (~15-20s)
cargo clippy -p atlas-runtime -- -D warnings                        # Zero warnings
```

---

**See `memory/testing-patterns.md` for complete testing rules, domain file list, and corpus workflow.**
