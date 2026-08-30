use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use uuid::Uuid;

pub struct UuidGenerator {
    uuids: Vec<String>,
    count: usize,
}

impl UuidGenerator {
    pub fn new() -> Self {
        let mut t = Self {
            uuids: Vec::new(),
            count: 5,
        };
        t.generate();
        t
    }

    fn generate(&mut self) {
        self.uuids.clear();
        for _ in 0..self.count {
            self.uuids.push(Uuid::new_v4().to_string());
        }
    }
}

impl Tool for UuidGenerator {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "uuid-generator",
            name: "UUID Generator",
            category: Category::Generator,
            description: "Generate v4 UUIDs.",
            keywords: &["uuid", "guid", "generator", "random"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)].as_ref())
            .split(area);

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Controls ")
                .border_style(border_style),
        );

        f.render_widget(instructions, chunks[0]);

        let uuids_text: Vec<Line> = self.uuids.iter().map(|u| Line::from(u.as_str())).collect();
        let uuids_paragraph = Paragraph::new(uuids_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" UUIDs ({}) ", self.count))
                .border_style(border_style),
        );

        f.render_widget(uuids_paragraph, chunks[1]);
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
            KeyCode::Char('c') | KeyCode::Char('C')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                copy_to_clipboard(self.uuids.join("\n"))
            }
            KeyCode::Esc => Action::Back,
            _ => Action::None,
        }
    }

    fn help(&self) -> Vec<&'static str> {
        vec!["Enter: regenerate", "+/-: change count", "Ctrl+C: copy all"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_uuid_v4() {
        let mut tool = UuidGenerator::new();
        tool.count = 1;
        tool.generate();
        let uuid = tool.uuids.first().unwrap();
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().nth(14).unwrap(), '4');
        assert!(uuid.chars().nth(19).unwrap().is_ascii_hexdigit());
    }

    #[test]
    fn generates_requested_count() {
        let mut tool = UuidGenerator::new();
        tool.count = 7;
        tool.generate();
        assert_eq!(tool.uuids.len(), 7);
        assert!(tool.uuids.iter().all(|u| u.parse::<Uuid>().is_ok()));
    }

    #[test]
    fn generated_uuids_are_unique() {
        let mut tool = UuidGenerator::new();
        tool.count = 20;
        tool.generate();
        let unique: std::collections::HashSet<_> = tool.uuids.iter().collect();
        assert_eq!(unique.len(), tool.uuids.len());
    }
}
