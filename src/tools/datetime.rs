use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use chrono::{DateTime, Local, SecondsFormat, TimeZone, Utc};
use chrono_tz::Tz;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
    Frame,
};
use regex::Regex;
use std::sync::OnceLock;
use tui_textarea::{Input, TextArea};

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

    fn format_with_tz(self, dt: DateTime<Utc>, tz: Tz) -> String {
        let local = dt.with_timezone(&tz);
        match self {
            Self::JsLocale => local.to_rfc2822(),
            Self::Iso8601 => dt.to_rfc3339_opts(SecondsFormat::Millis, true),
            Self::Iso9075 => local.format("%Y-%m-%d %H:%M:%S").to_string(),
            Self::Rfc3339 => dt.to_rfc3339(),
            Self::Rfc7231 => local.format("%a, %d %b %Y %H:%M:%S %Z").to_string(),
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

    fn parse(self, s: &str) -> Option<DateTime<Utc>> {
        if s.is_empty() {
            return Some(Utc::now());
        }
        match self {
            Self::UnixSeconds => s
                .parse::<i64>()
                .ok()
                .and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
            Self::UnixMillis => s
                .parse::<i64>()
                .ok()
                .and_then(|ts| Utc.timestamp_millis_opt(ts).single()),
            Self::Excel => s.parse::<f64>().ok().and_then(|v| {
                let ms = ((v - 25569.0) * 86_400_000.0) as i64;
                Utc.timestamp_millis_opt(ms).single()
            }),
            Self::MongoObjectId if s.len() >= 8 => u32::from_str_radix(&s[..8], 16)
                .ok()
                .and_then(|ts| Utc.timestamp_opt(ts as i64, 0).single()),
            _ => DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
                .or_else(|| {
                    DateTime::parse_from_rfc2822(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .or_else(|| s.parse::<DateTime<Utc>>().ok())
                .or_else(|| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|n| Utc.from_utc_datetime(&n))
                })
                .or_else(|| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                        .ok()
                        .map(|n| Utc.from_utc_datetime(&n))
                }),
        }
    }

    fn matches(self, s: &str) -> bool {
        static RE_UNIX: OnceLock<Regex> = OnceLock::new();
        static RE_MILLIS: OnceLock<Regex> = OnceLock::new();
        static RE_MONGO: OnceLock<Regex> = OnceLock::new();
        static RE_EXCEL: OnceLock<Regex> = OnceLock::new();
        static RE_ISO: OnceLock<Regex> = OnceLock::new();

        match self {
            Self::UnixSeconds => RE_UNIX
                .get_or_init(|| Regex::new(r"^-?[0-9]{1,10}$").unwrap())
                .is_match(s),
            Self::UnixMillis => RE_MILLIS
                .get_or_init(|| Regex::new(r"^-?[0-9]{11,13}$").unwrap())
                .is_match(s),
            Self::MongoObjectId => RE_MONGO
                .get_or_init(|| Regex::new(r"^[0-9a-fA-F]{24}$").unwrap())
                .is_match(s),
            Self::Excel => RE_EXCEL
                .get_or_init(|| Regex::new(r"^-?\d+(\.\d+)?$").unwrap())
                .is_match(s),
            Self::Iso8601 | Self::Rfc3339 => RE_ISO
                .get_or_init(|| Regex::new(r"^\d{4}-\d{2}-\d{2}T").unwrap())
                .is_match(s),
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
pub fn convert_datetime(
    value: &str,
    tz_name: Option<&str>,
) -> Result<Vec<(&'static str, String)>, String> {
    let text = value.trim();

    let tz: Tz = tz_name
        .unwrap_or("UTC")
        .parse()
        .map_err(|e| format!("Invalid timezone: {e}"))?;

    // Auto-detect input format if non-empty
    let mut selected_format = DateFormat::Iso8601;
    if !text.is_empty() {
        if let Some((idx, _)) = DateFormat::ALL
            .iter()
            .enumerate()
            .find(|(_, f)| f.matches(text))
        {
            selected_format = DateFormat::ALL[idx];
        }
    }

    match selected_format.parse(text) {
        Some(dt) => Ok(DateFormat::ALL
            .iter()
            .map(|f| (f.name(), f.format_with_tz(dt, tz)))
            .collect()),
        None => Err(format!(
            "Invalid date for format: {}",
            selected_format.name()
        )),
    }
}

fn current_offset_label() -> String {
    let local_offset = Local::now().format("%:z").to_string();
    format!("Local ({local_offset})")
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
                let tz = current_offset_label().parse::<Tz>().unwrap_or(Tz::UTC);
                self.results = DateFormat::ALL
                    .iter()
                    .map(|f| (*f, f.format_with_tz(dt, tz)))
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
            description: "Convert between date/time formats with optional timezone.",
            keywords: &[
                "date",
                "time",
                "timestamp",
                "unix",
                "iso",
                "rfc",
                "excel",
                "mongo",
            ],
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

        let table = Table::new(rows, [Constraint::Length(22), Constraint::Min(20)])
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
                        return copy_to_clipboard(value.clone());
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

    fn help(&self) -> Vec<&'static str> {
        vec![
            "Tab/Shift+Tab: input format",
            "Up/Down: result",
            "Enter/C: copy result",
            "CLI: --timezone America/New_York",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unix_epoch() {
        assert_eq!(DateFormat::UnixSeconds.parse("0").unwrap().timestamp(), 0);
    }
    #[test]
    fn detects_negative_unix() {
        assert!(DateFormat::UnixSeconds.matches("-1"));
    }
    #[test]
    fn rejects_bad_mongo() {
        assert!(DateFormat::MongoObjectId
            .parse("zzzzzzzz0000000000000000")
            .is_none());
    }
    #[test]
    fn excel_epoch() {
        assert_eq!(DateFormat::Excel.parse("25569").unwrap().timestamp(), 0);
    }
}
