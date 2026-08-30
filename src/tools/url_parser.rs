use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
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
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input URL (Type here) "),
        );
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
        match parse_url(&text) {
            Ok(parts) => {
                self.scheme = parts.scheme;
                self.host = parts.host;
                self.port = parts.port;
                self.path = parts.path;
                self.query = parts.query;
                self.fragment = parts.fragment;
                self.error.clear();
            }
            Err(e) => {
                self.clear_fields();
                self.error = e;
            }
        }
    }
}

pub struct UrlParts {
    pub scheme: String,
    pub host: String,
    pub port: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

pub fn parse_url(text: &str) -> Result<UrlParts, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("URL is empty".into());
    }

    let url = Url::parse(text).map_err(|e| format!("Invalid URL: {e}"))?;
    Ok(UrlParts {
        scheme: url.scheme().to_string(),
        host: url.host_str().unwrap_or("").to_string(),
        port: url.port().map(|p| p.to_string()).unwrap_or_default(),
        path: url.path().to_string(),
        query: url.query().unwrap_or("").to_string(),
        fragment: url.fragment().unwrap_or("").to_string(),
    })
}

impl<'a> Tool for UrlParser<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "url-parser",
            name: "URL Parser",
            category: Category::Network,
            description: "Parse a URL into its constituent parts.",
            keywords: &[
                "url", "parser", "scheme", "host", "path", "query", "network",
            ],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(4), // Input
                    Constraint::Min(10),   // Parts
                ]
                .as_ref(),
            )
            .split(area);

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        if focused {
            self.input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input URL (Esc to go back) ")
                    .border_style(border_style),
            );
            self.input.set_cursor_line_style(
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED),
            );
        } else {
            self.input
                .set_block(Block::default().borders(Borders::ALL).title(" Input URL "));
            self.input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[0]);

        if !self.error.is_empty() {
            let error_p = Paragraph::new(self.error.as_str())
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Parse Error "),
                );
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

        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(format!(
                "Scheme: {}\nHost: {}\nPort: {}\nPath: {}\nQuery: {}\nFragment: {}",
                self.scheme, self.host, self.port, self.path, self.query, self.fragment
            ));
        }

        if self.input.input(Input::from(key)) {
            self.process();
        }

        Action::None
    }

    fn help(&self) -> Vec<&'static str> {
        vec!["Type a complete URL", "Ctrl+C: copy parsed parts"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_url() {
        let parts = parse_url("https://example.com/path?q=1#top").unwrap();
        assert_eq!(parts.scheme, "https");
        assert_eq!(parts.host, "example.com");
        assert_eq!(parts.path, "/path");
        assert_eq!(parts.query, "q=1");
        assert_eq!(parts.fragment, "top");
    }

    #[test]
    fn parses_url_with_port() {
        let parts = parse_url("http://localhost:8080/api").unwrap();
        assert_eq!(parts.host, "localhost");
        assert_eq!(parts.port, "8080");
        assert_eq!(parts.path, "/api");
    }

    #[test]
    fn rejects_invalid_url() {
        assert!(parse_url("not a url").is_err());
    }

    #[test]
    fn rejects_empty_url() {
        assert!(parse_url("").is_err());
    }
}
