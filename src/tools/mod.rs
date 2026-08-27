pub mod base64_encoder;
pub mod color_converter;
pub mod cron_parser;
pub mod datetime;
pub mod hash_generator;
pub mod json_formatter;
pub mod jwt_parser;
pub mod lorem_ipsum;
pub mod number_base_converter;
pub mod password_generator;
pub mod regex_tester;
pub mod text_case_converter;
pub mod text_stats;
pub mod url_encoder;
pub mod uuid_generator;

pub mod html_entities;
pub mod ipv4_subnet;
pub mod json_yaml_converter;
pub mod mac_generator;
pub mod url_parser;

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

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
    Back,
    Status(String),
}

pub trait Tool {
    fn meta(&self) -> ToolMeta;
    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> Action;
    fn on_focus(&mut self) {}
    fn help(&self) -> Vec<&'static str> {
        vec!["Type to update", "Esc: back"]
    }
}

pub fn copy_to_clipboard(value: String) -> Action {
    match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(value)) {
        Ok(()) => Action::Status("Copied to clipboard".into()),
        Err(error) => Action::Status(format!("Clipboard error: {error}")),
    }
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
        Box::new(html_entities::HtmlEntities::new()),
        Box::new(url_parser::UrlParser::new()),
        Box::new(ipv4_subnet::Ipv4Subnet::new()),
        Box::new(mac_generator::MacGenerator::new()),
        Box::new(json_yaml_converter::JsonYamlConverter::new()),
        Box::new(number_base_converter::NumberBaseConverter::new()),
        Box::new(regex_tester::RegexTester::new()),
        Box::new(color_converter::ColorConverter::new()),
        Box::new(cron_parser::CronParser::new()),
    ]
}
