use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};

pub struct CronParser<'a> {
    input: TextArea<'a>,
    output: String,
}

const NAMES: [&str; 5] = ["Minute", "Hour", "Day of month", "Month", "Day of week"];
const RANGES: [(u8, u8); 5] = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 7)];

fn validate_field(field: &str, (min, max): (u8, u8)) -> bool {
    field.split(',').all(|part| {
        let core = part.split_once('/').map_or(part, |(a, b)| {
            if b.parse::<u8>().is_ok_and(|n| n > 0) {
                a
            } else {
                "!"
            }
        });
        if core == "*" {
            return true;
        }
        if let Some((a, b)) = core.split_once('-') {
            return a.parse::<u8>().is_ok_and(|v| v >= min && v <= max)
                && b.parse::<u8>().is_ok_and(|v| v >= min && v <= max);
        }
        core.parse::<u8>().is_ok_and(|v| v >= min && v <= max)
    })
}

fn expand_field(field: &str, (min, max): (u8, u8)) -> Vec<u8> {
    if field == "*" {
        return (min..=max).collect();
    }

    let (core, step) = match field.split_once('/') {
        Some((a, b)) => (a, b.parse::<u8>().unwrap_or(1).max(1)),
        None => (field, 1),
    };

    let values: Vec<u8> = if let Some((a, b)) = core.split_once('-') {
        let start = a.parse::<u8>().unwrap_or(min).max(min).min(max);
        let end = b.parse::<u8>().unwrap_or(max).max(min).min(max);
        (start..=end).collect()
    } else if core == "*" {
        (min..=max).collect()
    } else {
        core.split(',')
            .filter_map(|v| v.parse::<u8>().ok())
            .filter(|v| *v >= min && *v <= max)
            .collect()
    };

    if step > 1 {
        values.into_iter().step_by(step as usize).collect()
    } else {
        values
    }
}

fn matches_cron(
    dt: &DateTime<Utc>,
    minutes: &[u8],
    hours: &[u8],
    dom: &[u8],
    months: &[u8],
    dow: &[u8],
) -> bool {
    let month = dt.month() as u8;
    let day = dt.day() as u8;
    let weekday = dt.weekday().num_days_from_sunday() as u8;
    let hour = dt.hour() as u8;
    let minute = dt.minute() as u8;

    minutes.contains(&minute)
        && hours.contains(&hour)
        && dom.contains(&day)
        && months.contains(&month)
        && dow.contains(&weekday)
}

fn next_runs(value: &str, count: usize) -> Result<Vec<DateTime<Utc>>, String> {
    let fields: Vec<_> = value.split_whitespace().collect();
    if fields.len() != 5 {
        return Err("Expected five fields: minute hour day-of-month month day-of-week".into());
    }
    for (i, field) in fields.iter().enumerate() {
        if !validate_field(field, RANGES[i]) {
            return Err(format!("Invalid {} field: {field}", NAMES[i]));
        }
    }

    let minutes = expand_field(fields[0], RANGES[0]);
    let hours = expand_field(fields[1], RANGES[1]);
    let dom = expand_field(fields[2], RANGES[2]);
    let months = expand_field(fields[3], RANGES[3]);
    let dow = expand_field(fields[4], RANGES[4]);

    let mut runs = Vec::with_capacity(count);
    let mut candidate = Utc::now();
    // Start searching from the next minute boundary
    candidate = candidate
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    while runs.len() < count {
        candidate += Duration::minutes(1);
        if matches_cron(&candidate, &minutes, &hours, &dom, &months, &dow) {
            runs.push(candidate);
        }
        // Safety cap: don't search more than ~4 years ahead
        if runs.is_empty() && candidate > Utc::now() + Duration::days(366 * 4) {
            return Err("No matching run time found in the next four years".into());
        }
    }

    Ok(runs)
}

fn describe(v: &str) -> String {
    if v == "*" {
        "every value".into()
    } else if let Some(step) = v.strip_prefix("*/") {
        format!("every {step}")
    } else {
        v.into()
    }
}

pub fn parse_cron(value: &str) -> Result<String, String> {
    let fields: Vec<_> = value.split_whitespace().collect();
    if fields.is_empty() {
        return Ok(String::new());
    }
    if fields.len() != 5 {
        return Err("Expected five fields: minute hour day-of-month month day-of-week".into());
    }
    for (i, field) in fields.iter().enumerate() {
        if !validate_field(field, RANGES[i]) {
            return Err(format!("Invalid {} field: {field}", NAMES[i]));
        }
    }

    let mut lines: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(i, v)| format!("{}: {}", NAMES[i], describe(v)))
        .collect();

    match next_runs(value, 5) {
        Ok(runs) => {
            lines.push("".into());
            lines.push("Next runs (UTC):".into());
            for run in runs {
                lines.push(run.to_rfc3339());
            }
        }
        Err(e) => {
            lines.push(format!("\nCould not calculate next runs: {e}"));
        }
    }

    Ok(lines.join("\n"))
}

impl<'a> CronParser<'a> {
    pub fn new() -> Self {
        Self {
            input: TextArea::default(),
            output: String::new(),
        }
    }
    fn process(&mut self) {
        self.output = parse_cron(&self.input.lines().join(" ")).unwrap_or_else(|e| e)
    }
}

impl Tool for CronParser<'_> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "cron",
            name: "Cron Parser",
            category: Category::Development,
            description: "Validate, explain, and preview standard five-field cron expressions.",
            keywords: &["cron", "schedule", "timer", "job"],
        }
    }
    fn render(&mut self, f: &mut Frame, a: Rect, focused: bool) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(5)])
            .split(a);
        self.input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Cron expression ")
                .border_style(if focused {
                    crate::theme::Theme::default().border_active()
                } else {
                    crate::theme::Theme::default().border_inactive()
                }),
        );
        f.render_widget(&self.input, c[0]);
        f.render_widget(
            Paragraph::new(self.output.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Explanation "),
            ),
            c[1],
        );
    }
    fn handle_key(&mut self, k: KeyEvent) -> Action {
        if k.code == KeyCode::Esc {
            return Action::Back;
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
        vec![
            "Format: min hour dom month dow",
            "Supports *, lists, ranges, and steps",
            "Ctrl+C: copy explanation",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses() {
        let out = parse_cron("*/5 9-17 * * 1-5").unwrap();
        assert!(out.contains("every 5"));
        assert!(out.contains("Next runs"));
    }
    #[test]
    fn rejects_range() {
        assert!(parse_cron("61 * * * *").is_err());
    }
    #[test]
    fn expand_star() {
        assert_eq!(expand_field("*", (0, 3)).len(), 4);
    }
    #[test]
    fn expand_step() {
        let v = expand_field("*/2", (0, 5));
        assert_eq!(v, vec![0, 2, 4]);
    }
    #[test]
    fn expand_range() {
        assert_eq!(expand_field("1-3", (0, 5)), vec![1, 2, 3]);
    }
    #[test]
    fn next_run_is_in_the_future() {
        let runs = next_runs("0 0 * * *", 1).unwrap();
        assert!(runs[0] > Utc::now());
    }
}
