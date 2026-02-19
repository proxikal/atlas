//! TUI-based REPL using ratatui

use anyhow::Result;
use atlas_runtime::ReplCore;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

/// Application state for the TUI REPL
struct App {
    /// REPL core for evaluation
    repl: ReplCore,
    /// Current input being edited
    input: String,
    /// Cursor position in input
    cursor_pos: usize,
    /// History of inputs and outputs
    history: Vec<HistoryItem>,
    /// Whether to quit the app
    should_quit: bool,
    /// Status message
    status: String,
}

/// A single history item (input + output)
struct HistoryItem {
    input: String,
    output: String,
    is_error: bool,
}

impl App {
    fn new() -> Self {
        Self {
            repl: ReplCore::new(),
            input: String::new(),
            cursor_pos: 0,
            history: Vec::new(),
            should_quit: false,
            status: "Atlas TUI REPL - Press Ctrl+C to exit, Ctrl+R to reset".to_string(),
        }
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('r') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.repl.reset();
                self.history.clear();
                self.status = "REPL state reset".to_string();
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
            }
            KeyCode::Enter => {
                self.execute_input();
            }
            _ => {}
        }
    }

    /// Execute the current input
    fn execute_input(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        let input = self.input.clone();
        let result = self.repl.eval_line(&input);

        // Format output
        let output = if !result.diagnostics.is_empty() {
            // Show diagnostics
            result
                .diagnostics
                .iter()
                .map(|d| format!("{}: {}", d.level, d.message))
                .collect::<Vec<_>>()
                .join("\n")
        } else if let Some(value) = result.value {
            // Show value (unless it's null)
            if matches!(value, atlas_runtime::Value::Null) {
                String::new()
            } else {
                value.to_string()
            }
        } else {
            String::new()
        };

        let is_error = !result.diagnostics.is_empty();

        // Add to history
        self.history.push(HistoryItem {
            input: input.clone(),
            output,
            is_error,
        });

        // Clear input
        self.input.clear();
        self.cursor_pos = 0;

        // Update status
        self.status = format!("Executed: {}", input);
    }
}

/// Run the TUI REPL
pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

/// Run the application main loop
fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as ratatui::backend::Backend>::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key.code, key.modifiers);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Draw the UI
fn ui(f: &mut Frame, app: &App) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // History
            Constraint::Length(3), // Input
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // History panel
    let history_items: Vec<ListItem> = app
        .history
        .iter()
        .flat_map(|item| {
            let mut items = Vec::new();

            // Input line (in cyan)
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    ">> ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&item.input),
            ])));

            // Output line (in green or red depending on error)
            if !item.output.is_empty() {
                let color = if item.is_error {
                    Color::Red
                } else {
                    Color::Green
                };
                items.push(ListItem::new(Line::from(Span::styled(
                    &item.output,
                    Style::default().fg(color),
                ))));
            }

            items
        })
        .collect();

    let history_widget = List::new(history_items).block(
        Block::default()
            .title("History")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(history_widget, chunks[0]);

    // Input panel
    let input_text = if app.cursor_pos < app.input.len() {
        // Show cursor by splitting at cursor position
        vec![
            Span::raw(&app.input[..app.cursor_pos]),
            Span::styled(
                &app.input[app.cursor_pos..app.cursor_pos + 1],
                Style::default().bg(Color::White).fg(Color::Black),
            ),
            Span::raw(&app.input[app.cursor_pos + 1..]),
        ]
    } else {
        // Cursor at end
        vec![
            Span::raw(&app.input),
            Span::styled(" ", Style::default().bg(Color::White)),
        ]
    };

    let input_widget = Paragraph::new(Text::from(Line::from(input_text)))
        .block(
            Block::default()
                .title("Input (Press Enter to execute)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(input_widget, chunks[1]);

    // Status bar
    let status_widget = Paragraph::new(app.status.as_str()).style(Style::default().fg(Color::Gray));

    f.render_widget(status_widget, chunks[2]);
}
