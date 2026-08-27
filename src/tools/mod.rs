pub mod datetime;
pub mod uuid_generator;
pub mod base64_encoder;
pub mod url_encoder;
pub mod lorem_ipsum;
pub mod json_formatter;
pub mod hash_generator;
pub mod text_case_converter;
pub mod jwt_parser;
pub mod password_generator;
pub mod text_stats;

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    All,
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
            Self::All => "All Tools",
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
            Self::All,
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
        Box::new(uuid_generator::UuidGenerator::new()),
        Box::new(base64_encoder::Base64Encoder::new()),
        Box::new(url_encoder::UrlEncoder::new()),
        Box::new(lorem_ipsum::LoremIpsum::new()),
        Box::new(json_formatter::JsonFormatter::new()),
        Box::new(hash_generator::HashGenerator::new()),
        Box::new(text_case_converter::TextCaseConverter::new()),
        Box::new(jwt_parser::JwtParser::new()),
        Box::new(password_generator::PasswordGenerator::new()),
        Box::new(text_stats::TextStats::new()),
        // Add more tools here later
    ]
}
