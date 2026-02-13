# Atlas REPL Modes

Atlas provides two REPL modes to suit different workflows:

## 1. Line Editor Mode (Default)

**Best for:** AI agents, scripting, automation, CI/CD

```bash
atlas repl
```

Features:
- Simple line-based input/output
- Command history with arrow keys
- Standard readline keybindings
- Easy to script and automate
- Works in headless environments
- Lower resource usage

Commands:
- `:quit` or `:q` - Exit REPL
- `:reset` - Clear all state
- `:help` or `:h` - Show help

**Example:**
```
$ atlas repl
Atlas v0.1 REPL
Type expressions or statements, or :quit to exit
Commands: :quit (or :q), :reset, :help

>> let x = 42;
>> x + 8;
50
>> :quit
Goodbye!
```

## 2. TUI Mode (Rich UI)

**Best for:** Human developers, interactive exploration, learning

```bash
atlas repl --tui
```

Features:
- Full-screen terminal UI
- Split-pane interface (history + input)
- Visual cursor with syntax highlighting placeholder
- Color-coded output (green for values, red for errors)
- Real-time status bar
- Keyboard shortcuts:
  - `Ctrl+C` - Exit TUI
  - `Ctrl+R` - Reset REPL state
  - `Enter` - Execute input
  - Arrow keys - Navigate input
  - `Home`/`End` - Jump to start/end

**UI Layout:**
```
┌─ History ──────────────────────────────┐
│ >> let x = 42;                         │
│ >> x + 8;                              │
│ 50                                     │
│ >> let y = "hello";                    │
│ error: Type mismatch...                │
└────────────────────────────────────────┘
┌─ Input (Press Enter to execute) ───────┐
│ [your input here]█                     │
└────────────────────────────────────────┘
Status: Ready
```

## When to Use Each Mode

| Feature | Line Editor | TUI |
|---------|------------|-----|
| **AI-friendly** | ✅ Yes | ❌ No |
| **Scriptable** | ✅ Yes | ❌ No |
| **Human-friendly** | ⚠️ Basic | ✅ Excellent |
| **Visual feedback** | ❌ No | ✅ Yes |
| **Copy-paste** | ✅ Easy | ⚠️ Harder |
| **CI/CD** | ✅ Yes | ❌ No |
| **SSH/Remote** | ✅ Yes | ✅ Yes |
| **Resource usage** | ✅ Low | ⚠️ Higher |

## Design Philosophy

### AI-Optimized (Line Editor)
- **Simple I/O:** Plain text in, plain text out
- **No terminal dependencies:** Works with any text stream
- **Deterministic:** Same input → same output
- **Automatable:** Can be driven by scripts or AI agents

### Human-Optimized (TUI)
- **Rich feedback:** Visual cues, colors, formatting
- **Interactive:** Real-time feedback and exploration
- **Ergonomic:** Designed for human comfort and productivity
- **Educational:** Great for learning and experimentation

## Implementation

Both modes share the same **ReplCore** backend:
- Same language semantics
- Same evaluation logic
- Same state management
- Only the frontend differs

This ensures:
- ✅ Feature parity between modes
- ✅ Consistent behavior
- ✅ Single source of truth for language logic

## Examples

### Line Editor Mode (AI-Friendly)
```bash
# Can be automated
echo "1 + 2;" | atlas repl

# Works with pipes
cat program.atl | atlas repl

# Good for testing
atlas repl < test_input.txt > test_output.txt
```

### TUI Mode (Human-Friendly)
```bash
# Interactive exploration
atlas repl --tui

# Great for demos and presentations
atlas repl --tui

# Learning and experimentation
atlas repl --tui
```

## Future Enhancements

Planned features for TUI mode:
- [ ] Syntax highlighting in input
- [ ] Auto-completion
- [ ] Multi-line editing
- [ ] Output history search
- [ ] Variable inspector panel
- [ ] Keybinding customization
- [ ] Theme support

## Technical Details

- **Line Editor:** Uses `rustyline` (mature, stable, widely used)
- **TUI:** Uses `ratatui` (modern, fast, actively maintained)
- **Terminal Backend:** Uses `crossterm` (cross-platform)
- **Shared Core:** Both use `ReplCore` from `atlas-runtime`

See `docs/repl.md` for architectural details.
