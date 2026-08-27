use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tui_textarea::{Input, TextArea};
use md5;
use sha2::{Sha256, Sha512, Digest};

pub struct HashGenerator<'a> {
    input: TextArea<'a>,
    md5_out: String,
    sha256_out: String,
    sha512_out: String,
}

impl<'a> HashGenerator<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input (Type here) "));
        Self {
            input,
            md5_out: String::new(),
            sha256_out: String::new(),
            sha512_out: String::new(),
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        if text.is_empty() {
            self.md5_out.clear();
            self.sha256_out.clear();
            self.sha512_out.clear();
            return;
        }

        self.md5_out = format!("{:x}", md5::compute(text.as_bytes()));
        self.sha256_out = format!("{:x}", Sha256::digest(text.as_bytes()));
        self.sha512_out = format!("{:x}", Sha512::digest(text.as_bytes()));
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
            .constraints([
                Constraint::Percentage(40), // Input
                Constraint::Length(3),      // MD5
                Constraint::Length(3),      // SHA-256
                Constraint::Length(3),      // SHA-512
            ].as_ref())
            .split(area);

        let border_style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

        if focused {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input Text (Esc to go back) ").border_style(border_style));
            self.input.set_cursor_line_style(Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED));
        } else {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input Text "));
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

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }
}
