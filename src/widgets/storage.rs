use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Gauge, Paragraph, Widget},
};
use sysinfo::{DiskKind, Disks};
use std::collections::HashMap;

struct StorageCategory {
    label: &'static str,
    color: Color,
    used: u64,
    total: u64,
}

pub struct StorageWidget<'a> {
    disks: &'a Disks,
}

impl<'a> StorageWidget<'a> {
    pub fn new(disks: &'a Disks) -> Self {
        Self { disks }
    }

    fn categorize_disks(disks: &Disks) -> Vec<StorageCategory> {
        let mut seen: HashMap<String, bool> = HashMap::new();
        let mut ssd = StorageCategory { label: "SSD", color: Color::Blue, used: 0, total: 0 };
        let mut hdd = StorageCategory { label: "HDD", color: Color::Yellow, used: 0, total: 0 };
        let mut nvme = StorageCategory { label: "NVME", color: Color::Magenta, used: 0, total: 0 };

        for disk in disks.iter() {
            let disk_name = disk.name().to_string_lossy().to_string();
            if disk_name.is_empty() {
                continue;
            }

            let physical_name = Self::get_physical_device_name(&disk_name);
            if seen.contains_key(&physical_name) {
                continue;
            }
            seen.insert(physical_name.clone(), true);

            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            let is_nvme = Self::is_nvme(&disk_name, &physical_name);

            match (disk.kind(), is_nvme) {
                (_, true) => {
                    nvme.used += used;
                    nvme.total += total;
                }
                (DiskKind::SSD, false) => {
                    ssd.used += used;
                    ssd.total += total;
                }
                (DiskKind::HDD, _) => {
                    hdd.used += used;
                    hdd.total += total;
                }
                (DiskKind::Unknown(_), _) => {
                    ssd.used += used;
                    ssd.total += total;
                }
            }
        }

        vec![ssd, hdd, nvme].into_iter().filter(|c| c.total > 0).collect()
    }

    fn is_nvme(disk_name: &str, physical_name: &str) -> bool {
        let d = disk_name.to_lowercase();
        let p = physical_name.to_lowercase();
        d.starts_with("nvme") || d.contains("nvme") || p.starts_with("nvme") || p.contains("nvme")
    }

    fn get_physical_device_name(name: &str) -> String {
        let lower = name.to_lowercase();

        if lower.starts_with("nvme") {
            if let Some(pos) = lower.find('p') {
                return lower[..pos].to_string();
            }
            return lower.clone();
        }

        if lower.starts_with("disk")
            && let Some(rest) = lower.strip_prefix("disk") {
            let base: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            return format!("disk{}", base);
        }

        if lower.starts_with("rdisk")
            && let Some(rest) = lower.strip_prefix("rdisk") {
            let base: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            return format!("disk{}", base);
        }

        if (lower.starts_with("sd") || lower.starts_with("hd") || lower.starts_with("vd")) && lower.len() >= 3 {
            let base: String = lower.chars().take(3).collect();
            return base;
        }

        if lower.len() == 2 && lower.ends_with(':') {
            return lower.clone();
        }

        name.to_string()
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1_099_511_627_776 {
            format!("{:.1}TB", bytes as f64 / 1_099_511_627_776.0)
        } else if bytes >= 1_073_741_824 {
            format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.1}MB", bytes as f64 / 1_048_576.0)
        } else {
            format!("{}B", bytes)
        }
    }
}

impl Widget for StorageWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Storage ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.disks.is_empty() {
            Paragraph::new(Line::from(" No disks detected "))
                .style(Color::DarkGray)
                .render(inner, buf);
            return;
        }

        let categories = Self::categorize_disks(self.disks);
        let total_used: u64 = categories.iter().map(|c| c.used).sum();
        let total_all: u64 = categories.iter().map(|c| c.total).sum();
        let total_percent = if total_all > 0 { (total_used as f64 / total_all as f64 * 100.0) as u16 } else { 0 };

        let available_lines = inner.height as usize;
        if available_lines == 0 {
            return;
        }

        let chunks = Layout::vertical(std::iter::repeat_n(Constraint::Length(1), available_lines)).split(inner);

        let total_label = ratatui::text::Span::styled(
            format!(" Total: {} / {} ({:>3}%) ",
                Self::format_bytes(total_used),
                Self::format_bytes(total_all),
                total_percent),
            (Color::Black, if total_percent > 90 { Color::Red } else if total_percent > 70 { Color::Yellow } else { Color::Green }),
        );
        let total_color = if total_percent > 90 { Color::Red } else if total_percent > 70 { Color::Yellow } else { Color::Green };
        Gauge::default()
            .gauge_style(total_color)
            .percent(total_percent)
            .label(total_label)
            .use_unicode(true)
            .render(chunks[0], buf);

        if available_lines <= 1 {
            return;
        }

        let detail_slots = available_lines.saturating_sub(1);
        let categories_to_show: Vec<&StorageCategory> = categories.iter().take(detail_slots).collect();

        for (chunk_idx, cat) in categories_to_show.iter().enumerate() {
            let percent = if cat.total > 0 { (cat.used as f64 / cat.total as f64 * 100.0) as u16 } else { 0 };
            let label = ratatui::text::Span::styled(
                format!(" {}: {} / {} ({:>3}%) ",
                    cat.label,
                    Self::format_bytes(cat.used),
                    Self::format_bytes(cat.total),
                    percent),
                (Color::Black, cat.color),
            );
            Gauge::default()
                .gauge_style(cat.color)
                .percent(percent)
                .label(label)
                .use_unicode(true)
                .render(chunks[chunk_idx + 1], buf);
        }
    }
}
