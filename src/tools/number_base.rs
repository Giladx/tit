use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tui_textarea::{Input, TextArea};

pub struct NumberBase<'a> {
    input: TextArea<'a>,
    mode: Mode,
    dec: String,
    hex: String,
    bin: String,
    oct: String,
    error: String,
}

#[derive(PartialEq)]
enum Mode {
    Decimal,
    Hexadecimal,
    Binary,
    Octal,
}

impl<'a> NumberBase<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input (Type here) "));
        Self {
            input,
            mode: Mode::Decimal,
            dec: String::new(),
            hex: String::new(),
            bin: String::new(),
            oct: String::new(),
            error: String::new(),
        }
    }

    fn clear_fields(&mut self) {
        self.dec.clear();
        self.hex.clear();
        self.bin.clear();
        self.oct.clear();
        self.error.clear();
    }

    fn process(&mut self) {
        let text = self.input.lines().join("").trim().to_string();
        self.clear_fields();

        if text.is_empty() {
            return;
        }

        let parsed = match self.mode {
            Mode::Decimal => i64::from_str_radix(&text, 10),
            Mode::Hexadecimal => i64::from_str_radix(&text.replace("0x", ""), 16),
            Mode::Binary => i64::from_str_radix(&text.replace("0b", ""), 2),
            Mode::Octal => i64::from_str_radix(&text.replace("0o", ""), 8),
        };

        match parsed {
            Ok(num) => {
                self.dec = format!("{}", num);
                self.hex = format!("{:X}", num);
                self.bin = format!("{:b}", num);
                self.oct = format!("{:o}", num);
            }
            Err(_) => {
                self.error = "Invalid number for selected base.".to_string();
            }
        }
    }
}

impl<'a> Tool for NumberBase<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "number-base",
            name: "Number Base Converter",
            category: Category::Converter,
            description: "Convert between Decimal, Hex, Binary, and Octal.",
            keywords: &["number", "base", "decimal", "hexadecimal", "binary", "octal", "converter"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Mode selector
                Constraint::Length(3), // Input
                Constraint::Min(10),   // Results
            ].as_ref())
            .split(area);

        let border_style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

        let mode_str = match self.mode {
            Mode::Decimal => "Decimal (Base 10)",
            Mode::Hexadecimal => "Hexadecimal (Base 16)",
            Mode::Binary => "Binary (Base 2)",
            Mode::Octal => "Octal (Base 8)",
        };
        let instructions = Paragraph::new(format!("Input Base: {} (Press Tab to switch) | Esc to go back", mode_str))
            .block(Block::default().borders(Borders::ALL).title(" Controls ").border_style(border_style));
        f.render_widget(instructions, chunks[0]);

        if focused {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input Number ").border_style(border_style));
            self.input.set_cursor_line_style(Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED));
        } else {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input Number "));
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[1]);

        if !self.error.is_empty() {
            let error_p = Paragraph::new(self.error.as_str())
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title(" Parse Error "));
            f.render_widget(error_p, chunks[2]);
        } else {
            let parts_text = format!(
                "Decimal: {}\nHex:     {}\nBinary:  {}\nOctal:   {}",
                self.dec, self.hex, self.bin, self.oct
            );
            let parts_p = Paragraph::new(parts_text)
                .block(Block::default().borders(Borders::ALL).title(" Conversions "));
            f.render_widget(parts_p, chunks[2]);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }

        if key.code == KeyCode::Tab {
            self.mode = match self.mode {
                Mode::Decimal => Mode::Hexadecimal,
                Mode::Hexadecimal => Mode::Binary,
                Mode::Binary => Mode::Octal,
                Mode::Octal => Mode::Decimal,
            };
            self.process();
            return Action::None;
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }
}
