use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use serde_json::Value as JsonValue;
use tui_textarea::{Input, TextArea};

pub struct JsonYamlConverter<'a> {
    input: TextArea<'a>,
    output: String,
    mode: Mode,
}

#[derive(PartialEq)]
enum Mode {
    JsonToYaml,
    YamlToJson,
}

impl<'a> JsonYamlConverter<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input (Type here) "));
        Self {
            input,
            output: String::new(),
            mode: Mode::JsonToYaml,
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        if text.trim().is_empty() {
            self.output.clear();
            return;
        }

        self.output = match self.mode {
            Mode::JsonToYaml => {
                match serde_json::from_str::<JsonValue>(&text) {
                    Ok(val) => serde_yaml::to_string(&val).unwrap_or_else(|_| "Error converting to YAML".to_string()),
                    Err(e) => format!("Invalid JSON: {}", e),
                }
            }
            Mode::YamlToJson => {
                match serde_yaml::from_str::<JsonValue>(&text) {
                    Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| "Error converting to JSON".to_string()),
                    Err(e) => format!("Invalid YAML: {}", e),
                }
            }
        };
    }
}

impl<'a> Tool for JsonYamlConverter<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "json-yaml-converter",
            name: "JSON ↔ YAML Converter",
            category: Category::Development,
            description: "Convert between JSON and YAML.",
            keywords: &["json", "yaml", "converter", "format"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        let border_style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

        let mode_str = if self.mode == Mode::JsonToYaml { "JSON -> YAML" } else { "YAML -> JSON" };
        let instructions = Paragraph::new(format!("Mode: {} (Press Tab to switch) | Esc to go back", mode_str))
            .block(Block::default().borders(Borders::ALL).title(" Controls ").border_style(border_style));
        f.render_widget(instructions, chunks[0]);

        if focused {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input ").border_style(border_style));
            self.input.set_cursor_line_style(Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED));
        } else {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input "));
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[1]);

        let output_paragraph = Paragraph::new(self.output.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Output "));
        f.render_widget(output_paragraph, chunks[2]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }

        if key.code == KeyCode::Tab {
            self.mode = if self.mode == Mode::JsonToYaml { Mode::YamlToJson } else { Mode::JsonToYaml };
            self.process();
            return Action::None;
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }
}
