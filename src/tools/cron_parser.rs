use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};
pub struct CronParser<'a> {
    input: TextArea<'a>,
    output: String,
}
const NAMES: [&str; 5] = ["Minute", "Hour", "Day of month", "Month", "Day of week"];
const RANGES: [(u8, u8); 5] = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 7)];
fn validate_field(field: &str, (min, max): (u8, u8)) -> bool {
    field.split(',').all(|part| {
        let core = part.split_once('/').map_or(part, |(a, b)| {
            if b.parse::<u8>().is_ok_and(|n| n > 0) {
                a
            } else {
                "!"
            }
        });
        if core == "*" {
            return true;
        }
        if let Some((a, b)) = core.split_once('-') {
            return a.parse::<u8>().is_ok_and(|v| v >= min && v <= max)
                && b.parse::<u8>().is_ok_and(|v| v >= min && v <= max);
        }
        core.parse::<u8>().is_ok_and(|v| v >= min && v <= max)
    })
}
pub fn parse_cron(value: &str) -> Result<String, String> {
    let fields: Vec<_> = value.split_whitespace().collect();
    if fields.is_empty() {
        return Ok(String::new());
    }
    if fields.len() != 5 {
        return Err("Expected five fields: minute hour day-of-month month day-of-week".into());
    }
    for (i, field) in fields.iter().enumerate() {
        if !validate_field(field, RANGES[i]) {
            return Err(format!("Invalid {} field: {field}", NAMES[i]));
        }
    }
    Ok(fields
        .iter()
        .enumerate()
        .map(|(i, v)| format!("{}: {}", NAMES[i], describe(v)))
        .collect::<Vec<_>>()
        .join("\n"))
}
fn describe(v: &str) -> String {
    if v == "*" {
        "every value".into()
    } else if let Some(step) = v.strip_prefix("*/") {
        format!("every {step}")
    } else {
        v.into()
    }
}
impl<'a> CronParser<'a> {
    pub fn new() -> Self {
        Self {
            input: TextArea::default(),
            output: String::new(),
        }
    }
    fn process(&mut self) {
        self.output = parse_cron(&self.input.lines().join(" ")).unwrap_or_else(|e| e)
    }
}
impl Tool for CronParser<'_> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "cron",
            name: "Cron Parser",
            category: Category::Development,
            description: "Validate and explain standard five-field cron expressions.",
            keywords: &["cron", "schedule", "timer", "job"],
        }
    }
    fn render(&mut self, f: &mut Frame, a: Rect, focused: bool) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(5)])
            .split(a);
        self.input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Cron expression ")
                .border_style(if focused {
                    crate::theme::Theme::default().border_active()
                } else {
                    crate::theme::Theme::default().border_inactive()
                }),
        );
        f.render_widget(&self.input, c[0]);
        f.render_widget(
            Paragraph::new(self.output.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Explanation "),
            ),
            c[1],
        );
    }
    fn handle_key(&mut self, k: KeyEvent) -> Action {
        if k.code == KeyCode::Esc {
            return Action::Back;
        }
        if matches!(k.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && k.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(self.output.clone());
        }
        if self.input.input(Input::from(k)) {
            self.process()
        }
        Action::None
    }
    fn help(&self) -> Vec<&'static str> {
        vec![
            "Format: min hour dom month dow",
            "Supports *, lists, ranges, and steps",
            "Ctrl+C: copy explanation",
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses() {
        assert!(parse_cron("*/5 9-17 * * 1-5").unwrap().contains("every 5"));
    }
    #[test]
    fn rejects_range() {
        assert!(parse_cron("61 * * * *").is_err());
    }
}
