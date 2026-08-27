use super::{Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent};
use ipnet::Ipv4Net;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::str::FromStr;
use tui_textarea::{Input, TextArea};

pub struct Ipv4Subnet<'a> {
    input: TextArea<'a>,
    network: String,
    broadcast: String,
    netmask: String,
    total_hosts: String,
    host_range: String,
    error: String,
}

impl<'a> Ipv4Subnet<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(Block::default().borders(Borders::ALL).title(" Input CIDR (e.g. 192.168.1.1/24) "));
        Self {
            input,
            network: String::new(),
            broadcast: String::new(),
            netmask: String::new(),
            total_hosts: String::new(),
            host_range: String::new(),
            error: String::new(),
        }
    }

    fn clear_fields(&mut self) {
        self.network.clear();
        self.broadcast.clear();
        self.netmask.clear();
        self.total_hosts.clear();
        self.host_range.clear();
        self.error.clear();
    }

    fn process(&mut self) {
        let text = self.input.lines().join("").trim().to_string();
        self.clear_fields();

        if text.is_empty() {
            return;
        }

        match Ipv4Net::from_str(&text) {
            Ok(net) => {
                self.network = net.network().to_string();
                self.broadcast = net.broadcast().to_string();
                self.netmask = net.netmask().to_string();

                let hosts: Vec<_> = net.hosts().collect();
                if hosts.is_empty() {
                    self.total_hosts = "0 (or 1 depending on context)".to_string();
                    self.host_range = "N/A".to_string();
                } else if hosts.len() == 1 {
                    self.total_hosts = "1".to_string();
                    self.host_range = hosts[0].to_string();
                } else {
                    self.total_hosts = hosts.len().to_string();
                    self.host_range = format!("{} - {}", hosts.first().unwrap(), hosts.last().unwrap());
                }
            }
            Err(e) => {
                self.error = format!("Invalid IPv4 CIDR: {}", e);
            }
        }
    }
}

impl<'a> Tool for Ipv4Subnet<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "ipv4-subnet",
            name: "IPv4 Subnet Calculator",
            category: Category::Network,
            description: "Calculate network, broadcast, netmask and host range from a CIDR.",
            keywords: &["ipv4", "subnet", "cidr", "network", "ip", "address", "netmask"],
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
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input CIDR (Esc to go back) ").border_style(border_style));
            self.input.set_cursor_line_style(Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED));
        } else {
            self.input.set_block(Block::default().borders(Borders::ALL).title(" Input CIDR "));
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
                "Network:     {}\nBroadcast:   {}\nNetmask:     {}\nTotal Hosts: {}\nHost Range:  {}",
                self.network, self.broadcast, self.netmask, self.total_hosts, self.host_range
            );
            let parts_p = Paragraph::new(parts_text)
                .block(Block::default().borders(Borders::ALL).title(" Subnet Details "));
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
