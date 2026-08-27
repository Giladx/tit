use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct MacGenerator {
    macs: Vec<String>,
    count: usize,
}

impl MacGenerator {
    pub fn new() -> Self {
        let mut t = Self {
            macs: Vec::new(),
            count: 5,
        };
        t.generate();
        t
    }

    fn generate(&mut self) {
        self.macs.clear();
        let mut rng = rand::thread_rng();

        for _ in 0..self.count {
            let mut bytes = [0u8; 6];
            rng.fill(&mut bytes);
            // Set locally administered, unicast
            bytes[0] = (bytes[0] | 0b0000_0010) & 0b1111_1110;

            let mac = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]);
            self.macs.push(mac);
        }
    }
}

impl Tool for MacGenerator {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "mac-generator",
            name: "MAC Address Generator",
            category: Category::Network,
            description: "Generate random MAC addresses.",
            keywords: &["mac", "address", "generator", "random", "network"],
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
            Span::raw(" to change count, "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
            Span::raw(" to copy all."),
        ]))
        .block(Block::default().borders(Borders::ALL).title(" Controls ").border_style(border_style));

        f.render_widget(instructions, chunks[0]);

        let lines: Vec<Line> = self.macs.iter().map(|m| Line::from(m.as_str())).collect();
        let text_paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(format!(" MAC Addresses ({}) ", self.count)).border_style(border_style));

        f.render_widget(text_paragraph, chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                self.generate();
                Action::None
            }
            KeyCode::Char('+') => {
                self.count = self.count.saturating_add(1).min(50);
                self.generate();
                Action::None
            }
            KeyCode::Char('-') => {
                self.count = self.count.saturating_sub(1).max(1);
                self.generate();
                Action::None
            }
            KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(self.macs.join("\n"));
                    return Action::Copied;
                }
                Action::None
            }
            KeyCode::Esc => Action::Back,
            _ => Action::None,
        }
    }
}
