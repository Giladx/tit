use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use md5;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sha2::{Digest, Sha256, Sha512};
use tui_textarea::{Input, TextArea};

pub struct HashGenerator<'a> {
    input: TextArea<'a>,
    md5_out: String,
    sha256_out: String,
    sha512_out: String,
}

pub fn hash_text(text: &str) -> (String, String, String) {
    if text.is_empty() {
        return (String::new(), String::new(), String::new());
    }
    let md5_out = format!("{:x}", md5::compute(text.as_bytes()));
    let sha256_out = format!("{:x}", Sha256::digest(text.as_bytes()));
    let sha512_out = format!("{:x}", Sha512::digest(text.as_bytes()));
    (md5_out, sha256_out, sha512_out)
}

impl<'a> HashGenerator<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input (Type here) "),
        );
        Self {
            input,
            md5_out: String::new(),
            sha256_out: String::new(),
            sha512_out: String::new(),
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        (self.md5_out, self.sha256_out, self.sha512_out) = hash_text(&text);
    }
}

impl<'a> Tool for HashGenerator<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "hash-generator",
            name: "Hash Generator",
            category: Category::Crypto,
            description: "Generate MD5, SHA-256, and SHA-512 hashes from text.",
            keywords: &["hash", "md5", "sha256", "sha512", "crypto", "digest"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(40), // Input
                    Constraint::Length(3),      // MD5
                    Constraint::Length(3),      // SHA-256
                    Constraint::Length(3),      // SHA-512
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
                    .title(" Input Text (Esc to go back) ")
                    .border_style(border_style),
            );
            self.input.set_cursor_line_style(
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED),
            );
        } else {
            self.input
                .set_block(Block::default().borders(Borders::ALL).title(" Input Text "));
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[0]);

        let p_md5 = Paragraph::new(self.md5_out.as_str())
            .block(Block::default().borders(Borders::ALL).title(" MD5 "));
        f.render_widget(p_md5, chunks[1]);

        let p_sha256 = Paragraph::new(self.sha256_out.as_str())
            .block(Block::default().borders(Borders::ALL).title(" SHA-256 "));
        f.render_widget(p_sha256, chunks[2]);

        let p_sha512 = Paragraph::new(self.sha512_out.as_str())
            .block(Block::default().borders(Borders::ALL).title(" SHA-512 "));
        f.render_widget(p_sha512, chunks[3]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }
        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(format!(
                "MD5: {}\nSHA-256: {}\nSHA-512: {}",
                self.md5_out, self.sha256_out, self.sha512_out
            ));
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }

    fn help(&self) -> Vec<&'static str> {
        vec!["Type: generate hashes", "Ctrl+C: copy all"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        let (md5, sha256, sha512) = hash_text("hello");
        assert_eq!(md5, "5d41402abc4b2a76b9719d911017c592");
        assert_eq!(
            sha256,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert_eq!(
            sha512,
            "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
        );
    }

    #[test]
    fn empty_input_clears_outputs() {
        let (md5, sha256, sha512) = hash_text("");
        assert!(md5.is_empty() && sha256.is_empty() && sha512.is_empty());
    }

    #[test]
    fn unicode_hashes() {
        let (md5, _, _) = hash_text("שלום 🌍");
        assert_eq!(md5.len(), 32);
    }
}
