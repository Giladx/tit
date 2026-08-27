use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use base64::{
    engine::general_purpose::URL_SAFE, engine::general_purpose::URL_SAFE_NO_PAD, Engine as _,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;
use tui_textarea::{Input, TextArea};

pub struct JwtParser<'a> {
    input: TextArea<'a>,
    header_out: String,
    payload_out: String,
    signature_out: String,
}

impl<'a> JwtParser<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input JWT (Type here) "),
        );
        Self {
            input,
            header_out: String::new(),
            payload_out: String::new(),
            signature_out: String::new(),
        }
    }

    fn decode_part(part: &str) -> String {
        if part.is_empty() {
            return String::new();
        }

        let decode_result = URL_SAFE_NO_PAD.decode(part).or_else(|_| {
            let mut p = part.to_string();
            while !p.len().is_multiple_of(4) {
                p.push('=');
            }
            URL_SAFE.decode(&p)
        });

        match decode_result {
            Ok(b) => {
                if let Ok(s) = String::from_utf8(b) {
                    if let Ok(v) = serde_json::from_str::<Value>(&s) {
                        return serde_json::to_string_pretty(&v).unwrap_or(s);
                    }
                    return s;
                }
                "Invalid UTF-8".to_string()
            }
            Err(_) => "Invalid Base64Url sequence".to_string(),
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("").replace(['\n', '\r', ' '], "");
        if text.is_empty() {
            self.header_out.clear();
            self.payload_out.clear();
            self.signature_out.clear();
            return;
        }

        let parts: Vec<&str> = text.split('.').collect();

        self.header_out = if !parts.is_empty() {
            Self::decode_part(parts[0])
        } else {
            String::new()
        };
        self.payload_out = if parts.len() > 1 {
            Self::decode_part(parts[1])
        } else {
            String::new()
        };
        self.signature_out = if parts.len() > 2 {
            parts[2].to_string()
        } else {
            String::new()
        };
    }
}

impl<'a> Tool for JwtParser<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "jwt-parser",
            name: "JWT Parser",
            category: Category::Development,
            description: "Decode JWT contents without verifying signatures.",
            keywords: &["jwt", "decode", "parser", "token", "json web token"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(25), // Input
                    Constraint::Percentage(25), // Header
                    Constraint::Percentage(40), // Payload
                    Constraint::Percentage(10), // Signature
                ]
                .as_ref(),
            )
            .split(area);

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        if focused {
            self.input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Token (Esc to go back) ")
                    .border_style(border_style),
            );
            self.input.set_cursor_line_style(
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED),
            );
        } else {
            self.input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Token "),
            );
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[0]);

        let p_header = Paragraph::new(self.header_out.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Header (Algorithm & Token Type) "),
        );
        f.render_widget(p_header, chunks[1]);

        let p_payload = Paragraph::new(self.payload_out.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Payload (Data) "),
        );
        f.render_widget(p_payload, chunks[2]);

        let p_signature = Paragraph::new(self.signature_out.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Signature "));
        f.render_widget(p_signature, chunks[3]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }

        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(format!(
                "Header:\n{}\n\nPayload:\n{}\n\nSignature:\n{}",
                self.header_out, self.payload_out, self.signature_out
            ));
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }

    fn help(&self) -> Vec<&'static str> {
        vec![
            "Decode only; no signature verification",
            "Ctrl+C: copy decoded token",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_json_payload() {
        assert!(JwtParser::decode_part("eyJzdWIiOiIxMjMifQ").contains("\"sub\": \"123\""));
    }

    #[test]
    fn rejects_invalid_base64url() {
        assert_eq!(JwtParser::decode_part("!!!"), "Invalid Base64Url sequence");
    }
}
