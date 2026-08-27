use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use tui_textarea::{Input, TextArea};
use url::Url;

pub struct UrlParser<'a> {
    input: TextArea<'a>,
    scheme: String,
    host: String,
    port: String,
    path: String,
    query: String,
    fragment: String,
    error: String,
}

impl<'a> UrlParser<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input URL (Type here) "));
        Self {
            input,
            scheme: String::new(),
            host: String::new(),
            port: String::new(),
            path: String::new(),
            query: String::new(),
            fragment: String::new(),
            error: String::new(),
        }
    }

    fn clear_fields(&mut self) {
        self.scheme.clear();
        self.host.clear();
        self.port.clear();
        self.path.clear();
        self.query.clear();
        self.fragment.clear();
        self.error.clear();
    }

    fn process(&mut self) {
        let text = self.input.lines().join("").trim().to_string();
        self.clear_fields();

        if text.is_empty() {
            return;
        }

        match Url::parse(&text) {
            Ok(url) => {
                self.scheme = url.scheme().to_string();
                if let Some(h) = url.host_str() { self.host = h.to_string(); }
                if let Some(p) = url.port() { self.port = p.to_string(); }
                self.path = url.path().to_string();
                if let Some(q) = url.query() { self.query = q.to_string(); }
                if let Some(f) = url.fragment() { self.fragment = f.to_string(); }
            }
            Err(e) => {
                self.error = format!("Invalid URL: {}", e);
            }
        }
    }
}

impl<'a> Tool for UrlParser<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "url-parser",
            name: "URL Parser",
            category: Category::Network,
            description: "Parse a URL into its constituent parts.",
            keywords: &["url", "parser", "scheme", "host", "path", "query", "network"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Input
                Constraint::Min(10),   // Parts
            ].as_ref())
            .split(area);

        let border_style = if focused { Style::default().fg(Color::Yellow) } else { Style::default() };

        if focused {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input URL (Esc to go back) ").border_style(border_style));
            self.input.set_cursor_line_style(Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED));
        } else {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input URL "));
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[0]);

        if !self.error.is_empty() {
            let error_p = Paragraph::new(self.error.as_str())
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title(" Parse Error "));
            f.render_widget(error_p, chunks[1]);
        } else {
            let parts_text = format!(
                "Scheme:   {}\nHost:     {}\nPort:     {}\nPath:     {}\nQuery:    {}\nFragment: {}",
                self.scheme, self.host, self.port, self.path, self.query, self.fragment
            );
            let parts_p = Paragraph::new(parts_text)
                .block(Block::default().borders(Borders::ALL).title(" URL Parts "));
            f.render_widget(parts_p, chunks[1]);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }
}
