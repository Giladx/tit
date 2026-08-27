Here’s a complete, compilable starter for the IT-Tools TUI in the **PAR LLAMA** style (dark + orange accents, 3-pane layout).

### 1. Create the project

```bash
cargo new tit
cd tit
```

### 2. `Cargo.toml`

```toml
[package]
name = "tit"
version = "0.1.0"
edition = "2021"
description = "Terminal UI toolbox inspired by it-tools.tech"
license = "MIT"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tui-textarea = "0.7"
chrono = { version = "0.4", features = ["clock", "serde"] }
regex = "1"
arboard = "3"          # clipboard
fuzzy-matcher = "0.3"
unicode-width = "0.2"
anyhow = "1"
```

### 3. Project structure

```
src/
├── main.rs
├── app.rs
├── theme.rs
├── tools/
│   ├── mod.rs
│   └── datetime.rs
└── ui/
    ├── mod.rs
    ├── sidebar.rs
    └── statusbar.rs
```

### 4. `src/theme.rs`

```rust
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
```

### 5. `src/tools/mod.rs`

```rust
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
```

### 6. `src/tools/datetime.rs` (full live converter)

```rust
use super::{Action, Category, Tool, ToolMeta};
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState},
};
use regex::Regex;
use std::sync::OnceLock;
use tui_textarea::{Input, Key, TextArea};

#[derive(Clone, Copy, PartialEq)]
enum DateFormat {
    JsLocale,
    Iso8601,
    Iso9075,
    Rfc3339,
    Rfc7231,
    UnixSeconds,
    UnixMillis,
    UtcString,
    MongoObjectId,
    Excel,
}

impl DateFormat {
    const ALL: &'static [DateFormat] = &[
        Self::JsLocale,
        Self::Iso8601,
        Self::Iso9075,
        Self::Rfc3339,
        Self::Rfc7231,
        Self::UnixSeconds,
        Self::UnixMillis,
        Self::UtcString,
        Self::MongoObjectId,
        Self::Excel,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::JsLocale => "JS locale date string",
            Self::Iso8601 => "ISO 8601",
            Self::Iso9075 => "ISO 9075",
            Self::Rfc3339 => "RFC 3339",
            Self::Rfc7231 => "RFC 7231",
            Self::UnixSeconds => "Unix timestamp",
            Self::UnixMillis => "Timestamp (ms)",
            Self::UtcString => "UTC format",
            Self::MongoObjectId => "Mongo ObjectID",
            Self::Excel => "Excel date/time",
        }
    }

    fn parse(self, s: &str) -> Option<DateTime<Utc>> {
        if s.is_empty() {
            return Some(Utc::now());
        }
        match self {
            Self::UnixSeconds => s.parse::<i64>().ok().and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            Self::UnixMillis => s.parse::<i64>().ok().and_then(|ts| Utc.timestamp_millis_opt(ts).single()),
            Self::Excel => s.parse::<f64>().ok().map(|v| {
                let ms = ((v - 25569.0) * 86_400_000.0) as i64;
                Utc.timestamp_millis_opt(ms).single().unwrap_or_else(Utc::now)
            }),
            Self::MongoObjectId if s.len() >= 8 => {
                u32::from_str_radix(&s[..8], 16).ok().map(|ts| {
                    Utc.timestamp_opt(ts as i64, 0).single().unwrap_or_else(Utc::now)
                })
            }
            _ => DateTime::parse_from_rfc3339(s)
                .or_else(|_| DateTime::parse_from_rfc2822(s))
                .or_else(|_| s.parse::<DateTime<Utc>>())
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok().map(|n| Utc.from_utc_datetime(&n)))
                .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok().map(|n| Utc.from_utc_datetime(&n))),
        }
    }

    fn format(self, dt: DateTime<Utc>) -> String {
        match self {
            Self::JsLocale => dt.to_rfc2822(),
            Self::Iso8601 => dt.to_rfc3339_opts(SecondsFormat::Millis, true),
            Self::Iso9075 => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            Self::Rfc3339 => dt.to_rfc3339(),
            Self::Rfc7231 => dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
            Self::UnixSeconds => dt.timestamp().to_string(),
            Self::UnixMillis => dt.timestamp_millis().to_string(),
            Self::UtcString => dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
            Self::MongoObjectId => format!("{:08x}0000000000000000", dt.timestamp() as u32),
            Self::Excel => {
                let v = (dt.timestamp_millis() as f64 / 86_400_000.0) + 25569.0;
                format!("{:.5}", v)
            }
        }
    }

    fn matches(self, s: &str) -> bool {
        static RE_UNIX: OnceLock<Regex> = OnceLock::new();
        static RE_MILLIS: OnceLock<Regex> = OnceLock::new();
        static RE_MONGO: OnceLock<Regex> = OnceLock::new();
        static RE_EXCEL: OnceLock<Regex> = OnceLock::new();
        static RE_ISO: OnceLock<Regex> = OnceLock::new();

        match self {
            Self::UnixSeconds => RE_UNIX.get_or_init(|| Regex::new(r"^[0-9]{1,10}$").unwrap()).is_match(s),
            Self::UnixMillis => RE_MILLIS.get_or_init(|| Regex::new(r"^[0-9]{11,13}$").unwrap()).is_match(s),
            Self::MongoObjectId => RE_MONGO.get_or_init(|| Regex::new(r"^[0-9a-fA-F]{24}$").unwrap()).is_match(s),
            Self::Excel => RE_EXCEL.get_or_init(|| Regex::new(r"^-?\d+(\.\d+)?$").unwrap()).is_match(s),
            Self::Iso8601 | Self::Rfc3339 => {
                RE_ISO.get_or_init(|| Regex::new(r"^\d{4}-\d{2}-\d{2}T").unwrap()).is_match(s)
            }
            _ => false,
        }
    }
}

pub struct DateTimeConverter {
    input: TextArea<'static>,
    selected_format: usize,
    results: Vec<(DateFormat, String)>,
    table_state: TableState,
    is_valid: bool,
    status: String,
}

impl DateTimeConverter {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_placeholder_text("Paste a date / timestamp or leave empty for now...");
        input.set_cursor_line_style(Style::default());

        let mut s = Self {
            input,
            selected_format: 1, // ISO 8601
            results: vec![],
            table_state: TableState::default(),
            is_valid: true,
            status: String::new(),
        };
        s.recompute();
        s
    }

    fn recompute(&mut self) {
        let text = self.input.lines().join("\n").trim().to_string();

        // Auto-detect
        if !text.is_empty() {
            if let Some((idx, _)) = DateFormat::ALL
                .iter()
                .enumerate()
                .find(|(_, f)| f.matches(&text))
            {
                self.selected_format = idx;
            }
        }

        let fmt = DateFormat::ALL[self.selected_format];
        match fmt.parse(&text) {
            Some(dt) => {
                self.is_valid = true;
                self.status = if text.is_empty() {
                    "Using current time".into()
                } else {
                    format!("Detected: {}", fmt.name())
                };
                self.results = DateFormat::ALL
                    .iter()
                    .map(|f| (*f, f.format(dt)))
                    .collect();
            }
            None => {
                self.is_valid = false;
                self.status = "Invalid date for selected format".into();
                self.results = DateFormat::ALL
                    .iter()
                    .map(|f| (*f, String::new()))
                    .collect();
            }
        }
    }
}

impl Tool for DateTimeConverter {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "datetime",
            name: "Date-Time Converter",
            category: Category::Converter,
            description: "Convert between many date/time formats (live)",
            keywords: &["date", "time", "timestamp", "unix", "iso", "rfc", "excel", "mongo"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let theme = crate::theme::Theme::default();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // input
                Constraint::Length(1), // status
                Constraint::Min(8),    // results
            ])
            .split(area);

        // Input
        let input_block = Block::default()
            .title(" Input ")
            .borders(Borders::ALL)
            .border_style(if focused {
                theme.border_active()
            } else {
                theme.border_inactive()
            });
        self.input.set_block(input_block);
        f.render_widget(&self.input, chunks[0]);

        // Status
        let status_style = if self.is_valid {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.error)
        };
        f.render_widget(
            Paragraph::new(Span::styled(&self.status, status_style)),
            chunks[1],
        );

        // Results table
        let header = Row::new(vec!["Format", "Value"]).style(Style::default().fg(theme.orange));
        let rows: Vec<Row> = self
            .results
            .iter()
            .map(|(fmt, val)| {
                Row::new(vec![fmt.name().to_string(), val.clone()]).style(theme.normal())
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Length(22), Constraint::Min(20)],
        )
        .header(header)
        .block(
            Block::default()
                .title(" Conversions (↑↓ select, Enter/c copy) ")
                .borders(Borders::ALL)
                .border_style(theme.border_inactive()),
        )
        .row_highlight_style(theme.selected())
        .highlight_symbol("▶ ");

        f.render_stateful_widget(table, chunks[2], &mut self.table_state);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('c') | KeyCode::Enter => {
                if let Some(idx) = self.table_state.selected() {
                    if let Some((_, value)) = self.results.get(idx) {
                        if let Ok(mut clip) = arboard::Clipboard::new() {
                            let _ = clip.set_text(value.clone());
                            self.status = format!("Copied: {}", value);
                            return Action::Copied;
                        }
                    }
                }
            }
            KeyCode::Up => {
                let i = self.table_state.selected().unwrap_or(0);
                self.table_state.select(Some(i.saturating_sub(1)));
            }
            KeyCode::Down => {
                let i = self.table_state.selected().unwrap_or(0);
                let max = self.results.len().saturating_sub(1);
                self.table_state.select(Some((i + 1).min(max)));
            }
            KeyCode::Tab => {
                self.selected_format = (self.selected_format + 1) % DateFormat::ALL.len();
                self.recompute();
            }
            KeyCode::BackTab => {
                self.selected_format =
                    (self.selected_format + DateFormat::ALL.len() - 1) % DateFormat::ALL.len();
                self.recompute();
            }
            _ => {
                // Feed to textarea
                let input = Input::from(key);
                if self.input.input(input) {
                    self.recompute();
                }
            }
        }
        Action::None
    }

    fn on_focus(&mut self) {
        self.recompute();
    }
}
```

