use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
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

        if lower.contains("cpu") || lower.contains("core") || lower.contains("package") || lower.contains("tctl") {
            if lower.contains("package") || lower.contains("tctl") {
                return "CPU".to_string();
            }
            return "CPU".to_string();
        }

        if lower.contains("gpu") || lower.contains("nvidia") || lower.contains("amd") || lower.contains("radeon") {
            return "GPU".to_string();
        }

        if lower.contains("memory") || lower.contains("ram") || lower.contains("dram") || lower.contains("ddr") {
            return "RAM".to_string();
        }

        if lower.contains("soc") {
            return "SoC".to_string();
        }

        if lower.contains("battery") || lower.contains("bat") {
            return "BAT".to_string();
        }

        if lower.contains("nand") || lower.contains("ssd") || lower.contains("disk") || lower.contains("nvme") {
            return "SSD".to_string();
        }

        if lower.contains("pmu") || lower.contains("tdev") || lower.contains("ans") {
            return String::new();
        }

        if lower.contains("acpitz") || lower.contains("thermal") {
            return "SYS".to_string();
        }

        if lower.contains("pch") || lower.contains("chipset") {
            return "PCH".to_string();
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

    fn find_sensor(components: &Components, target: &str) -> Option<f32> {
        for component in components.iter() {
            let label = Self::simplify_label(component.label());
            if label == target
                && let Some(t) = component.temperature()
                && t > 0.0 && t < 150.0 {
                return Some(t);
            }
        }
        None
    }

    #[cfg(not(windows))]
    fn fallback_gpu_temp() -> Option<u32> {
        let nvml = nvml_wrapper::Nvml::init().ok()?;
        let device = nvml.device_by_index(0).ok()?;
        device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .ok()
    }

    #[cfg(windows)]
    fn fallback_gpu_temp() -> Option<u32> {
        None
    }
}

impl Widget for TemperatureWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Temp ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

        let cpu_temp = Self::find_sensor(self.components, "CPU");

        let gpu_temp = Self::find_sensor(self.components, "GPU")
            .or_else(|| Self::fallback_gpu_temp().map(|t| t as f32));

        let ram_temp = Self::find_sensor(self.components, "RAM");

        let render_temp_line = |label: &str, temp: Option<f32>| -> Line {
            if let Some(t) = temp {
                let color = if t > 80.0 {
                    Color::Red
                } else if t > 60.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                Line::from(vec![
                    Span::raw(format!(" {:<5} ", label)).fg(Color::White),
                    Span::raw(format!("{:>3.0}°C", t)).fg(color),
                ])
            } else {
                Line::from(vec![
                    Span::raw(format!(" {:<5} ", label)).fg(Color::White),
                    Span::raw(" N/A  ").fg(Color::DarkGray),
                ])
            }
        };

        Paragraph::new(render_temp_line("CPU", cpu_temp)).render(chunks[0], buf);
        Paragraph::new(render_temp_line("GPU", gpu_temp)).render(chunks[1], buf);
        Paragraph::new(render_temp_line("RAM", ram_temp)).render(chunks[2], buf);

        let mut extras: Vec<Line> = Vec::new();
        for component in self.components.iter() {
            let temp = match component.temperature() {
                Some(t) if t > 0.0 && t < 150.0 => t,
                _ => continue,
            };

            let label = Self::simplify_label(component.label());
            if label.is_empty() || label == "CPU" || label == "GPU" || label == "RAM" {
                continue;
            }

            let color = if temp > 80.0 {
                Color::Red
            } else if temp > 60.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            extras.push(Line::from(vec![
                Span::raw(format!(" {:<5} ", label)).fg(Color::White),
                Span::raw(format!("{:>3.0}°C", temp)).fg(color),
            ]));
        }

        if !extras.is_empty() {
            Paragraph::new(extras).render(chunks[3], buf);
        }
    }
}
