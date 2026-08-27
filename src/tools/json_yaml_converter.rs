use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;
use tui_textarea::{Input, TextArea};
pub fn json_to_yaml(text: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?;
    serde_yaml::to_string(&value).map_err(|e| format!("YAML error: {e}"))
}
pub fn yaml_to_json(text: &str) -> Result<String, String> {
    let value: Value = serde_yaml::from_str(text).map_err(|e| format!("Invalid YAML: {e}"))?;
    serde_json::to_string_pretty(&value).map_err(|e| format!("JSON error: {e}"))
}
#[derive(PartialEq)]
enum Mode {
    JsonToYaml,
    YamlToJson,
}
pub struct JsonYamlConverter<'a> {
    input: TextArea<'a>,
    output: String,
    mode: Mode,
}
impl<'a> JsonYamlConverter<'a> {
    pub fn new() -> Self {
        Self {
            input: TextArea::default(),
            output: String::new(),
            mode: Mode::JsonToYaml,
        }
    }
    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        if text.trim().is_empty() {
            self.output.clear()
        } else {
            self.output = match self.mode {
                Mode::JsonToYaml => json_to_yaml(&text),
                Mode::YamlToJson => yaml_to_json(&text),
            }
            .unwrap_or_else(|e| e)
        }
    }
}
impl Tool for JsonYamlConverter<'_> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "json-yaml",
            name: "JSON ↔ YAML Converter",
            category: Category::Development,
            description: "Convert documents between JSON and YAML.",
            keywords: &["json", "yaml", "convert", "format"],
        }
    }
    fn render(&mut self, f: &mut Frame, a: Rect, focused: bool) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(a);
        let mode = if self.mode == Mode::JsonToYaml {
            "JSON → YAML"
        } else {
            "YAML → JSON"
        };
        f.render_widget(
            Paragraph::new(format!("Mode: {mode} (Tab switches)"))
                .block(Block::default().borders(Borders::ALL).title(" Controls ")),
            c[0],
        );
        self.input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input ")
                .border_style(if focused {
                    crate::theme::Theme::default().border_active()
                } else {
                    crate::theme::Theme::default().border_inactive()
                }),
        );
        f.render_widget(&self.input, c[1]);
        f.render_widget(
            Paragraph::new(self.output.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Output ")),
            c[2],
        );
    }
    fn handle_key(&mut self, k: KeyEvent) -> Action {
        if k.code == KeyCode::Esc {
            return Action::Back;
        }
        if k.code == KeyCode::Tab {
            self.mode = if self.mode == Mode::JsonToYaml {
                Mode::YamlToJson
            } else {
                Mode::JsonToYaml
            };
            self.process();
            return Action::None;
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
        vec!["Tab: conversion direction", "Ctrl+C: copy output"]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let yaml = json_to_yaml(r#"{"name":"tit","ok":true}"#).unwrap();
        let json = yaml_to_json(&yaml).unwrap();
        assert!(json.contains("\"name\": \"tit\""));
    }
    #[test]
    fn rejects_bad_yaml() {
        assert!(yaml_to_json("[bad").is_err());
    }
}
