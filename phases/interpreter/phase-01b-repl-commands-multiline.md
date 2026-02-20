# Phase Interpreter-01b: REPL Commands & Multi-line Input

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Interpreter-01a complete. REPL exists in `src/repl.rs`.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
ls crates/atlas-runtime/src/repl.rs                  # REPL exists
ls crates/atlas-runtime/src/interpreter/debugger.rs # From 01a
cargo nextest run -p atlas-runtime 2>&1 | tail -3   # Suite green
```

---

## Objective

Enhance the REPL with a command system (`:help`, `:clear`, `:load`, etc.) and multi-line input detection for incomplete expressions. This transforms the REPL from a basic evaluator into a professional interactive development tool.

---

## Files Changed

- `crates/atlas-runtime/src/repl.rs` — **UPDATE** (~200 lines) add command parsing and execution
- `crates/atlas-runtime/src/repl/multiline.rs` — **CREATE** (~150 lines) multi-line detection
- `crates/atlas-runtime/src/repl/mod.rs` — **CREATE** if restructuring to submodule
- `crates/atlas-runtime/tests/repl.rs` — **UPDATE** add command and multi-line tests

---

## Dependencies

- Interpreter-01a complete
- Existing REPL infrastructure (`repl.rs`)

---

## Implementation

### Step 1: Define REPL commands

Commands use colon prefix to distinguish from Atlas code:
```rust
pub enum ReplCommand {
    Help,                      // :help - list commands
    Clear,                     // :clear - reset REPL state
    Load(PathBuf),             // :load <file> - execute file
    Type(String),              // :type <expr> - show expression type
    Vars,                      // :vars - list all variables
    Quit,                      // :quit - exit REPL
    Debug(DebugSubcommand),    // :debug <subcmd> - debugger control
}

pub enum DebugSubcommand {
    Break(String),             // :debug break <location>
    Step,                      // :debug step
    Continue,                  // :debug continue
    Vars,                      // :debug vars
}
```

### Step 2: Command parsing

Add command parser to REPL:
```rust
impl Repl {
    pub fn parse_input(&self, input: &str) -> InputType {
        let trimmed = input.trim();
        if trimmed.starts_with(':') {
            self.parse_command(trimmed)
        } else {
            InputType::Code(input.to_string())
        }
    }

    fn parse_command(&self, input: &str) -> InputType {
        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        match parts.get(0).map(|s| *s) {
            Some("help") | Some("h") => InputType::Command(ReplCommand::Help),
            Some("clear") => InputType::Command(ReplCommand::Clear),
            Some("load") => {
                let path = parts.get(1).map(PathBuf::from);
                InputType::Command(ReplCommand::Load(path.unwrap_or_default()))
            }
            Some("type") | Some("t") => {
                let expr = parts[1..].join(" ");
                InputType::Command(ReplCommand::Type(expr))
            }
            Some("vars") | Some("v") => InputType::Command(ReplCommand::Vars),
            Some("quit") | Some("q") => InputType::Command(ReplCommand::Quit),
            Some("debug") | Some("d") => self.parse_debug_command(&parts[1..]),
            _ => InputType::UnknownCommand(parts.get(0).unwrap_or(&"").to_string()),
        }
    }
}
```

### Step 3: Command execution

Implement each command:
```rust
impl Repl {
    pub fn execute_command(&mut self, cmd: ReplCommand) -> ReplResult {
        match cmd {
            ReplCommand::Help => self.show_help(),
            ReplCommand::Clear => { self.reset(); ReplResult::Ok(None) }
            ReplCommand::Load(path) => self.load_file(&path),
            ReplCommand::Type(expr) => self.show_type(&expr),
            ReplCommand::Vars => self.show_vars(),
            ReplCommand::Quit => ReplResult::Quit,
            ReplCommand::Debug(sub) => self.handle_debug(sub),
        }
    }
}
```

### Step 4: Multi-line input detection

Create `repl/multiline.rs`:
```rust
pub struct MultilineDetector {
    brace_depth: i32,      // { }
    bracket_depth: i32,    // [ ]
    paren_depth: i32,      // ( )
    in_string: bool,
    in_comment: bool,
}

