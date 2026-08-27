use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::Rng;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn generate_mac<R: Rng + ?Sized>(rng: &mut R) -> String {
    let mut b = [0_u8; 6];
    rng.fill(&mut b);
    b[0] = (b[0] | 2) & 0xfe;
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        b[0], b[1], b[2], b[3], b[4], b[5]
    )
}
pub struct MacGenerator {
    macs: Vec<String>,
    count: usize,
}
impl MacGenerator {
    pub fn new() -> Self {
        let mut s = Self {
            macs: vec![],
            count: 5,
        };
        s.generate();
        s
    }
    fn generate(&mut self) {
        let mut rng = rand::thread_rng();
        self.macs = (0..self.count).map(|_| generate_mac(&mut rng)).collect();
    }
}
impl Tool for MacGenerator {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "mac-generator",
            name: "MAC Address Generator",
            category: Category::Network,
            description: "Generate random locally administered unicast MAC addresses.",
            keywords: &["mac", "address", "network", "random", "generator"],
        }
    }
    fn render(&mut self, f: &mut Frame, a: Rect, focused: bool) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(a);
        let style = if focused {
            crate::theme::Theme::default().border_active()
        } else {
            crate::theme::Theme::default().border_inactive()
        };
        f.render_widget(
            Paragraph::new("Enter: regenerate  +/-: count  Ctrl+C: copy").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Controls ")
                    .border_style(style),
            ),
            c[0],
        );
        let lines: Vec<Line> = self.macs.iter().map(|m| Line::from(m.as_str())).collect();
        f.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" MAC addresses ({}) ", self.count)),
            ),
            c[1],
        );
    }
    fn handle_key(&mut self, k: KeyEvent) -> Action {
        match k.code {
            KeyCode::Enter => {
                self.generate();
                Action::None
            }
            KeyCode::Char('+') => {
                self.count = (self.count + 1).min(50);
                self.generate();
                Action::None
            }
            KeyCode::Char('-') => {
                self.count = self.count.saturating_sub(1).max(1);
                self.generate();
                Action::None
            }
            KeyCode::Char('c') | KeyCode::Char('C')
                if k.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                copy_to_clipboard(self.macs.join("\n"))
            }
            KeyCode::Esc => Action::Back,
            _ => Action::None,
        }
    }
    fn help(&self) -> Vec<&'static str> {
        vec![
            "Enter: regenerate",
            "+/-: address count",
            "Ctrl+C: copy all",
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    #[test]
    fn generates_local_unicast() {
        let m = generate_mac(&mut StdRng::seed_from_u64(1));
        let first = u8::from_str_radix(&m[..2], 16).unwrap();
        assert_eq!(first & 2, 2);
        assert_eq!(first & 1, 0);
    }
}
