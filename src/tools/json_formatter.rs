use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;
use tui_textarea::{Input, TextArea};

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
fn format_json(text: &str, pretty: bool) -> Result<String, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?;
    if pretty {
        serde_json::to_string_pretty(&value)
    } else {
        serde_json::to_string(&value)
    }
    .map_err(|e| e.to_string())
}

impl<'a> JsonFormatter<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input (Type here) "),
        );
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

        self.output = format_json(&text, self.mode == Mode::Format).unwrap_or_else(|e| e);
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
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(area);

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let mode_str = if self.mode == Mode::Format {
            "Format"
        } else {
            "Minify"
        };
        let instructions = Paragraph::new(format!(
            "Mode: {} (Press Tab to switch) | Esc to go back",
            mode_str
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Controls ")
                .border_style(border_style),
        );
        f.render_widget(instructions, chunks[0]);

        if focused {
            self.input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input ")
                    .border_style(border_style),
            );
            self.input.set_cursor_line_style(
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED),
            );
        } else {
            self.input
                .set_block(Block::default().borders(Borders::ALL).title(" Input "));
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
            self.mode = if self.mode == Mode::Format {
                Mode::Minify
            } else {
                Mode::Format
            };
            self.process();
            return Action::None;
        }
        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(self.output.clone());
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }

    fn help(&self) -> Vec<&'static str> {
        vec!["Tab: format/minify", "Ctrl+C: copy output"]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minifies() {
        assert_eq!(format_json("{ \"a\": 1 }", false).unwrap(), "{\"a\":1}");
    }
    #[test]
    fn rejects() {
        assert!(format_json("{", true).is_err());
    }
}