impl MultilineDetector {
    pub fn is_complete(&self, input: &str) -> bool {
        let mut state = self.clone();
        state.scan(input);
        state.brace_depth == 0
            && state.bracket_depth == 0
            && state.paren_depth == 0
            && !state.in_string
    }

    pub fn needs_continuation(&self) -> bool {
        self.brace_depth > 0
            || self.bracket_depth > 0
            || self.paren_depth > 0
            || self.in_string
    }

    fn scan(&mut self, input: &str) { ... }
}
```

Handle edge cases:
- Strings spanning multiple lines
- Comments (single-line `//` and multi-line `/* */`)
- Escape sequences in strings
- Template strings if supported

### Step 5: Integrate multi-line in eval_line

Update `eval_line` to accumulate input:
```rust
impl Repl {
    pub fn eval_line(&mut self, input: &str) -> ReplResult {
        self.buffer.push_str(input);
        self.buffer.push('\n');

        if !self.detector.is_complete(&self.buffer) {
            return ReplResult::Continuation;
        }

        let full_input = std::mem::take(&mut self.buffer);
        self.evaluate(&full_input)
    }

    pub fn cancel_multiline(&mut self) {
        self.buffer.clear();
        self.detector.reset();
    }
}
```

### Step 6: Update ReplResult enum

Extend result type:
```rust
pub enum ReplResult {
    Ok(Option<Value>),       // Successful evaluation
    Error(String),           // Evaluation error
    Continuation,            // Need more input (multi-line)
    Quit,                    // User requested exit
    CommandOutput(String),   // Command produced output (e.g., :help)
}
```

---

## Tests

Add to `tests/repl.rs` (use rstest):

**Command parsing tests (8):**
- `test_parse_help_command`
- `test_parse_clear_command`
- `test_parse_load_command_with_path`
- `test_parse_type_command_with_expression`
- `test_parse_vars_command`
- `test_parse_quit_command`
- `test_parse_unknown_command`
- `test_parse_code_not_command`

**Command execution tests (8):**
- `test_help_lists_commands`
- `test_clear_resets_state`
- `test_load_executes_file`
- `test_load_nonexistent_file_error`
- `test_type_shows_expression_type`
- `test_vars_lists_all_bindings`
- `test_quit_returns_quit_result`
- `test_debug_break_sets_breakpoint`

**Multi-line detection tests (12):**
- `test_complete_single_line`
- `test_incomplete_open_brace`
- `test_incomplete_open_bracket`
- `test_incomplete_open_paren`
- `test_nested_braces_incomplete`
- `test_nested_braces_complete`
- `test_string_with_brace_not_counted`
- `test_multiline_string_detection`
- `test_single_line_comment_ignored`
- `test_multiline_comment_detection`
- `test_escape_in_string_handled`
- `test_mixed_nesting_complete`

**Multi-line input flow tests (7):**
- `test_multiline_function_definition`
- `test_multiline_array_literal`
- `test_multiline_object_literal`
- `test_multiline_if_expression`
- `test_cancel_multiline_clears_buffer`
- `test_continuation_prompt_returned`
- `test_multiline_error_shows_full_context`

**Minimum test count:** 35 tests

---

## Acceptance

- All REPL commands functional: `:help`, `:clear`, `:load`, `:type`, `:vars`, `:quit`
- Command shortcuts work: `:h`, `:t`, `:v`, `:q`
- Unknown commands produce helpful error
- Multi-line input detected for incomplete expressions
- Braces, brackets, parens tracked correctly
- Strings and comments don't affect nesting counts
- Continuation prompt returned for incomplete input
- Cancel multi-line clears accumulated buffer
- 35+ new tests pass
- All existing tests pass unchanged
- Zero clippy warnings
- Commit: `feat(repl): Add command system and multi-line input detection`
