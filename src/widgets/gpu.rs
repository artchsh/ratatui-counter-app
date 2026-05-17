use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

struct GpuInfo {
    name: String,
    utilization: u32,
    memory_used: u64,
    memory_total: u64,
    temperature: u32,
    power_usage: u32,
}

pub struct GpuWidget {
    info: Option<GpuInfo>,
}

impl GpuWidget {
    pub fn new() -> Self {
        let info = Self::fetch_gpu_info();
        Self { info }
    }

    fn fetch_gpu_info() -> Option<GpuInfo> {
        let nvml = nvml_wrapper::Nvml::init().ok()?;
        let device = nvml.device_by_index(0).ok()?;

        let name = device.name().ok()?.to_string();
        let utilization = device.utilization_rates().ok()?;
        let memory = device.memory_info().ok()?;

        let temperature = device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .ok()
            .unwrap_or(0);
        let power = device.power_usage().ok().unwrap_or(0);

        Some(GpuInfo {
            name,
            utilization: utilization.gpu,
            memory_used: memory.used,
            memory_total: memory.total,
            temperature,
            power_usage: power / 1000,
        })
    }

    pub fn refresh(&mut self) {
        self.info = Self::fetch_gpu_info();
    }
}

impl Widget for GpuWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" GPU ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        match self.info {
            Some(info) => {
                let mem_used_gb = info.memory_used as f64 / 1024.0 / 1024.0 / 1024.0;
                let mem_total_gb = info.memory_total as f64 / 1024.0 / 1024.0 / 1024.0;
                let mem_percent = if mem_total_gb > 0.0 {
                    (mem_used_gb / mem_total_gb * 100.0) as u16
                } else {
                    0
                };

                let temp_color = if info.temperature > 80 {
                    Color::Red
                } else if info.temperature > 60 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                let lines = vec![
                    Line::from(format!(" {} | {}% | {}W ", info.name, info.utilization, info.power_usage).bold()),
                    Line::from(format!(
                        " VRAM: {:.1}/{:.1}GB ({}%) ",
                        mem_used_gb, mem_total_gb, mem_percent
                    )),
                    Line::from(format!(" Temp: {}°C ", info.temperature)).style(temp_color),
                ];

                Paragraph::new(lines).render(inner, buf);
            }
            None => {
                Paragraph::new(Line::from(" No NVIDIA GPU "))
                    .style(Color::DarkGray)
                    .render(inner, buf);
            }
        }
    }
}
