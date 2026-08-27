use crate::theme::Theme;
use crate::tools::{self, Action, Category, Tool};
use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
                (cat == Category::All || m.category == cat)
                    && (self.search.is_empty()
                        || matcher.fuzzy_match(m.name, &self.search).is_some()
                        || m.keywords.iter().any(|k| matcher.fuzzy_match(k, &self.search).is_some()))
            })
            .map(|(i, _)| i)
            .collect();

        if self.filtered.is_empty() {
            self.tool_list_state.select(None);
        } else if self.tool_list_state.selected().is_none()
            || self.tool_list_state.selected().unwrap() >= self.filtered.len()
        {
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