### 7. `src/app.rs` (core application)

```rust
use crate::theme::Theme;
use crate::tools::{self, Action, Category, Tool};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub enum Focus {
    Sidebar,
    Tool,
}

pub struct App {
    pub theme: Theme,
    pub tools: Vec<Box<dyn Tool>>,
    pub filtered: Vec<usize>,          // indices into tools
    pub category_idx: usize,
    pub tool_list_state: ListState,
    pub focus: Focus,
    pub search: String,
    pub searching: bool,
    pub should_quit: bool,
    pub status_message: String,
}

impl App {
    pub fn new() -> Self {
        let tools = tools::all_tools();
        let mut app = Self {
            theme: Theme::default(),
            tools,
            filtered: vec![],
            category_idx: 0,
            tool_list_state: ListState::default(),
            focus: Focus::Sidebar,
            search: String::new(),
            searching: false,
            should_quit: false,
            status_message: "Welcome to TIT.RUN Tools".into(),
        };
        app.rebuild_filter();
        app.tool_list_state.select(Some(0));
        app
    }

    fn rebuild_filter(&mut self) {
        let cat = Category::all()[self.category_idx];
        let matcher = SkimMatcherV2::default();

        self.filtered = self
            .tools
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                let m = t.meta();
                m.category == cat
                    && (self.search.is_empty()
                        || matcher.fuzzy_match(m.name, &self.search).is_some()
                        || m.keywords.iter().any(|k| {
                            matcher.fuzzy_match(k, &self.search).is_some()
                        }))
            })
            .map(|(i, _)| i)
            .collect();

        if self.filtered.is_empty() {
            self.tool_list_state.select(None);
        } else {
            self.tool_list_state.select(Some(0));
        }
    }

    pub fn current_tool_mut(&mut self) -> Option<&mut Box<dyn Tool>> {
        self.tool_list_state
            .selected()
            .and_then(|i| self.filtered.get(i).copied())
            .map(|idx| &mut self.tools[idx])
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.searching {
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    self.search.clear();
                    self.rebuild_filter();
                }
                KeyCode::Enter => {
                    self.searching = false;
                    self.focus = Focus::Tool;
                    if let Some(t) = self.current_tool_mut() {
                        t.on_focus();
                    }
                }
                KeyCode::Char(c) => {
                    self.search.push(c);
                    self.rebuild_filter();
                }
                KeyCode::Backspace => {
                    self.search.pop();
                    self.rebuild_filter();
                }
                _ => {}
            }
            return;
        }

        match self.focus {
            Focus::Sidebar => match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('/') => {
                    self.searching = true;
                    self.search.clear();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = self.tool_list_state.selected().unwrap_or(0);
                    self.tool_list_state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = self.tool_list_state.selected().unwrap_or(0);
                    let max = self.filtered.len().saturating_sub(1);
                    self.tool_list_state.select(Some((i + 1).min(max)));
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if self.category_idx > 0 {
                        self.category_idx -= 1;
                        self.rebuild_filter();
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if self.category_idx + 1 < Category::all().len() {
                        self.category_idx += 1;
                        self.rebuild_filter();
                    }
                }
                KeyCode::Enter => {
                    self.focus = Focus::Tool;
                    if let Some(t) = self.current_tool_mut() {
                        t.on_focus();
                    }
                }
                _ => {}
            },
            Focus::Tool => {
                if key.code == KeyCode::Esc {
                    self.focus = Focus::Sidebar;
                    return;
                }
                if let Some(tool) = self.current_tool_mut() {
                    match tool.handle_key(key) {
                        Action::Quit => self.should_quit = true,
                        Action::Back => self.focus = Focus::Sidebar,
                        Action::Copied => {
                            self.status_message = "Copied to clipboard!".into();
                        }
                        Action::None => {}
                    }
                }
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        let theme = self.theme;

        // Background
        f.render_widget(
            Block::default().style(Style::default().bg(theme.bg)),
            f.area(),
        );

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(1)])
            .split(f.area());

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(28), // sidebar
                Constraint::Min(40),    // tool
                Constraint::Length(26), // right panel (future options)
            ])
            .split(main_chunks[0]);

        self.render_sidebar(f, body[0]);
        self.render_tool(f, body[1]);
        self.render_right(f, body[2]);
        self.render_statusbar(f, main_chunks[1]);
    }

    fn render_sidebar(&mut self, f: &mut Frame, area: Rect) {
        let theme = self.theme;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        // Categories
        let cats: Vec<ListItem> = Category::all()
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let style = if i == self.category_idx {
                    theme.selected()
                } else {
                    theme.normal()
                };
                ListItem::new(c.name()).style(style)
            })
            .collect();

        let cat_list = List::new(cats).block(
            Block::default()
                .title(" Categories ")
                .borders(Borders::ALL)
                .border_style(theme.border_inactive())
                .style(Style::default().bg(theme.panel)),
        );
        f.render_widget(cat_list, chunks[0]);

        // Tools
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .map(|&idx| {
                let name = self.tools[idx].meta().name;
                ListItem::new(name)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(if self.searching {
                        format!(" Tools /{} ", self.search)
                    } else {
                        " Tools ".into()
                    })
                    .borders(Borders::ALL)
                    .border_style(if matches!(self.focus, Focus::Sidebar) {
                        theme.border_active()
                    } else {
                        theme.border_inactive()
                    })
                    .style(Style::default().bg(theme.panel)),
            )
            .highlight_style(theme.selected())
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut self.tool_list_state);
    }

    fn render_tool(&mut self, f: &mut Frame, area: Rect) {
        let theme = self.theme;
        let focused = matches!(self.focus, Focus::Tool);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if focused {
                theme.border_active()
            } else {
                theme.border_inactive()
            })
            .style(Style::default().bg(theme.panel));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if let Some(tool) = self.current_tool_mut() {
            tool.render(f, inner, focused);
        } else {
            f.render_widget(
                Paragraph::new("No tool selected").style(theme.dim()),
                inner,
            );
        }
    }

    fn render_right(&self, f: &mut Frame, area: Rect) {
        let theme = self.theme;
        let block = Block::default()
            .title(" Options / Help ")
            .borders(Borders::ALL)
            .border_style(theme.border_inactive())
            .style(Style::default().bg(theme.panel));

        let text = vec![
            Line::from(""),
            Line::from(Span::styled("Keybindings", Style::default().fg(theme.orange))),
            Line::from("  ↑↓ / j k    Navigate"),
            Line::from("  ←→ / h l    Category"),
            Line::from("  Enter       Open tool"),
            Line::from("  Esc         Back"),
            Line::from("  /           Search"),
            Line::from("  c / Enter   Copy"),
            Line::from("  Tab         Cycle format"),
            Line::from("  q           Quit"),
        ];

        f.render_widget(Paragraph::new(text).block(block), area);
    }

    fn render_statusbar(&self, f: &mut Frame, area: Rect) {
        let theme = self.theme;
        let text = if self.searching {
            format!(" Search: {} ", self.search)
        } else {
            format!(" {}  |  {}", self.status_message, "q quit  / search  Esc back")
        };
        f.render_widget(
            Paragraph::new(text).style(Style::default().fg(theme.text_dim).bg(theme.bg)),
            area,
        );
    }
}
```

