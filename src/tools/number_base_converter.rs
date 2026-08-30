use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};

pub struct NumberBaseConverter<'a> {
    input: TextArea<'a>,
    output: String,
    base: u32,
}
pub fn convert_number(value: &str, base: u32) -> Result<String, String> {
    let cleaned = value.trim().replace('_', "");
    if cleaned.is_empty() {
        return Ok(String::new());
    }
    let negative = cleaned.starts_with('-');
    let digits = cleaned.trim_start_matches('-');
    let number = u128::from_str_radix(digits, base)
        .map_err(|e| format!("Invalid base-{base} number: {e}"))?;
    let number = if negative {
        if number > i128::MAX as u128 {
            return Err("Number too large for signed 128-bit range".into());
        }
        -(number as i128)
    } else {
        number as i128
    };
    Ok(format!(
        "Binary: {number:b}\nOctal: {number:o}\nDecimal: {number}\nHex: {number:X}"
    ))
}
impl<'a> NumberBaseConverter<'a> {
    pub fn new() -> Self {
        Self {
            input: TextArea::default(),
            output: String::new(),
            base: 10,
        }
    }
    fn process(&mut self) {
        self.output = convert_number(&self.input.lines().join(""), self.base).unwrap_or_else(|e| e)
    }
}
impl Tool for NumberBaseConverter<'_> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "number-base",
            name: "Number Base Converter",
            category: Category::Converter,
            description: "Convert signed integers between binary, octal, decimal, and hexadecimal.",
            keywords: &["binary", "hex", "octal", "decimal", "radix"],
        }
    }
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(5)])
            .split(area);
        self.input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Base {} input (Tab changes base) ", self.base))
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
                    .title(" Conversions "),
            ),
            c[1],
        );
    }
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }
        if key.code == KeyCode::Tab {
            self.base = match self.base {
                2 => 8,
                8 => 10,
                10 => 16,
                _ => 2,
            };
            self.process();
            return Action::None;
        }
        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(self.output.clone());
        }
        if self.input.input(Input::from(key)) {
            self.process()
        }
        Action::None
    }
    fn help(&self) -> Vec<&'static str> {
        vec!["Tab: input base 2/8/10/16", "Ctrl+C: copy conversions"]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts() {
        assert_eq!(
            convert_number("255", 10).unwrap(),
            "Binary: 11111111\nOctal: 377\nDecimal: 255\nHex: FF"
        );
    }
    #[test]
    fn negatives() {
        assert!(convert_number("-10", 10).unwrap().contains("Decimal: -10"));
    }
    #[test]
    fn large_unsigned() {
        let out = convert_number("ffffffffffffffffffffffffffffffff", 16).unwrap();
        assert!(out.contains("Decimal: -1"));
    }
    #[test]
    fn rejects_too_large_negative() {
        assert!(convert_number("-ffffffffffffffffffffffffffffffff", 16).is_err());
    }
}
