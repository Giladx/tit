use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use regex::Regex;
use tui_textarea::{Input, TextArea};
pub struct RegexTester<'a> {
    pattern: TextArea<'a>,
    text: TextArea<'a>,
    output: String,
    pattern_focus: bool,
}
pub fn test_regex(pattern: &str, text: &str) -> Result<String, String> {
    if pattern.is_empty() {
        return Ok(String::new());
    }
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex: {e}"))?;
    let matches: Vec<_> = re
        .find_iter(text)
        .enumerate()
        .map(|(i, m)| format!("{}: [{}..{}] {}", i + 1, m.start(), m.end(), m.as_str()))
        .collect();
    Ok(if matches.is_empty() {
        "No matches".into()
    } else {
        format!("{} match(es)\n{}", matches.len(), matches.join("\n"))
    })
}
impl<'a> RegexTester<'a> {
    pub fn new() -> Self {
        Self {
            pattern: TextArea::default(),
            text: TextArea::default(),
            output: String::new(),
            pattern_focus: true,
        }
    }
    fn process(&mut self) {
        self.output = test_regex(
            &self.pattern.lines().join(""),
            &self.text.lines().join("\n"),
        )
        .unwrap_or_else(|e| e)
    }
}
impl Tool for RegexTester<'_> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "regex",
            name: "Regex Tester",
            category: Category::Development,
            description: "Test Rust regular expressions and inspect match ranges.",
            keywords: &["regex", "regexp", "match", "pattern"],
        }
    }
    fn render(&mut self, f: &mut Frame, a: Rect, focused: bool) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Percentage(45),
                Constraint::Percentage(45),
            ])
            .split(a);
        let active = crate::theme::Theme::default().border_active();
        self.pattern.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Pattern ")
                .border_style(if focused && self.pattern_focus {
                    active
                } else {
                    crate::theme::Theme::default().border_inactive()
                }),
        );
        self.text.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Test text ")
                .border_style(if focused && !self.pattern_focus {
                    active
                } else {
                    crate::theme::Theme::default().border_inactive()
                }),
        );
        f.render_widget(&self.pattern, c[0]);
        f.render_widget(&self.text, c[1]);
        f.render_widget(
            Paragraph::new(self.output.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Matches ")),
            c[2],
        );
    }
    fn handle_key(&mut self, k: KeyEvent) -> Action {
        if k.code == KeyCode::Esc {
            return Action::Back;
        }
        if k.code == KeyCode::Tab {
            self.pattern_focus = !self.pattern_focus;
            return Action::None;
        }
        if matches!(k.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && k.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(self.output.clone());
        }
        let changed = if self.pattern_focus {
            self.pattern.input(Input::from(k))
        } else {
            self.text.input(Input::from(k))
        };
        if changed {
            self.process()
        }
        Action::None
    }
    fn help(&self) -> Vec<&'static str> {
        vec!["Tab: pattern/test text", "Ctrl+C: copy matches"]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_ranges() {
        let o = test_regex(r"\d+", "a12 b3").unwrap();
        assert!(o.contains("2 match(es)"));
    }
    #[test]
    fn bad_pattern() {
        assert!(test_regex("(", "x").is_err());
    }
}
