use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};

pub struct TextCaseConverter<'a> {
    input: TextArea<'a>,
    output: String,
    mode_list_state: ListState,
    modes: Vec<&'static str>,
    focus_input: bool,
}

impl<'a> TextCaseConverter<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input Text (Press Tab to switch pane) "),
        );

        let modes = vec![
            "Lowercase",
            "Uppercase",
            "Title Case",
            "Camel Case",
            "Snake Case",
            "Kebab Case",
        ];
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            input,
            output: String::new(),
            mode_list_state: state,
            modes,
            focus_input: true,
        }
    }

    fn process(&mut self) {
        let text = self.input.lines().join("\n");
        let mode_idx = self.mode_list_state.selected().unwrap_or(0);
        let mode = self.modes[mode_idx];
        self.output = convert_case(&text, mode).unwrap_or_default();
    }
}

pub fn convert_case(text: &str, mode: &str) -> Result<String, String> {
    match mode.to_ascii_lowercase().as_str() {
        "lowercase" => Ok(text.to_lowercase()),
        "uppercase" => Ok(text.to_uppercase()),
        "title" | "title case" => Ok(text
            .split_whitespace()
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")),
        "camel" | "camel case" => Ok(text
            .split_whitespace()
            .enumerate()
            .map(|(i, word)| {
                let word = word.to_lowercase();
                if i == 0 {
                    word
                } else {
                    let mut c = word.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                }
            })
            .collect::<String>()),
        "snake" | "snake case" => Ok(text
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("_")),
        "kebab" | "kebab case" => Ok(text
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")),
        _ => Err(format!("Unknown case mode: {mode}")),
    }
}

impl<'a> Tool for TextCaseConverter<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "text-case-converter",
            name: "Text Case Converter",
            category: Category::Text,
            description: "Convert text to various casings.",
            keywords: &[
                "text",
                "case",
                "lowercase",
                "uppercase",
                "camel",
                "snake",
                "kebab",
            ],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);

        let border_style_focus = Style::default().fg(Color::Yellow);

        let list_items: Vec<ListItem> = self.modes.iter().map(|m| ListItem::new(*m)).collect();
        let modes_list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Modes ")
                    .border_style(if focused && !self.focus_input {
                        border_style_focus
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(modes_list, left_chunks[0], &mut self.mode_list_state);

        if focused && self.focus_input {
            self.input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Text (Press Tab to select Mode) ")
                    .border_style(border_style_focus),
            );
            self.input.set_cursor_line_style(
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED),
            );
        } else {
            self.input
                .set_block(Block::default().borders(Borders::ALL).title(" Input Text "));
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, right_chunks[0]);

        let output_paragraph = Paragraph::new(self.output.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Output "));
        f.render_widget(output_paragraph, right_chunks[1]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }

        if key.code == KeyCode::Tab {
            self.focus_input = !self.focus_input;
            return Action::None;
        }
        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(self.output.clone());
        }

        if self.focus_input {
            if self.input.input(Input::from(key)) {
                self.process();
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let mut i = self.mode_list_state.selected().unwrap_or(0);
                    i = i.saturating_sub(1);
                    self.mode_list_state.select(Some(i));
                    self.process();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let mut i = self.mode_list_state.selected().unwrap_or(0);
                    if i < self.modes.len() - 1 {
                        i += 1;
                    }
                    self.mode_list_state.select(Some(i));
                    self.process();
                }
                _ => {}
            }
        }

        Action::None
    }

    fn help(&self) -> Vec<&'static str> {
        vec![
            "Tab: input/mode",
            "Up/Down: select case",
            "Ctrl+C: copy output",
        ]
    }
}