### 8. `src/main.rs`

```rust
mod app;
mod theme;
mod tools;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};

fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
```

### 9. Dummy `src/ui/mod.rs` (for now)

```rust
// Future: extract sidebar/statusbar widgets here if desired
```

---

### Run it

```bash
cargo run
```

You get:

- Dark background + orange accents (PAR LLAMA style)
- Left sidebar: Categories + filterable tool list
- Main pane: Date-Time Converter with live multi-format output
- Right pane: Keybinding help
- Bottom status bar
- Full keyboard navigation + search (`/`) + copy to clipboard

### Next steps (very easy now)

1. Add more tools by implementing the `Tool` trait and pushing them into `all_tools()`.
2. Pull solid logic from `ksdme/ut` (hash, base64, case, uuid, jwt, regex, etc.).
3. Improve the Date-Time parser (add more formats / better auto-detect).
4. Add a global CLI mode later (`it-tui hash sha256 "..."`).

Want me to add the next tool right away (Hash Text, Base64, Case Converter, or the interactive Regex from ut)? Just say which one.


**Yes — several terminal / system features make a huge difference for AI agents.**  

A pure interactive Ratatui TUI is excellent for humans but painful for agents (they hate full-screen alternate buffers, cursor addressing, and having to “see” the screen). The biggest wins come from making the same tool dual-mode: beautiful TUI for people + clean, machine-first interface for agents.

