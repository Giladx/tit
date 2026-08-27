use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tui_textarea::{Input, TextArea};
use serde_json::Value;

pub struct JsonFormatter<'a> {
    input: TextArea<'a>,
    output: String,
    mode: Mode,
}

#[derive(PartialEq)]
enum Mode {
    Format,
    Minify,
}

impl<'a> JsonFormatter<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input (Type here) "));
        Self {
            input,
            output: String::new(),
            mode: Mode::Format,
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        if text.trim().is_empty() {
            self.output.clear();
            return;
        }

        match serde_json::from_str::<Value>(&text) {
            Ok(val) => {
                self.output = match self.mode {
                    Mode::Format => serde_json::to_string_pretty(&val).unwrap_or_else(|_| "Error formatting JSON".to_string()),
                    Mode::Minify => serde_json::to_string(&val).unwrap_or_else(|_| "Error minifying JSON".to_string()),
                };
            }
            Err(e) => {
                self.output = format!("Invalid JSON: {}", e);
            }
        }
    }
}

impl<'a> Tool for JsonFormatter<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "json-formatter",
            name: "JSON Formatter",
            category: Category::Development,
            description: "Format or minify JSON strings.",
            keywords: &["json", "format", "minify", "pretty", "uglify"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        let border_style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

        let mode_str = if self.mode == Mode::Format { "Format" } else { "Minify" };
        let instructions = Paragraph::new(format!("Mode: {} (Press Tab to switch) | Esc to go back", mode_str))
            .block(Block::default().borders(Borders::ALL).title(" Controls ").border_style(border_style));
        f.render_widget(instructions, chunks[0]);

        if focused {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input ").border_style(border_style));
            self.input.set_cursor_line_style(Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED));
        } else {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input "));
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[1]);

        let output_paragraph = Paragraph::new(self.output.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Output "));
        f.render_widget(output_paragraph, chunks[2]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }

        if key.code == KeyCode::Tab {
            self.mode = if self.mode == Mode::Format { Mode::Minify } else { Mode::Format };
            self.process();
            return Action::None;
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }
}
