# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CI/CD automation with GitHub Actions
- Benchmark workflow with performance regression detection
- Automated release workflow with multi-platform binaries
- Security audit workflows (daily audits + supply chain checks)
- Dependabot configuration for automated dependency updates

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.2.0] - TBD

### Added

**Foundation Infrastructure:**
- Runtime API expansion with conversion traits (ToAtlasValue, FromAtlasValue)
- Embedding API for native Rust functions with sandboxing support
- Configuration system with atlas.toml and global config
- Module system with imports, exports, and dependency resolution
- Error propagation operator (?) with comprehensive error handling
- Complete FFI system (extern types, library loading, callbacks)
- Package manifest system (atlas.toml, atlas.lock, validation)
- Security & permissions model (capability-based security, sandbox enforcement, policy system)
- Method call syntax for types (.method() notation)

**Standard Library:**
- Complete string API (18 functions: trim, split, replace, substring, etc.)
- Complete array API (21 functions: map, filter, reduce, find, sort, etc.)
- Complete math API (18 functions + 5 constants: trigonometry, logarithms, rounding, etc.)
- JSON type utilities (17 functions: parsing, validation, manipulation, etc.)
- Complete file I/O API (10 functions: read, write, append, delete, exists, etc.)

**Type System:**
- First-class functions with function types `(T) -> U`
- Generic types: Option<T>, Result<T,E>
- Pattern matching on types and values

**Testing:**
- 1,500+ comprehensive tests across all components
- 100% interpreter/VM parity maintained
- Property-based testing with proptest
- Snapshot testing with insta

### Changed
- Improved error messages with better context and suggestions
- Enhanced type inference for complex expressions
- Optimized bytecode VM performance

### Fixed
- Various edge cases in type checking
- Memory safety issues in FFI layer
- Parser recovery on malformed input

## [0.1.0] - 2026-02-13

### Added
- Initial language implementation
- Lexer, parser, and AST
- Type checker with basic inference
- Tree-walking interpreter
- Bytecode compiler and VM
- REPL with syntax highlighting
- Basic standard library (5 functions)
- Comprehensive test suite (1,391 tests)
- Documentation and examples

[Unreleased]: https://github.com/proxikal/atlas/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/proxikal/atlas/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/proxikal/atlas/releases/tag/v0.1.0
