use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::{distributions::Alphanumeric, Rng};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct PasswordGenerator {
    passwords: Vec<String>,
    length: usize,
    count: usize,
}

impl PasswordGenerator {
    pub fn new() -> Self {
        let mut t = Self {
            passwords: Vec::new(),
            length: 16,
            count: 5,
        };
        t.generate();
        t
    }

    fn generate(&mut self) {
        self.passwords.clear();
        let mut rng = rand::thread_rng();

        // Allowed characters (mixing uppercase, lowercase, numbers, symbols)
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";

        for _ in 0..self.count {
            let pwd: String = (0..self.length)
                .map(|_| {
                    let idx = rng.gen_range(0..chars.len());
                    chars[idx] as char
                })
                .collect();
            self.passwords.push(pwd);
        }
    }
}

impl Tool for PasswordGenerator {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "password-generator",
            name: "Password Generator",
            category: Category::Generator,
            description: "Generate secure, random passwords.",
            keywords: &["password", "generator", "random", "secure", "secret"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)].as_ref())
            .split(area);

        let border_style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

        let instructions = Paragraph::new(Line::from(vec![
            Span::raw("Press "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" to generate new, "),
            Span::styled("+", Style::default().fg(Color::Yellow)),
            Span::raw("/"),
            Span::styled("-", Style::default().fg(Color::Yellow)),
            Span::raw(" to change length, "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
            Span::raw(" to copy all."),
        ]))
        .block(Block::default().borders(Borders::ALL).title(" Controls ").border_style(border_style));

        f.render_widget(instructions, chunks[0]);

        let lines: Vec<Line> = self.passwords.iter().map(|p| Line::from(p.as_str())).collect();
        let text_paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(format!(" Passwords (Length: {}) ", self.length)).border_style(border_style));

        f.render_widget(text_paragraph, chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                self.generate();
                Action::None
            }
            KeyCode::Char('+') => {
                self.length = self.length.saturating_add(1).min(128);
                self.generate();
                Action::None
            }
            KeyCode::Char('-') => {
                self.length = self.length.saturating_sub(1).max(4);
                self.generate();
                Action::None
            }
            KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(self.passwords.join("\n"));
                    return Action::Copied;
                }
                Action::None
            }
            KeyCode::Esc => Action::Back,
            _ => Action::None,
        }
    }
}
