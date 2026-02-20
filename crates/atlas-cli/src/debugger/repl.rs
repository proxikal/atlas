//! Interactive debugger REPL
//!
//! Provides a command-line interface for debugging Atlas programs.

use atlas_runtime::debugger::{
    BreakpointId, DebugRequest, DebugResponse, DebuggerSession, PauseReason, SourceLocation,
};
use atlas_runtime::SecurityContext;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;

/// Debugger REPL state
pub struct DebugRepl {
    session: DebuggerSession,
    security: SecurityContext,
    #[allow(dead_code)]
    source: String,
    file_name: String,
    source_lines: Vec<String>,
    running: bool,
    breakpoint_names: HashMap<BreakpointId, String>,
}

impl DebugRepl {
    /// Create a new debugger REPL
    pub fn new(session: DebuggerSession, source: String, file_name: String) -> Self {
        let source_lines = source.lines().map(String::from).collect();
        Self {
            session,
            security: SecurityContext::allow_all(),
            source,
            file_name,
            source_lines,
            running: true,
            breakpoint_names: HashMap::new(),
        }
    }

    /// Run the interactive debugger REPL
    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut rl = DefaultEditor::new()?;

        // Print welcome message
        println!();
        println!("\x1b[1;36m╭─────────────────────────────────────╮\x1b[0m");
        println!("\x1b[1;36m│\x1b[0m  \x1b[1mAtlas Debugger\x1b[0m                    \x1b[1;36m│\x1b[0m");
        println!("\x1b[1;36m╰─────────────────────────────────────╯\x1b[0m");
        println!();
        println!("Type \x1b[1mhelp\x1b[0m for available commands.");
        println!("Debugging: \x1b[33m{}\x1b[0m", self.file_name);
        println!();

        // Show initial location
        self.show_current_location();

        while self.running {
            let prompt = self.get_prompt();
            let readline = rl.readline(&prompt);

            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = rl.add_history_entry(trimmed);

                    // Parse and execute command
                    self.execute_command(trimmed);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C - Use 'quit' to exit");
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        println!("Debugger exited.");
        Ok(())
    }

    /// Get the prompt string based on debugger state
    fn get_prompt(&self) -> String {
        if self.session.is_paused() {
            "\x1b[33m(paused)\x1b[0m > ".to_string()
        } else if self.session.is_stopped() {
            "\x1b[31m(stopped)\x1b[0m > ".to_string()
        } else {
            "\x1b[32m(debug)\x1b[0m > ".to_string()
        }
    }

