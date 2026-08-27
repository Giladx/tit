use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub panel: Color,
    pub orange: Color,
    pub orange_dim: Color,
    pub text: Color,
    pub text_dim: Color,
    pub success: Color,
    pub error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(18, 18, 18),
            panel: Color::Rgb(28, 28, 28),
            orange: Color::Rgb(230, 160, 20),
            orange_dim: Color::Rgb(180, 120, 15),
            text: Color::Rgb(220, 220, 220),
            text_dim: Color::Rgb(140, 140, 140),
            success: Color::Rgb(80, 200, 120),
            error: Color::Rgb(220, 80, 80),
        }
    }
}

impl Theme {
    pub fn border_active(&self) -> Style {
        Style::default().fg(self.orange)
    }

    pub fn border_inactive(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    pub fn selected(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.orange)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn dim(&self) -> Style {
        Style::default().fg(self.text_dim)
    }
}