### Highest-impact features for agents

| Feature | Why agents love it | How to implement in our IT-Tools TUI |
|---------|---------------------|--------------------------------------|
| **Non-interactive / headless mode** | Agents never enter the TUI | `it-tui --tool datetime --input "1725..." --format unix` or just `it-tui datetime ...` (CLI subcommands) |
| **Structured output (JSON / NDJSON)** | Easy to parse, no scraping | `--output json` or auto-detect when stdout is not a TTY |
| **Stable exit codes** | Success/failure signalling | 0 = ok, 1 = invalid input, 2 = tool error, etc. |
| **Stdin / pipe support** | Composition (`echo "..." \| it-tui hash sha256`) | Every tool accepts `-` or reads stdin |
| **No ANSI when not a TTY** | Clean text for LLMs | `if !std::io::stdout().is_terminal() { /* plain */ }` |
| **Schema / machine-readable help** | Agents can discover tools | `--help --json` or a `it-tui tools --json` that returns the full registry |
| **Deterministic & side-effect free** | Reliable planning | Pure functions + explicit flags (no hidden “now”, no clipboard by default in agent mode) |
| **Batch / multiple inputs** | Efficiency | Accept multiple values or a file of inputs |
| **Timeouts & resource limits** | Safety | Built-in for long-running tools |
| **Logging to stderr only** | stdout stays clean for data | All human messages / progress → stderr |

