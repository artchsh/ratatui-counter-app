use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use sysinfo::Networks;

pub struct NetworkWidget<'a> {
    networks: &'a Networks,
}

impl<'a> NetworkWidget<'a> {
    pub fn new(networks: &'a Networks) -> Self {
        Self { networks }
    }

    fn format_speed(bytes_per_sec: u64) -> String {
        if bytes_per_sec >= 1024 * 1024 * 1024 {
            format!("{:.1}G", bytes_per_sec as f64 / 1024.0 / 1024.0 / 1024.0)
        } else if bytes_per_sec >= 1024 * 1024 {
            format!("{:.1}M", bytes_per_sec as f64 / 1024.0 / 1024.0)
        } else if bytes_per_sec >= 1024 {
            format!("{:.1}K", bytes_per_sec as f64 / 1024.0)
        } else {
            format!("{}", bytes_per_sec)
        }
    }

    fn get_interfaces(networks: &Networks) -> Vec<(&String, &sysinfo::NetworkData)> {
        networks
            .iter()
            .filter(|(name, data)| {
                if name.starts_with("lo") || name.as_str() == "lo0" {
                    return false;
                }
                if name.starts_with("docker") || name.starts_with("br-") || name.starts_with("veth") {
                    return false;
                }
                data.ip_networks().iter().any(|ip| ip.addr.is_ipv4())
            })
            .collect()
    }
}

impl Widget for NetworkWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Network ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        let interfaces = Self::get_interfaces(self.networks);

        if interfaces.is_empty() {
            Paragraph::new(Line::from(" No active interfaces "))
                .style(Color::DarkGray)
                .render(inner, buf);
            return;
        }

        let chunks = Layout::vertical(
            std::iter::repeat(Constraint::Length(1)).take(interfaces.len())
        ).split(inner);

        for (i, (name, data)) in interfaces.iter().enumerate() {
            let ipv4: Vec<String> = data
                .ip_networks()
                .iter()
                .filter(|ip| ip.addr.is_ipv4())
                .map(|ip| ip.addr.to_string())
                .collect();

            let ip_str = ipv4.first().cloned().unwrap_or_default();
            let dl = Self::format_speed(data.received());
            let ul = Self::format_speed(data.transmitted());

            let is_tailscale = name.starts_with("utun") || name.starts_with("tailscale");
            let is_wifi = name.starts_with("en") && !name.contains("bridge");

            let prefix = if is_tailscale {
                "TS"
            } else if is_wifi {
                "WiFi"
            } else if name.starts_with("en") {
                "ETH"
            } else {
                &name[..name.len().min(4)]
            };

            let line = Line::from(vec![
                Span::raw(format!(" {:<4} ", prefix)).fg(if is_tailscale { Color::Blue } else { Color::White }),
                Span::raw(format!("{:<15} ", ip_str)).fg(Color::DarkGray),
                "↓".green(),
                format!("{:>6} ", dl).green(),
                "↑".red(),
                format!("{:>6}", ul).red(),
            ]);

            Paragraph::new(line).render(chunks[i], buf);
        }
    }
}