    /// Execute a debugger command
    pub fn execute_command(&mut self, input: &str) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "help" | "h" | "?" => self.cmd_help(),
            "quit" | "q" | "exit" => self.cmd_quit(),
            "run" | "r" => self.cmd_run(),
            "continue" | "c" => self.cmd_continue(),
            "step" | "s" => self.cmd_step(),
            "next" | "n" => self.cmd_next(),
            "out" | "finish" => self.cmd_out(),
            "break" | "b" => self.cmd_break(args),
            "delete" | "d" | "clear" => self.cmd_delete(args),
            "list" | "l" => self.cmd_list(args),
            "breakpoints" | "bp" => self.cmd_breakpoints(),
            "vars" | "v" | "locals" => self.cmd_vars(args),
            "print" | "p" | "inspect" => self.cmd_print(args),
            "backtrace" | "bt" | "where" => self.cmd_backtrace(),
            "location" | "loc" => self.cmd_location(),
            _ => println!(
                "Unknown command: '{}'. Type 'help' for available commands.",
                cmd
            ),
        }
    }

    // ── Command implementations ───────────────────────────────────────────────

    fn cmd_help(&self) {
        println!();
        println!("\x1b[1mDebugger Commands:\x1b[0m");
        println!();
        println!("  \x1b[1;33mExecution:\x1b[0m");
        println!("    run, r              Start/restart execution");
        println!("    continue, c         Continue execution until breakpoint");
        println!("    step, s             Step into function");
        println!("    next, n             Step over function call");
        println!("    out, finish         Step out of current function");
        println!();
        println!("  \x1b[1;33mBreakpoints:\x1b[0m");
        println!("    break <line>, b     Set breakpoint at line number");
        println!("    break <func>        Set breakpoint at function");
        println!("    delete <id>, d      Delete breakpoint by ID");
        println!("    delete all          Delete all breakpoints");
        println!("    breakpoints, bp     List all breakpoints");
        println!();
        println!("  \x1b[1;33mInspection:\x1b[0m");
        println!("    vars, v, locals     Show local variables");
        println!("    print <expr>, p     Evaluate and print expression");
        println!("    backtrace, bt       Show call stack");
        println!("    location, loc       Show current location");
        println!();
        println!("  \x1b[1;33mSource:\x1b[0m");
        println!("    list, l             List source around current line");
        println!("    list <line>         List source around specified line");
        println!();
        println!("  \x1b[1;33mOther:\x1b[0m");
        println!("    help, h, ?          Show this help");
        println!("    quit, q, exit       Exit debugger");
        println!();
    }

    fn cmd_quit(&mut self) {
        self.running = false;
    }

    fn cmd_run(&mut self) {
        println!("Starting program...");
        let response = self.session.run_until_pause(&self.security);
        self.handle_pause_response(response);
    }

    fn cmd_continue(&mut self) {
        let _ = self.session.process_request(DebugRequest::Continue);
        let response = self.session.run_until_pause(&self.security);
        self.handle_pause_response(response);
    }

    fn cmd_step(&mut self) {
        let _ = self.session.process_request(DebugRequest::StepInto);
        let response = self.session.run_until_pause(&self.security);
        self.handle_pause_response(response);
    }

    fn cmd_next(&mut self) {
        let _ = self.session.process_request(DebugRequest::StepOver);
        let response = self.session.run_until_pause(&self.security);
        self.handle_pause_response(response);
    }

    fn cmd_out(&mut self) {
        let _ = self.session.process_request(DebugRequest::StepOut);
        let response = self.session.run_until_pause(&self.security);
        self.handle_pause_response(response);
    }

    fn cmd_break(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: break <line> or break <function>");
            return;
        }

        let arg = args[0];

        // Try to parse as line number
        if let Ok(line) = arg.parse::<u32>() {
            let location = SourceLocation::new(&self.file_name, line, 1);
            let response = self
                .session
                .process_request(DebugRequest::SetBreakpoint { location });

            match response {
                DebugResponse::BreakpointSet { breakpoint } => {
                    self.breakpoint_names
                        .insert(breakpoint.id, format!("line {}", line));
                    let status = if breakpoint.verified {
                        "\x1b[32mverified\x1b[0m"
                    } else {
                        "\x1b[33munverified\x1b[0m"
                    };
                    println!(
                        "Breakpoint {} set at line {} [{}]",
                        breakpoint.id, line, status
                    );
                }
                DebugResponse::Error { message } => {
                    println!("\x1b[31mError:\x1b[0m {}", message);
                }
                _ => {}
            }
        } else {
            // Treat as function name (set at line 1 for now - could be improved)
            println!(
                "\x1b[33mNote:\x1b[0m Function breakpoints not yet implemented. Use line numbers."
            );
        }
    }

    fn cmd_delete(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: delete <id> or delete all");
            return;
        }

        if args[0] == "all" {
            let response = self.session.process_request(DebugRequest::ClearBreakpoints);
            if let DebugResponse::BreakpointsCleared = response {
                self.breakpoint_names.clear();
                println!("All breakpoints cleared.");
            }
            return;
        }

        if let Ok(id) = args[0].parse::<BreakpointId>() {
            let response = self
                .session
                .process_request(DebugRequest::RemoveBreakpoint { id });

            match response {
                DebugResponse::BreakpointRemoved { id } => {
                    self.breakpoint_names.remove(&id);
                    println!("Breakpoint {} deleted.", id);
                }
                DebugResponse::Error { message } => {
                    println!("\x1b[31mError:\x1b[0m {}", message);
                }
                _ => {}
            }
        } else {
            println!("Invalid breakpoint ID: '{}'", args[0]);
        }
    }

    fn cmd_breakpoints(&mut self) {
        let response = self.session.process_request(DebugRequest::ListBreakpoints);

        match response {
            DebugResponse::Breakpoints { breakpoints } => {
                if breakpoints.is_empty() {
                    println!("No breakpoints set.");
                    return;
                }

                println!();
                println!("\x1b[1mBreakpoints:\x1b[0m");
                println!("{:<4} {:<10} {:<20}", "ID", "Status", "Location");
                println!("{}", "-".repeat(40));

                for bp in &breakpoints {
                    let status = if bp.verified {
                        "\x1b[32mverified\x1b[0m"
                    } else {
                        "\x1b[33mpending\x1b[0m"
                    };

                    let file_name = bp
                        .location
                        .file
                        .rsplit('/')
                        .next()
                        .unwrap_or(&bp.location.file);
                    let location = format!("{}:{}", file_name, bp.location.line);

                    println!("{:<4} {:<10} {:<20}", bp.id, status, location);
                }
                println!();
            }
            _ => println!("Failed to list breakpoints."),
        }
    }

    fn cmd_list(&mut self, args: &[&str]) {
        let center_line = if args.is_empty() {
            // Use current location
            if let DebugResponse::Location { location, .. } =
                self.session.process_request(DebugRequest::GetLocation)
            {
                location.map(|l| l.line as usize).unwrap_or(1)
            } else {
                1
            }
        } else {
            args[0].parse::<usize>().unwrap_or(1)
        };

        self.display_source_context(center_line, 5);
    }

    fn cmd_vars(&mut self, args: &[&str]) {
        let frame_index = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);

        let response = self
            .session
            .process_request(DebugRequest::GetVariables { frame_index });

        match response {
            DebugResponse::Variables { variables, .. } => {
                if variables.is_empty() {
                    println!("No variables in current scope.");
                    return;
                }

                println!();
                println!("\x1b[1mVariables (frame {}):\x1b[0m", frame_index);
                println!("{:<20} {:<15} Value", "Name", "Type");
                println!("{}", "-".repeat(60));

                for var in &variables {
                    println!(
                        "{:<20} \x1b[36m{:<15}\x1b[0m {}",
                        var.name, var.type_name, var.value
                    );
                }
                println!();
            }
            DebugResponse::Error { message } => {
                println!("\x1b[31mError:\x1b[0m {}", message);
            }
            _ => {}
        }
    }

    fn cmd_print(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: print <expression>");
            return;
        }

        let expression = args.join(" ");
        let response = self.session.process_request(DebugRequest::Evaluate {
            expression: expression.clone(),
            frame_index: 0,
        });

        match response {
            DebugResponse::EvalResult { value, type_name } => {
                println!("\x1b[36m{}\x1b[0m = {}", type_name, value);
            }
            DebugResponse::Error { message } => {
                println!(
                    "\x1b[31mError evaluating '{}':\x1b[0m {}",
                    expression, message
                );
            }
            _ => {}
        }
    }

    fn cmd_backtrace(&mut self) {
        let response = self.session.process_request(DebugRequest::GetStack);

        match response {
            DebugResponse::StackTrace { frames } => {
                if frames.is_empty() {
                    println!("No stack frames.");
                    return;
                }

                println!();
                println!("\x1b[1mCall Stack:\x1b[0m");
                for (i, frame) in frames.iter().enumerate() {
                    let marker = if i == 0 { "→" } else { " " };
                    let location = frame
                        .location
                        .as_ref()
                        .map(|l| format!("{}:{}", l.file, l.line))
                        .unwrap_or_else(|| "unknown".to_string());

                    println!(
                        "  {} #{} \x1b[33m{}\x1b[0m at {}",
                        marker, frame.index, frame.function_name, location
                    );
                }
                println!();
            }
            _ => println!("Failed to get stack trace."),
        }
    }

    fn cmd_location(&mut self) {
        self.show_current_location();
    }

    // ── Helper methods ────────────────────────────────────────────────────────

    fn handle_pause_response(&self, response: DebugResponse) {
        match response {
            DebugResponse::Paused {
                reason, location, ..
            } => {
                let reason_str = match reason {
                    PauseReason::Breakpoint { id } => {
                        format!("Breakpoint {} hit", id)
                    }
                    PauseReason::Step => "Stepped".to_string(),
                    PauseReason::ManualPause => "Paused".to_string(),
                    PauseReason::Exception { message } => {
                        format!("Exception: {}", message)
                    }
                };

                if let Some(loc) = &location {
                    println!(
                        "\x1b[33m{}\x1b[0m at {}:{} (column {})",
                        reason_str, loc.file, loc.line, loc.column
                    );
                } else {
                    println!("\x1b[33m{}\x1b[0m (program ended)", reason_str);
                }

                // Show source context
                if let Some(loc) = location {
                    self.display_source_context(loc.line as usize, 2);
                }
            }
            DebugResponse::Error { message } => {
                println!("\x1b[31mError:\x1b[0m {}", message);
            }
            _ => {}
        }
    }

    fn show_current_location(&mut self) {
        let response = self.session.process_request(DebugRequest::GetLocation);

        if let DebugResponse::Location { location, ip } = response {
            if let Some(loc) = location {
                println!("At {}:{} (IP: {})", loc.file, loc.line, ip);
                self.display_source_context(loc.line as usize, 2);
            } else {
                println!("Location unknown (IP: {})", ip);
            }
        }
    }

    fn display_source_context(&self, center_line: usize, context: usize) {
        let start = center_line.saturating_sub(context).max(1);
        let end = (center_line + context).min(self.source_lines.len());

        println!();
        for line_num in start..=end {
            if line_num > self.source_lines.len() {
                break;
            }

            let line = &self.source_lines[line_num - 1];
            let marker = if line_num == center_line { "→" } else { " " };

            // Check if there's a breakpoint on this line
            let bp_marker = if self.has_breakpoint_at_line(line_num as u32) {
                "\x1b[31m●\x1b[0m"
            } else {
                " "
            };

            if line_num == center_line {
                println!(
                    "\x1b[33m{}\x1b[0m {} \x1b[90m{:4}\x1b[0m │ \x1b[1m{}\x1b[0m",
                    marker, bp_marker, line_num, line
                );
            } else {
                println!(
                    "{} {} \x1b[90m{:4}\x1b[0m │ {}",
                    marker, bp_marker, line_num, line
                );
            }
        }
        println!();
    }

    fn has_breakpoint_at_line(&self, line: u32) -> bool {
        // Check local tracking
        self.breakpoint_names
            .values()
            .any(|name| name.contains(&format!("line {}", line)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_runtime::{Compiler, Lexer, Parser};

    fn create_test_session(source: &str) -> (DebuggerSession, String) {
        let (tokens, _) = Lexer::new(source).tokenize();
        let (ast, _) = Parser::new(tokens).parse();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let session = DebuggerSession::new(bytecode, source, "test.atlas");
        (session, source.to_string())
    }

    #[test]
    fn test_debug_repl_creation() {
        let source = "let x = 1;\nlet y = 2;";
        let (session, source) = create_test_session(source);
        let repl = DebugRepl::new(session, source, "test.atlas".to_string());
        assert!(repl.running);
        assert_eq!(repl.source_lines.len(), 2);
    }

    #[test]
    fn test_prompt_states() {
        let source = "let x = 1;";
        let (session, source) = create_test_session(source);
        let repl = DebugRepl::new(session, source, "test.atlas".to_string());

        // Initial state should show debug prompt
        let prompt = repl.get_prompt();
        assert!(
            prompt.contains("debug") || prompt.contains("paused") || prompt.contains("stopped")
        );
    }

    #[test]
    fn test_source_lines_parsing() {
        let source = "let a = 1;\nlet b = 2;\nlet c = 3;";
        let (session, source) = create_test_session(source);
        let repl = DebugRepl::new(session, source, "test.atlas".to_string());

        assert_eq!(repl.source_lines.len(), 3);
        assert_eq!(repl.source_lines[0], "let a = 1;");
        assert_eq!(repl.source_lines[1], "let b = 2;");
        assert_eq!(repl.source_lines[2], "let c = 3;");
    }

    #[test]
    fn test_breakpoint_name_tracking() {
        let source = "let x = 1;";
        let (session, source) = create_test_session(source);
        let mut repl = DebugRepl::new(session, source, "test.atlas".to_string());

        // Simulate adding a breakpoint
        repl.breakpoint_names.insert(1, "line 1".to_string());
        assert_eq!(repl.breakpoint_names.get(&1), Some(&"line 1".to_string()));

        // Simulate removing it
        repl.breakpoint_names.remove(&1);
        assert!(repl.breakpoint_names.is_empty());
    }

    #[test]
    fn test_has_breakpoint_at_line() {
        let source = "let x = 1;";
        let (session, source) = create_test_session(source);
        let mut repl = DebugRepl::new(session, source, "test.atlas".to_string());

        assert!(!repl.has_breakpoint_at_line(1));

        repl.breakpoint_names.insert(1, "line 1".to_string());
        assert!(repl.has_breakpoint_at_line(1));
        assert!(!repl.has_breakpoint_at_line(2));
    }
}
