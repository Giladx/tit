pub mod datetime;

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Converter,
    Crypto,
    Text,
    Network,
    Development,
    Generator,
}

impl Category {
    pub fn name(self) -> &'static str {
        match self {
            Self::Converter => "Converter",
            Self::Crypto => "Crypto",
            Self::Text => "Text",
            Self::Network => "Network",
            Self::Development => "Development",
            Self::Generator => "Generator",
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Self::Converter,
            Self::Crypto,
            Self::Text,
            Self::Network,
            Self::Development,
            Self::Generator,
        ]
    }
}

pub struct ToolMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub description: &'static str,
    pub keywords: &'static [&'static str],
}

pub enum Action {
    None,
    Quit,
    Back,
    Copied,
}

pub trait Tool {
    fn meta(&self) -> ToolMeta;
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> Action;
    fn on_focus(&mut self) {}
}

/// Registry of all tools
pub fn all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(datetime::DateTimeConverter::new()),
        // Add more tools here later
    ]
}
