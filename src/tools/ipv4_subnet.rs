use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ipnet::Ipv4Net;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::{net::Ipv4Addr, str::FromStr};
use tui_textarea::{Input, TextArea};

#[derive(Debug, PartialEq)]
pub struct SubnetDetails {
    pub network: Ipv4Addr,
    pub broadcast: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub usable_hosts: u64,
    pub host_range: String,
}

pub fn calculate_subnet(value: &str) -> Result<SubnetDetails, String> {
    let net = Ipv4Net::from_str(value.trim()).map_err(|e| format!("Invalid IPv4 CIDR: {e}"))?;
    let network = net.network();
    let broadcast = net.broadcast();
    let prefix = net.prefix_len();
    let usable_hosts = match prefix {
        32 => 1,
        31 => 2,
        _ => (1_u64 << (32 - prefix)) - 2,
    };
    let host_range = match prefix {
        32 => network.to_string(),
        31 => format!("{network} - {broadcast}"),
        _ => format!(
            "{} - {}",
            Ipv4Addr::from(u32::from(network) + 1),
            Ipv4Addr::from(u32::from(broadcast) - 1)
        ),
    };
    Ok(SubnetDetails {
        network,
        broadcast,
        netmask: net.netmask(),
        usable_hosts,
        host_range,
    })
}

pub struct Ipv4Subnet<'a> {
    input: TextArea<'a>,
    output: String,
}
impl<'a> Ipv4Subnet<'a> {
    pub fn new() -> Self {
        Self {
            input: TextArea::default(),
            output: String::new(),
        }
    }
    fn process(&mut self) {
        let text = self.input.lines().join("");
        self.output = if text.trim().is_empty() {
            String::new()
        } else {
            match calculate_subnet(&text) {
                Ok(d) => format!(
                    "Network: {}\nBroadcast: {}\nNetmask: {}\nUsable hosts: {}\nHost range: {}",
                    d.network, d.broadcast, d.netmask, d.usable_hosts, d.host_range
                ),
                Err(e) => e,
            }
        };
    }
}
impl Tool for Ipv4Subnet<'_> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "ipv4-subnet",
            name: "IPv4 Subnet Calculator",
            category: Category::Network,
            description: "Calculate IPv4 network, broadcast, netmask, and usable host range.",
            keywords: &["ipv4", "subnet", "cidr", "netmask", "network"],
        }
    }
    fn render(&mut self, f: &mut Frame, a: Rect, focused: bool) {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(7)])
            .split(a);
        self.input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CIDR (for example 192.168.1.10/24) ")
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
                    .title(" Subnet details "),
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
        vec!["Input: IPv4 address/prefix", "Ctrl+C: copy details"]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculates_24() {
        let d = calculate_subnet("192.168.1.10/24").unwrap();
        assert_eq!(d.network, Ipv4Addr::new(192, 168, 1, 0));
        assert_eq!(d.usable_hosts, 254);
        assert_eq!(d.host_range, "192.168.1.1 - 192.168.1.254");
    }
    #[test]
    fn handles_31() {
        assert_eq!(calculate_subnet("10.0.0.0/31").unwrap().usable_hosts, 2);
    }
    #[test]
    fn rejects_invalid() {
        assert!(calculate_subnet("bad").is_err());
    }
}
