use super::{Action, Category, Tool, ToolMeta};
use urlencoding::{encode, decode};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tui_textarea::{Input, TextArea};

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

impl<'a> UrlEncoder<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input (Type here) "));
        Self {
            input,
            output: String::new(),
            mode: Mode::Encode,
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        self.output = match self.mode {
            Mode::Encode => encode(&text).into_owned(),
            Mode::Decode => {
                match decode(&text) {
                    Ok(decoded) => decoded.into_owned(),
                    Err(_) => "Invalid URL encoded string".to_string(),
                }
            }
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
            .constraints([Constraint::Length(3), Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        let border_style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

        let mode_str = if self.mode == Mode::Encode { "Encode" } else { "Decode" };
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
            self.mode = if self.mode == Mode::Encode { Mode::Decode } else { Mode::Encode };
            self.process();
            return Action::None;
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }
}
