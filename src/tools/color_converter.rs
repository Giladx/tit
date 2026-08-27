use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};
pub struct ColorConverter<'a> {
    input: TextArea<'a>,
    output: String,
}
pub fn convert_color(value: &str) -> Result<String, String> {
    let s = value.trim().trim_start_matches('#');
    if s.is_empty() {
        return Ok(String::new());
    }
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Enter a six-digit hex color such as #E6A014".into());
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap();
    let g = u8::from_str_radix(&s[2..4], 16).unwrap();
    let b = u8::from_str_radix(&s[4..6], 16).unwrap();
    let max = r.max(g).max(b) as f32 / 255.;
    let min = r.min(g).min(b) as f32 / 255.;
    let l = (max + min) / 2.;
    let d = max - min;
    let (h, sat) = if d == 0. {
        (0., 0.)
    } else {
        let sat = d / (1. - (2. * l - 1.).abs());
        let h = if max == r as f32 / 255. {
            60. * (((g as f32 - b as f32) / 255. / d) % 6.)
        } else if max == g as f32 / 255. {
            60. * (((b as f32 - r as f32) / 255. / d) + 2.)
        } else {
            60. * (((r as f32 - g as f32) / 255. / d) + 4.)
        };
        (if h < 0. { h + 360. } else { h }, sat)
    };
    Ok(format!(
        "HEX: #{}\nRGB: rgb({r}, {g}, {b})\nHSL: hsl({:.0}, {:.0}%, {:.0}%)",
        s.to_uppercase(),
        h,
        sat * 100.,
        l * 100.
    ))
}
impl<'a> ColorConverter<'a> {
    pub fn new() -> Self {
        Self {
            input: TextArea::default(),
            output: String::new(),
        }
    }
    fn process(&mut self) {
        self.output = convert_color(&self.input.lines().join("")).unwrap_or_else(|e| e)
    }
}
impl Tool for ColorConverter<'_> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "color",
            name: "Color Converter",
            category: Category::Converter,
            description: "Convert six-digit HEX colors to RGB and HSL.",
            keywords: &["color", "hex", "rgb", "hsl"],
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
                .title(" HEX color ")
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
                    .title(" Color values "),
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
        vec!["Input: #RRGGBB", "Ctrl+C: copy values"]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orange() {
        let o = convert_color("#E6A014").unwrap();
        assert!(o.contains("rgb(230, 160, 20)"));
    }
    #[test]
    fn invalid() {
        assert!(convert_color("red").is_err());
    }
}
