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

    fn is_loopback(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower == "lo" || lower == "lo0" || lower.starts_with("loopback")
    }

    fn is_virtual(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.starts_with("docker")
            || lower.starts_with("br-")
            || lower.starts_with("veth")
            || lower.starts_with("vet")
            || lower.starts_with("virtual")
            || lower.starts_with("vmware")
            || lower.starts_with("vmnet")
            || lower.starts_with("virbr")
            || lower.starts_with("pan")
            || lower.starts_with("awdl")
            || lower.starts_with("llw")
    }

    fn get_interfaces(networks: &Networks) -> Vec<(&String, &sysinfo::NetworkData)> {
        networks
            .iter()
            .filter(|(name, data)| {
                if Self::is_loopback(name) {
                    return false;
                }
                if Self::is_virtual(name) {
                    return false;
                }
                data.ip_networks().iter().any(|ip| ip.addr.is_ipv4())
            })
            .collect()
    }

    fn detect_type(name: &str) -> (&'static str, Color) {
        let lower = name.to_lowercase();

        if lower.starts_with("utun") || lower.starts_with("tailscale") || lower.starts_with("wg") {
            return ("VPN", Color::Blue);
        }

        if lower.starts_with("wlan") || lower.starts_with("wl") || lower.starts_with("wi-fi") {
            return ("WiFi", Color::White);
        }

        if lower.starts_with("en") && !lower.contains("bridge") && !lower.contains("ether") {
            return ("WiFi", Color::White);
        }

        if lower.starts_with("eth") || lower.starts_with("eno") || lower.starts_with("ens") || lower.starts_with("enp") {
            return ("ETH", Color::White);
        }

        if lower.starts_with("veth") || lower.starts_with("br-") || lower.starts_with("docker") {
            return ("VIRT", Color::DarkGray);
        }

        ("NET", Color::White)
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

            let (prefix, color) = Self::detect_type(name);

            let line = Line::from(vec![
                Span::raw(format!(" {:<4} ", prefix)).fg(color),
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
