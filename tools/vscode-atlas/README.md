# Atlas VS Code Extension (local)

Local-only VS Code support for the Atlas language: syntax highlighting and LSP client wiring. Designed for in-repo use while Atlas remains private.

## Contents
- TextMate grammar: `syntaxes/atlas.tmLanguage.json`
- Language config (comments/brackets/etc.): `language-configuration.json`
- LSP client: `src/extension.ts` (launches `atlas-lsp`)
- Settings:
  - `atlas.lsp.path` (default `atlas-lsp`)
  - `atlas.lsp.trace` (`off` | `messages` | `verbose`)

## Prereqs
- Node 18+
- `atlas-lsp` binary built and on PATH (or point `atlas.lsp.path` to it)
- VS Code 1.86+
- `vsce` for packaging (use npx if you don’t want a global install)

## Build & use locally (no publishing)
```bash
cd tools/vscode-atlas
npm install        # once
npm run compile    # builds out/extension.js
npx vsce package   # produces atlas-language-support-0.0.1.vsix
# install the VSIX
code --install-extension atlas-language-support-0.0.1.vsix
```
Alternative: open this folder in VS Code and hit F5 (“Run Extension”) to launch an Extension Development Host with Atlas support.

> Private use only: do not publish this package. It’s intended for local development while Atlas remains closed.

## Expected behavior
- Syntax coloring via TextMate grammar (keywords, types, literals, builtins).
- LSP features delegated to `atlas-lsp` (hover/completion/diagnostics/etc., as implemented in the server).
- Semantic tokens will be used automatically when the server advertises them; the grammar stays minimal on purpose.

## Notes
- Activation is lazy (`onLanguage: atlas`).
- File I/O or tool execution policies stay inside the server; this client only starts the binary.
- Keep the extension version in sync with protocol changes; bump `package.json` when LSP messages change.
