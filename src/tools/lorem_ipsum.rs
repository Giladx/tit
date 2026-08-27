use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct LoremIpsum {
    paragraphs: usize,
    text: String,
}

impl LoremIpsum {
    pub fn new() -> Self {
        let mut t = Self {
            paragraphs: 3,
            text: String::new(),
        };
        t.generate();
        t
    }

    fn generate(&mut self) {
        let lipsum = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        self.text = vec![lipsum; self.paragraphs].join("\n\n");
    }
}

impl Tool for LoremIpsum {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "lorem-ipsum",
            name: "Lorem Ipsum",
            category: Category::Text,
            description: "Generate placeholder text.",
            keywords: &["lorem", "ipsum", "placeholder", "text", "generator"],
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
            Span::styled("+", Style::default().fg(Color::Yellow)),
            Span::raw("/"),
            Span::styled("-", Style::default().fg(Color::Yellow)),
            Span::raw(" to change paragraphs, "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
            Span::raw(" to copy."),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Controls ")
                .border_style(border_style),
        );

        f.render_widget(instructions, chunks[0]);

        let text_paragraph = Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Text ({} paragraphs) ", self.paragraphs))
                    .border_style(border_style),
            );

        f.render_widget(text_paragraph, chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('+') => {
                self.paragraphs = self.paragraphs.saturating_add(1).min(50);
                self.generate();
                Action::None
            }
            KeyCode::Char('-') => {
                self.paragraphs = self.paragraphs.saturating_sub(1).max(1);
                self.generate();
                Action::None
            }
            KeyCode::Char('c') | KeyCode::Char('C')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                copy_to_clipboard(self.text.clone())
            }
            KeyCode::Esc => Action::Back,
            _ => Action::None,
        }
    }

    fn help(&self) -> Vec<&'static str> {
        vec!["+/-: paragraph count", "Ctrl+C: copy"]
    }
}
