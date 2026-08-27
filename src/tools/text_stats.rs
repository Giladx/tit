use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tui_textarea::{Input, TextArea};

pub struct TextStats<'a> {
    input: TextArea<'a>,
    chars: usize,
    chars_no_spaces: usize,
    words: usize,
    lines: usize,
    bytes: usize,
}

impl<'a> TextStats<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input Text (Type here) "));
        Self {
            input,
            chars: 0,
            chars_no_spaces: 0,
            words: 0,
            lines: 0,
            bytes: 0,
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        if text.is_empty() {
            self.chars = 0;
            self.chars_no_spaces = 0;
            self.words = 0;
            self.lines = 0;
            self.bytes = 0;
            return;
        }

        self.chars = text.chars().count();
        self.chars_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();
        self.words = text.split_whitespace().count();
        self.lines = self.input.lines().len();
        self.bytes = text.len(); // bytes
    }
}

impl<'a> Tool for TextStats<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "text-statistics",
            name: "Text Statistics",
            category: Category::Text,
            description: "Count characters, words, lines, and bytes in text.",
            keywords: &["text", "statistics", "count", "words", "characters", "length"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70), // Input
                Constraint::Percentage(30), // Stats
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

        let stats_text = format!(
            "Characters (total): {}\nCharacters (no spaces): {}\nWords: {}\nLines: {}\nBytes: {}",
            self.chars, self.chars_no_spaces, self.words, self.lines, self.bytes
        );

        let p_stats = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title(" Statistics "));
        f.render_widget(p_stats, chunks[1]);
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
