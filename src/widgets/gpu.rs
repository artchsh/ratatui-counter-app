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

    pub fn refresh(&mut self) {
        self.info = Self::fetch_gpu_info();
    }

    #[cfg(not(windows))]
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

    #[cfg(windows)]
    fn fetch_gpu_info() -> Option<GpuInfo> {
        use windows::Win32::Graphics::Dxgi::*;
        use windows::core::Interface;

        unsafe {
            let factory: IDXGIFactory1 = CreateDXGIFactory1().ok()?;

            for adapter_idx in 0u32.. {
                let adapter = match factory.EnumAdapters1(adapter_idx) {
                    Ok(a) => a,
                    Err(_) => break,
                };

                let mut desc = std::mem::zeroed();
                if adapter.GetDesc1(&mut desc).is_err() {
                    continue;
                }

                let name = String::from_utf16_lossy(&desc.Description)
                    .trim_matches('\0')
                    .to_string();

                if name.is_empty() || name.contains("Microsoft Basic Render") {
                    continue;
                }

                let (mem_used, mem_total) = match adapter.cast::<IDXGIAdapter3>() {
                    Ok(adapter3) => {
                        match adapter3.QueryVideoMemoryInfo(
                            0,
                            DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                        ) {
                            Ok(info) => (info.CurrentUsage, info.Budget),
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                };

                return Some(GpuInfo {
                    name,
                    utilization: 0,
                    memory_used: mem_used,
                    memory_total: mem_total,
                    temperature: 0,
                    power_usage: 0,
                });
            }
            None
        }
    }
}

impl Widget for &GpuWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" GPU ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        match &self.info {
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
                Paragraph::new(Line::from(" No GPU detected "))
                    .style(Color::DarkGray)
                    .render(inner, buf);
            }
        }
    }
}
