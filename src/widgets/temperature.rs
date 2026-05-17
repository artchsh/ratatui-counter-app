use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use sysinfo::Components;

pub struct TemperatureWidget<'a> {
    components: &'a Components,
}

impl<'a> TemperatureWidget<'a> {
    pub fn new(components: &'a Components) -> Self {
        Self { components }
    }

    fn simplify_label(label: &str) -> String {
        let lower = label.to_lowercase();

        if lower.contains("cpu") || lower.contains("core") || lower.contains("package") {
            if lower.contains("package") {
                return "CPU".to_string();
            }
            if let Some(num) = lower.chars().find(|c| c.is_ascii_digit()) {
                return format!("C{}", num);
            }
            return "CPU".to_string();
        }

        if lower.contains("gpu") || lower.contains("nvidia") || lower.contains("amd") {
            return "GPU".to_string();
        }

        if lower.contains("soc") {
            return "SoC".to_string();
        }

        if lower.contains("battery") || lower.contains("bat") {
            return "BAT".to_string();
        }

        if lower.contains("nand") || lower.contains("ssd") || lower.contains("disk") {
            return "SSD".to_string();
        }

        if lower.contains("pmu") || lower.contains("tdev") || lower.contains("ans") {
            return String::new();
        }

        let cleaned: String = label
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
            .collect();

        if cleaned.len() > 10 {
            cleaned[..10].to_string()
        } else {
            cleaned
        }
    }
}

impl Widget for TemperatureWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Temp ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.components.is_empty() {
            Paragraph::new(Line::from(" No sensors "))
                .style(Color::DarkGray)
                .render(inner, buf);
            return;
        }

        let mut temps: Vec<(String, f32)> = Vec::new();

        for component in self.components.iter() {
            let temp = match component.temperature() {
                Some(t) if t > 0.0 && t < 150.0 => t,
                _ => continue,
            };

            let label = Self::simplify_label(component.label());
            if label.is_empty() {
                continue;
            }

            temps.push((label, temp));
        }

        if temps.is_empty() {
            Paragraph::new(Line::from(" No valid sensors "))
                .style(Color::DarkGray)
                .render(inner, buf);
            return;
        }

        let mut parts: Vec<Line> = Vec::new();

        for (label, temp) in temps {
            let color = if temp > 80.0 {
                Color::Red
            } else if temp > 60.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            parts.push(Line::from(vec![
                Span::raw(format!(" {:<5} ", label)).fg(Color::White),
                Span::raw(format!("{:>3.0}°C", temp)).fg(color),
            ]));
        }

        Paragraph::new(parts).render(inner, buf);
    }
}
