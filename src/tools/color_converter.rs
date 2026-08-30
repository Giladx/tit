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

#[derive(Clone, Copy, PartialEq, Debug)]
enum ColorInput {
    Hex,
    Rgb,
    Hsl,
}

fn parse_rgb(text: &str) -> Option<(u8, u8, u8)> {
    let trimmed = text.trim();
    let inner = trimmed
        .strip_prefix("rgb(")
        .or_else(|| trimmed.strip_prefix("RGBA("))
        .or_else(|| trimmed.strip_prefix("rgba("))
        .or_else(|| trimmed.strip_prefix("RGB("))?
        .trim_end_matches(')');
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.len() < 3 {
        return None;
    }
    let r = parts[0].parse::<u8>().ok()?;
    let g = parts[1].parse::<u8>().ok()?;
    let b = parts[2].parse::<u8>().ok()?;
    Some((r, g, b))
}

fn parse_hsl(text: &str) -> Option<(u16, u8, u8)> {
    let trimmed = text.trim();
    let inner = trimmed
        .strip_prefix("hsl(")
        .or_else(|| trimmed.strip_prefix("HSL("))?
        .trim_end_matches(')')
        .replace("%", "");
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return None;
    }
    let h = parts[0].parse::<u16>().ok()?;
    let s = parts[1].parse::<u8>().ok()?;
    let l = parts[2].parse::<u8>().ok()?;
    if h > 360 || s > 100 || l > 100 {
        return None;
    }
    Some((h, s, l))
}

fn hsl_to_rgb(h: u16, s: u8, l: u8) -> (u8, u8, u8) {
    let s = s as f32 / 100.0;
    let l = l as f32 / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h as f32 / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h_prime {
        _ if h_prime < 1.0 => (c, x, 0.0),
        _ if h_prime < 2.0 => (x, c, 0.0),
        _ if h_prime < 3.0 => (0.0, c, x),
        _ if h_prime < 4.0 => (0.0, x, c),
        _ if h_prime < 5.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;
    (r, g, b)
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;

    if d == 0.0 {
        return (0, 0, (l * 100.0).round() as u8);
    }

    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };

    (
        h.round() as u16,
        (s * 100.0).round() as u8,
        (l * 100.0).round() as u8,
    )
}

fn detect_input_type(value: &str) -> Option<ColorInput> {
    let trimmed = value.trim();
    if trimmed.starts_with("rgb(") || trimmed.starts_with("rgba(") {
        return Some(ColorInput::Rgb);
    }
    if trimmed.starts_with("hsl(") {
        return Some(ColorInput::Hsl);
    }
    if trimmed.starts_with('#') || trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(ColorInput::Hex);
    }
    None
}

fn parse_hex(value: &str) -> Option<(u8, u8, u8)> {
    let s = value.trim().trim_start_matches('#');
    match s.len() {
        3 => {
            let chars: Vec<char> = s.chars().collect();
            let r = u8::from_str_radix(&format!("{}{}", chars[0], chars[0]), 16).ok()?;
            let g = u8::from_str_radix(&format!("{}{}", chars[1], chars[1]), 16).ok()?;
            let b = u8::from_str_radix(&format!("{}{}", chars[2], chars[2]), 16).ok()?;
            Some((r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

pub fn convert_color(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let input_type =
        detect_input_type(trimmed).ok_or("Unsupported input. Use HEX, RGB, or HSL.")?;
    let (r, g, b) = match input_type {
        ColorInput::Hex => parse_hex(trimmed).ok_or("Invalid HEX color. Use #RGB or #RRGGBB.")?,
        ColorInput::Rgb => parse_rgb(trimmed).ok_or("Invalid RGB color. Use rgb(r, g, b).")?,
        ColorInput::Hsl => {
            let (h, s, l) = parse_hsl(trimmed).ok_or("Invalid HSL color. Use hsl(h, s%, l%).")?;
            hsl_to_rgb(h, s, l)
        }
    };

    let (h, s, l) = rgb_to_hsl(r, g, b);
    let hex = format!("{:02X}{:02X}{:02X}", r, g, b);

    Ok(format!(
        "HEX: #{hex}\nRGB: rgb({r}, {g}, {b})\nHSL: hsl({h}, {s}%, {l}%)",
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
            description: "Convert between HEX, RGB, and HSL color formats.",
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
                .title(" Color (HEX, RGB, HSL) ")
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
        vec!["Input: #RGB, #RRGGBB, rgb(), hsl()", "Ctrl+C: copy values"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orange_hex() {
        let o = convert_color("#E6A014").unwrap();
        assert!(o.contains("rgb(230, 160, 20)"));
    }
    #[test]
    fn short_hex() {
        let o = convert_color("#FA0").unwrap();
        assert!(o.contains("rgb(255, 170, 0)"));
    }
    #[test]
    fn rgb_input() {
        let o = convert_color("rgb(255, 0, 128)").unwrap();
        assert!(o.contains("HEX: #FF0080"));
    }
    #[test]
    fn hsl_input() {
        let o = convert_color("hsl(120, 100%, 50%)").unwrap();
        assert!(o.contains("HEX: #00FF00"));
    }
    #[test]
    fn invalid() {
        assert!(convert_color("red").is_err());
    }
    #[test]
    fn empty() {
        assert_eq!(convert_color("").unwrap(), "");
    }
}
