# Phase Interpreter-01c: REPL Usability Enhancements

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Interpreter-01b complete. Command system and multi-line input working.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
grep "ReplCommand" crates/atlas-runtime/src/repl.rs  # Commands exist
grep "MultilineDetector" crates/atlas-runtime/src/repl*.rs  # Multi-line exists
cargo nextest run -p atlas-runtime --test repl 2>&1 | tail -3  # REPL tests pass
```

---

## Objective

Complete the REPL with professional usability features: persistent command history, color-coded output, and tab completion. These polish features transform the REPL into a tool developers want to use.

---

## Files Changed

- `crates/atlas-runtime/src/repl.rs` — **UPDATE** (~100 lines) integrate history and colors
- `crates/atlas-runtime/src/repl/history.rs` — **CREATE** (~100 lines) persistent history
- `crates/atlas-runtime/src/repl/completion.rs` — **CREATE** (~100 lines) tab completion
- `crates/atlas-runtime/src/repl/colors.rs` — **CREATE** (~80 lines) color output
- `crates/atlas-cli/src/commands/repl.rs` — **UPDATE** (~50 lines) wire up readline
- `crates/atlas-runtime/tests/repl.rs` — **UPDATE** add usability tests

---

## Dependencies

- Interpreter-01b complete
- `rustyline` crate for readline (already in deps)

---

## Implementation

### Step 1: Persistent history

Create `repl/history.rs`:
```rust
pub struct ReplHistory {
    path: PathBuf,
    entries: Vec<String>,
    max_entries: usize,
}

impl ReplHistory {
    pub fn load(path: &Path) -> Self {
        // Load from ~/.atlas_history or project-local
        // Parse entries, deduplicate adjacent duplicates
    }

    pub fn add(&mut self, entry: &str) {
        if !entry.trim().is_empty() && Some(entry) != self.entries.last().map(|s| s.as_str()) {
            self.entries.push(entry.to_string());
            if self.entries.len() > self.max_entries {
                self.entries.remove(0);
            }
        }
    }

    pub fn save(&self) -> io::Result<()> {
        // Write to history file
    }

    pub fn search(&self, prefix: &str) -> Vec<&str> {
        // Reverse search matching prefix
    }
}
```

History file location: `~/.atlas/history` (create dir if needed).

### Step 2: Color-coded output

Create `repl/colors.rs`:
```rust
pub struct ColorScheme {
    pub number: Style,
    pub string: Style,
    pub boolean: Style,
    pub null: Style,
    pub function: Style,
    pub array: Style,
    pub error: Style,
    pub prompt: Style,
    pub continuation: Style,
}

impl ColorScheme {
    pub fn default() -> Self {
        Self {
            number: Style::new().cyan(),
            string: Style::new().green(),
            boolean: Style::new().yellow(),
            null: Style::new().dim(),
            function: Style::new().magenta(),
            array: Style::new().blue(),
            error: Style::new().red().bold(),
            prompt: Style::new().bold(),
            continuation: Style::new().dim(),
        }
    }

    pub fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Number(n) => self.number.apply_to(n.to_string()),
            Value::String(s) => self.string.apply_to(format!("\"{}\"", s)),
            Value::Boolean(b) => self.boolean.apply_to(b.to_string()),
            Value::Null => self.null.apply_to("null"),
            Value::Function(f) => self.function.apply_to(format!("<fn {}>", f.name)),
            Value::Array(a) => self.format_array(a),
            // ... other types
        }
    }
}
```

Use `console` crate for terminal colors (or ANSI codes directly).

### Step 3: Tab completion

Create `repl/completion.rs`:
```rust
pub struct ReplCompleter {
    keywords: Vec<String>,
    builtins: Vec<String>,
}

impl ReplCompleter {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "let", "var", "fn", "return", "if", "else", "while", "for",
                "match", "import", "export", "true", "false", "null",
            ].into_iter().map(String::from).collect(),
            builtins: get_builtin_names(), // From stdlib registry
        }
    }

    pub fn complete(&self, line: &str, pos: usize, env: &Environment) -> Vec<Completion> {
        let prefix = self.extract_prefix(line, pos);
        let mut completions = Vec::new();

        // Keywords
        completions.extend(self.keywords.iter()
            .filter(|k| k.starts_with(&prefix))
            .map(|k| Completion::keyword(k)));

        // Builtins
        completions.extend(self.builtins.iter()
            .filter(|b| b.starts_with(&prefix))
            .map(|b| Completion::builtin(b)));

        // User variables from environment
        completions.extend(env.names()
            .filter(|n| n.starts_with(&prefix))
            .map(|n| Completion::variable(n)));

        completions
    }
}
```

Integrate with rustyline's `Completer` trait.

### Step 4: Integrate with rustyline

Update `atlas-cli/src/commands/repl.rs`:
```rust
use rustyline::{Editor, Config, EditMode};
use rustyline::completion::Completer;
use rustyline::hint::Hinter;

struct AtlasHelper {
    completer: ReplCompleter,
    history: ReplHistory,
}

impl Completer for AtlasHelper { ... }
impl Hinter for AtlasHelper { ... }

pub fn run_repl(config: &ReplConfig) -> Result<()> {
    let mut rl = Editor::with_config(Config::builder()
        .edit_mode(EditMode::Emacs)
        .auto_add_history(false)  // We manage history
        .build());

    rl.set_helper(Some(AtlasHelper::new()));
    rl.load_history(&history_path())?;

    // Main REPL loop with colors and completion
}
```

### Step 5: Welcome message and prompts

Add helpful welcome:
```
Atlas REPL v0.2.0
Type :help for commands, :quit to exit

atlas>
```

Continuation prompt for multi-line:
```
atlas> fn add(a: number, b: number) -> number {
.....>     return a + b;
.....> }
```

### Step 6: Result type display

Show types alongside values:
```
atlas> 42
42 : number

atlas> "hello"
"hello" : string

atlas> [1, 2, 3]
[1, 2, 3] : array<number>
```

---

## Tests

Add to `tests/repl.rs`:

**History tests (5):**
- `test_history_add_entry`
- `test_history_no_duplicate_adjacent`
- `test_history_max_entries_enforced`
- `test_history_search_prefix`
- `test_history_persistence_roundtrip`

**Color output tests (5):**
- `test_color_number_format`
- `test_color_string_format`
- `test_color_boolean_format`
- `test_color_array_format`
- `test_color_error_format`

**Completion tests (5):**
- `test_complete_keyword`
- `test_complete_builtin`
- `test_complete_user_variable`
- `test_complete_partial_match`
- `test_complete_no_match_empty`

**Minimum test count:** 15 tests

---

## Acceptance

- Command history persists across sessions in `~/.atlas/history`
- Up/down arrows navigate history
- History search with Ctrl+R (via rustyline)
- Color-coded output for all value types
- Tab completion for keywords, builtins, and user variables
- Welcome message shown on REPL start
- Continuation prompt for multi-line input
- Result types displayed alongside values
- 15+ new tests pass
- All existing tests pass unchanged
- Zero clippy warnings
- Commit: `feat(repl): Add history, colors, and tab completion`

---

## Final Phase Summary

After completing 01a, 01b, and 01c:
- **Total new tests:** 100+ (50 debugger + 35 commands/multiline + 15 usability)
- **Interpreter debugger:** Full parity with VM debugger
- **REPL commands:** Professional command system
- **Multi-line input:** Intelligent continuation detection
- **Usability:** History, colors, completion

Archive original `phase-01-debugger-repl-improvements.md` after all sub-phases complete.