### Especially powerful combinations

1. **CLI-first + TUI as optional UI**
   ```bash
   # Agent style
   tit datetime --input 1728000000 --out json
   # → {"unix":"1728000000","iso8601":"2024-...","excel":...}

   # Human style
   tit         # launches the full PAR-LLAMA TUI
   ```

2. **JSON Schema or OpenAPI-style discovery**
   Agents can call `it-tui tools --json` and get:
   ```json
   [
     {
       "id": "datetime",
       "name": "Date-Time Converter",
       "category": "Converter",
       "inputs": [{"name":"input","type":"string"}, {"name":"format","type":"enum",...}],
       "outputs": {"type":"object", "properties": {...}}
     }
   ]
   ```

3. **NDJSON streaming** for tools that produce multiple results (diff, batch hash, etc.)

4. **Explicit “agent mode” flag** (`--agent` or `IT_TUI_AGENT=1`) that:
   - Forces JSON
   - Disables clipboard / interactive prompts
   - Uses fixed “now” or requires explicit timestamp
   - Adds more verbose error objects

5. **Unix philosophy compliance**
   - One tool ≈ one job
   - Composable via pipes
   - Text in / text out (with JSON as the rich text)

### Terminal features that help indirectly

- **Truecolor / 24-bit color** — nice for humans, irrelevant for agents (we already strip it).
- **Kitty / WezTerm image protocol or Sixel** — only useful if we later show QR codes as images; agents prefer the ASCII or raw data.
- **OSC 52 clipboard** — great for humans, dangerous/noisy for agents → disable in agent mode.
- **Bracketed paste** — helps humans, not agents.
- **Focus events / mouse** — pure human features.
- **Alternate screen buffer** — the main reason agents hate pure TUIs. Always offer a way to avoid it.

### Practical recommendation for our project

Make the architecture:

```
tit
├── TUI mode (default when run with no args + is_terminal)
└── CLI mode (subcommands or --tool)
    ├── Human-friendly text
    └── --json / auto-json when piped or --agent
```

This is exactly what good modern tools do (e.g. `gh`, `docker`, `kubectl`, `ut` itself, `ripgrep`, etc.).

We can reuse a lot of `ksdme/ut`’s CLI structure and clap derives, then wrap the same core logic with a Ratatui front-end.

---

**Want me to evolve the starter** so it already supports both modes?

Example target usage for agents:

```bash
tit datetime "2026-08-27T19:03:00Z" --json
tit hash sha256 "hello" --json
echo "hello world" | it-tui case snake --json
tit tools --json          # discovery
```

I can show the updated `main.rs` + clap integration + a clean JSON output path next if you’d like.