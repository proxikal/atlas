# Atlas Language Comparison Notes

## Purpose
Document how Atlas borrows ideas from TypeScript, Go, Rust, and Python while staying cohesive.

## Structure
Feature | Inspiration | Atlas Decision | Rationale

## Draft Entries
- Strict typing | TypeScript | Explicit parameter/return types, no implicit any | Predictability and AI-friendliness
- Simple concurrency | Go | Planned `spawn` + `chan<T>` | Clear mental model
- Error diagnostics | Rust | Precise spans and structured diagnostics | High-quality tooling
- Readability | Python | Low-ceremony syntax | Natural for humans/AI

## Notes
- Atlas avoids overly clever syntax or implicit behaviors.
