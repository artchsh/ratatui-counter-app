use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use sysinfo::{Pid, Process, System};

pub struct ProcessesWidget<'a> {
    system: &'a System,
    sort_by: SortBy,
}

#[derive(Clone, Copy)]
pub enum SortBy {
    Cpu,
    Memory,
}

impl<'a> ProcessesWidget<'a> {
    pub fn new(system: &'a System, sort_by: SortBy) -> Self {
        Self { system, sort_by }
    }

    fn get_top_processes(system: &System, sort_by: SortBy, limit: usize) -> Vec<(&Pid, &Process)> {
        let mut procs: Vec<_> = system.processes().iter().collect();
        match sort_by {
            SortBy::Cpu => procs.sort_by(|a, b| {
                b.1.cpu_usage()
                    .partial_cmp(&a.1.cpu_usage())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortBy::Memory => procs.sort_by(|a, b| {
                b.1.memory()
                    .cmp(&a.1.memory())
            }),
        }
        procs.truncate(limit);
        procs
    }

    fn truncate_name(name: &str, max: usize) -> String {
        if name.len() > max {
            format!("{}…", &name[..max - 1])
        } else {
            name.to_string()
        }
    }

    fn format_memory(bytes: u64) -> String {
        if bytes >= 1_073_741_824 {
            format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.0}M", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.0}K", bytes as f64 / 1024.0)
        } else {
            format!("{}", bytes)
        }
    }
}

impl Widget for ProcessesWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = match self.sort_by {
            SortBy::Cpu => " Processes (CPU) ",
            SortBy::Memory => " Processes (MEM) ",
        };
        let block = Block::bordered()
            .title(Line::from(title).bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        let limit = inner.height as usize;
        let procs = Self::get_top_processes(self.system, self.sort_by, limit);

        let max_name_width = (inner.width as usize).saturating_sub(12);

        let lines: Vec<Line> = procs
            .iter()
            .map(|(pid, proc)| {
                let name = Self::truncate_name(&proc.name().to_string_lossy(), max_name_width);
                let cpu = proc.cpu_usage();
                let mem = proc.memory();

                let cpu_color = if cpu > 50.0 {
                    Color::Red
                } else if cpu > 10.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                match self.sort_by {
                    SortBy::Cpu => Line::from(vec![
                        Span::raw(format!("{:<5} ", pid)).fg(Color::DarkGray),
                        Span::raw(format!("{:<width$} ", name, width = max_name_width)).fg(Color::White),
                        Span::raw(format!("{:>5.1}%", cpu)).fg(cpu_color),
                    ]),
                    SortBy::Memory => Line::from(vec![
                        Span::raw(format!("{:<5} ", pid)).fg(Color::DarkGray),
                        Span::raw(format!("{:<width$} ", name, width = max_name_width)).fg(Color::White),
                        Span::raw(format!("{:>6}", Self::format_memory(mem))).fg(Color::Cyan),
                    ]),
                }
            })
            .collect();

        Paragraph::new(lines).render(inner, buf);
    }
}
