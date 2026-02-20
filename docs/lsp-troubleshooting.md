# Atlas LSP Troubleshooting

Common issues and solutions for the Atlas Language Server.

---

## Server Not Starting

### Symptom
LSP features don't work, no connection to server.

### Solutions

1. **Verify Atlas CLI installation:**
   ```bash
   which atlas
   atlas lsp --help
   ```

2. **Check server starts manually:**
   ```bash
   atlas lsp --verbose
   ```

3. **Verify PATH:**
   - Ensure `atlas` is in your editor's PATH
   - For GUI editors, PATH may differ from terminal

---

## Features Not Working

### Hover Shows Nothing

**Cause:** Position not on a valid symbol.

**Solution:** Hover directly on function/variable names, not whitespace.

### No Syntax Highlighting

**Cause:** Semantic tokens not enabled in editor.

**Solution:** Enable semantic tokens in editor settings.

### Code Actions Empty

**Cause:** No applicable actions for current context.

**Solution:** Code actions are context-specific. Try on function declarations or variables.

---

## Performance Issues

### Slow Responses

**Symptom:** Features take > 1 second to respond.

**Diagnosis:**
- Check file size (very large files may be slower)
- Check for syntax errors (parsing errors can slow analysis)

**Solutions:**
- Split very large files (> 5000 lines)
- Fix syntax errors
- Restart LSP server

### High CPU Usage

**Cause:** Rapid file changes triggering re-analysis.

**Solution:** LSP debounces changes, but very rapid typing may cause temporary CPU spikes. This is normal.

---

## Connection Issues

### stdio vs TCP

**Default:** stdio (recommended)
**Alternative:** TCP mode for debugging

TCP mode:
```bash
atlas lsp --tcp --port 9257
```

Configure editor to connect to `localhost:9257`.

---

## Logging and Diagnostics

### Enable Verbose Logging

```bash
atlas lsp --verbose
```

### View LSP Protocol Messages

**VS Code:** Output panel → "Atlas Language Server"
**Neovim:** `~/.local/state/nvim/lsp.log`
**Emacs:** `M-x lsp-workspace-show-log`

---

## Common Error Messages

### "Document not found"

**Cause:** Querying a file that wasn't opened.

**Solution:** Ensure file is opened in editor before using LSP features.

### "Position out of bounds"

**Cause:** Position beyond end of document.

**Solution:** Editor bug or stale cache. Close and reopen file.

---

## Getting Help

If issues persist:

1. Check [LSP Status](./lsp-status.md) for known limitations
2. File issue: https://github.com/atl-lang/atlas/issues
3. Include:
   - Atlas version (`atlas --version`)
   - Editor and version
   - LSP logs
   - Minimal reproduction steps

---

## FAQ

**Q: Does LSP work on Windows?**
A: Yes, full cross-platform support.

**Q: Can I use multiple editors simultaneously?**
A: Yes, each editor instance runs its own LSP server.

**Q: What's the performance overhead?**
A: Minimal. LSP runs as separate process, < 50MB RAM typical.

**Q: Do I need to restart LSP when I change files?**
A: No, LSP automatically detects file changes.
