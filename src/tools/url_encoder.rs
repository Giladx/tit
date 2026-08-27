use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};
use urlencoding::{decode, encode};

pub struct UrlEncoder<'a> {
    input: TextArea<'a>,
    output: String,
    mode: Mode,
}

#[derive(PartialEq)]
enum Mode {
    Encode,
    Decode,
}
fn encode_url(text: &str) -> String {
    encode(text).into_owned()
}
fn decode_url(text: &str) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return Err("Invalid URL encoded string".into());
            }
            index += 3;
        } else {
            index += 1;
        }
    }
    decode(text)
        .map(|v| v.into_owned())
        .map_err(|_| "Invalid URL encoded string".into())
}

impl<'a> UrlEncoder<'a> {
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
            Mode::Encode => encode_url(&text),
            Mode::Decode => decode_url(&text).unwrap_or_else(|e| e),
        };
    }
}
impl<'a> Tool for UrlEncoder<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "url-encoder",
            name: "URL Encoder/Decoder",
            category: Category::Converter,
            description: "Encode and decode URL-encoded strings.",
            keywords: &["url", "encode", "decode", "converter", "uri"],
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
    fn round_trip() {
        let s = "a b/שלום";
        assert_eq!(decode_url(&encode_url(s)).unwrap(), s);
    }
    #[test]
    fn rejects_bad_percent() {
        assert!(decode_url("%GG").is_err());
    }
}
