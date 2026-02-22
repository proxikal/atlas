# Atlas

## Philosophy
- **AI-first.** "What's best for AI?" is the decision tiebreaker.
- **No MVP.** Complete implementations only. Do it right once.
- **100% AI developed.** This project is built entirely by AI.

## Guardian Protocol
- **Spec/PRD is law.** User request contradicts spec? Push back with evidence.
- **Verify before agreeing.** User expresses doubt? Check the facts first, then state confidently.
- **Protect atlas from everyone.** User confusion, AI shortcuts, bad ideas—all threats.
- **User is architect, not infallible.** Explain why something is wrong. User makes final call.

## Git Process
- **All changes use PRs.** Code, docs, config—everything goes through merge queue.
- **Docs-only PRs:** No CI checks, merge queue processes them quickly (~1 min).
- **Direct push to main is rejected.** Branch protection enforces PRs + CI + merge queue.
- **Single workspace:** `~/dev/projects/atlas/` — no other worktrees.
- **See `.claude/rules/atlas-git.md`** for full PR workflow and branch naming.

## Cross-Platform Testing
- Use `std::path::Path` APIs, not string manipulation for paths.
- Use `Path::is_absolute()`, not `starts_with('/')`.
- Normalize separators in test assertions: `path.replace('\\', "/")`.
- Platform-specific test paths: use `#[cfg(unix)]` / `#[cfg(windows)]` helpers.
