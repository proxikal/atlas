# Atlas LSP - VS Code Setup

Set up the Atlas Language Server in Visual Studio Code for full IDE support.

---

## Quick Start

### 1. Install Atlas CLI

```bash
# From repository
cargo install --path crates/atlas-cli

# Verify installation
atlas --version
```

### 2. Configure VS Code

Create `.vscode/settings.json` in your workspace:

```json
{
  "atlas.lsp.enable": true,
  "atlas.lsp.path": "atlas"
}
```

### 3. Install Language Extension (Future)

Currently manual setup. Future: install from VS Code Marketplace.

---

## Manual LSP Configuration

Add to your VS Code `settings.json`:

```json
{
  "languageServerSettings": {
    "atlas": {
      "command": "atlas",
      "args": ["lsp"],
      "filetypes": ["atlas", "atl"],
      "rootPatterns": ["atlas.toml", ".git"]
    }
  }
}
```

---

## Features Available

Once configured, you'll have:

- ✅ **Syntax Highlighting** - Semantic tokens with accurate coloring
- ✅ **Hover Information** - Type info and documentation on hover
- ✅ **Code Actions** - Quick fixes and refactorings
- ✅ **Symbol Navigation** - Go to symbol, outline view
- ✅ **Folding** - Collapse/expand functions and blocks
- ✅ **Inlay Hints** - Inline type annotations
- ✅ **Auto-Formatting** - Format document/selection
- ✅ **Completion** - Context-aware code completion

---

## Keyboard Shortcuts

| Action | Shortcut (Mac) | Shortcut (Win/Linux) |
|--------|----------------|----------------------|
| Go to Symbol | Cmd+Shift+O | Ctrl+Shift+O |
| Format Document | Shift+Alt+F | Shift+Alt+F |
| Hover | Hover mouse | Hover mouse |
| Code Actions | Cmd+. | Ctrl+. |
| Command Palette | Cmd+Shift+P | Ctrl+Shift+P |

---

## Troubleshooting

### LSP Not Starting

1. Verify `atlas` is in PATH:
   ```bash
   which atlas
   atlas lsp --help
   ```

2. Check VS Code Output panel:
   - View → Output
   - Select "Atlas Language Server" from dropdown

3. Enable verbose logging:
   ```json
   {
     "atlas.lsp.trace.server": "verbose"
   }
   ```

### Features Not Working

- Ensure file has `.at` or `.atl` extension
- Reload VS Code window: Cmd+Shift+P → "Reload Window"
- Check LSP server status: bottom-right status bar

---

## Advanced Configuration

### Custom LSP Port

```json
{
  "atlas.lsp.tcp": true,
  "atlas.lsp.port": 9257
}
```

### File Associations

```json
{
  "files.associations": {
    "*.at": "atlas",
    "*.atl": "atlas"
  }
}
```

---

## More Information

- [LSP Features](../lsp-features.md)
- [Troubleshooting Guide](../lsp-troubleshooting.md)
- [Atlas Documentation](https://github.com/atl-lang/atlas)
