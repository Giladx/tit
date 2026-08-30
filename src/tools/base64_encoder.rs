use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};

pub struct Base64Encoder<'a> {
    input: TextArea<'a>,
    output: String,
    mode: Mode,
}

#[derive(PartialEq)]
enum Mode {
    Encode,
    Decode,
}

pub fn encode_base64(text: &str) -> String {
    STANDARD.encode(text)
}
pub fn decode_base64(text: &str) -> Result<String, String> {
    let compact = text.replace(&['\n', '\r', ' '][..], "");
    let bytes = STANDARD
        .decode(compact)
        .map_err(|_| "Invalid Base64 sequence".to_string())?;
    String::from_utf8(bytes).map_err(|_| "Invalid UTF-8 sequence".to_string())
}

impl<'a> Base64Encoder<'a> {
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
            mode: Mode::Encode,
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        self.output = match self.mode {
            Mode::Encode => encode_base64(&text),
            Mode::Decode => decode_base64(&text).unwrap_or_else(|e| e),
        };
    }
}

impl<'a> Tool for Base64Encoder<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "base64-encoder",
            name: "Base64 Encoder/Decoder",
            category: Category::Converter,
            description: "Encode and decode Base64 strings.",
            keywords: &["base64", "encode", "decode", "converter"],
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

        let mode_str = if self.mode == Mode::Encode {
            "Encode"
        } else {
            "Decode"
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
            self.mode = if self.mode == Mode::Encode {
                Mode::Decode
            } else {
                Mode::Encode
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
        vec!["Tab: encode/decode", "Ctrl+C: copy output"]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_unicode() {
        let s = "שלום 🌍";
        assert_eq!(decode_base64(&encode_base64(s)).unwrap(), s);
    }
    #[test]
    fn rejects_invalid() {
        assert!(decode_base64("@@@").is_err());
    }
}
