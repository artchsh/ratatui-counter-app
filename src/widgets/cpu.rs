use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Widget, Wrap},
};
use sysinfo::System;

const BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

pub struct CpuWidget<'a> {
    system: &'a System,
}

impl<'a> CpuWidget<'a> {
    pub fn new(system: &'a System) -> Self {
        Self { system }
    }

    fn usage_to_bar(usage: f32, width: usize) -> String {
        let filled = usage / 100.0 * width as f32;
        let full_blocks = filled.floor() as usize;
        let partial = filled - full_blocks as f32;
        let partial_idx = (partial * 8.0).round() as usize;

        let mut bar = String::with_capacity(width);
        for _ in 0..full_blocks {
            bar.push(BLOCKS[8]);
        }
        if full_blocks < width {
            bar.push(BLOCKS[partial_idx.min(8)]);
        }
        while bar.len() < width {
            bar.push(BLOCKS[0]);
        }
        bar
    }

    fn usage_color(usage: f32) -> Color {
        if usage > 80.0 {
            Color::Red
        } else if usage > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        }
    }
}

impl Widget for CpuWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = ratatui::widgets::Block::bordered()
            .title(Line::from(" CPU ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        let global_usage = self.system.global_cpu_usage();
        let cpus = self.system.cpus();

        let brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();
        let freq = cpus.first().map(|c| c.frequency()).unwrap_or(0);
        let physical = self.system.physical_core_count().unwrap_or(0);
        let logical = cpus.len().max(physical);

        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

        let summary = format!(
            " {} | {}C/{}T | {}MHz ",
            brand, physical, logical, freq
        );
        Paragraph::new(Line::from(summary))
            .wrap(Wrap { trim: false })
            .render(chunks[0], buf);

        let gauge_color = if global_usage > 80.0 {
            Color::Red
        } else if global_usage > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        let gauge_label = Span::styled(
            format!(" Total: {:.0}% ", global_usage),
            (Color::Black, gauge_color),
        );
        Gauge::default()
            .gauge_style(gauge_color)
            .percent(global_usage as u16)
            .label(gauge_label)
            .use_unicode(true)
            .render(chunks[1], buf);

        let max_bar_width = 25;
        let bar_width = (inner.width as usize).saturating_sub(8).min(max_bar_width).max(5);

        let is_aggregate = cpus.len() == 1 && physical > 1;

        let core_lines: Vec<Line> = if is_aggregate {
            let usage = global_usage;
            let color = Self::usage_color(usage);
            let bar = Self::usage_to_bar(usage, bar_width);
            vec![Line::from(vec![
                Span::raw("ALL ").fg(Color::DarkGray),
                Span::raw(format!("{:>3.0}% ", usage)).fg(color),
                Span::raw(bar).fg(color),
            ])]
        } else {
            let available_lines = chunks[2].height as usize;

            if cpus.len() <= available_lines.max(1) {
                cpus
                    .iter()
                    .map(|cpu| {
                        let usage = cpu.cpu_usage();
                        let color = Self::usage_color(usage);
                        let bar = Self::usage_to_bar(usage, bar_width);
                        Line::from(vec![
                            Span::raw(format!("{:>3} ", cpu.name())).fg(Color::DarkGray),
                            Span::raw(format!("{:>3.0}% ", usage)).fg(color),
                            Span::raw(bar).fg(color),
                        ])
                    })
                    .collect()
            } else {
                let max_width = chunks[2].width as usize;
                let mut lines: Vec<Vec<Span>> = vec![Vec::new()];
                let mut current_len = 0usize;

                for (idx, cpu) in cpus.iter().enumerate() {
                    let usage = cpu.cpu_usage();
                    let color = Self::usage_color(usage);
                    let block_idx = ((usage / 100.0) * 8.0).round() as usize;
                    let token = format!("{:02}{}", idx, BLOCKS[block_idx.min(8)]);
                    let token_len = token.len() + 1;

                    if current_len + token_len > max_width && !lines.last().is_some_and(|l| l.is_empty()) {
                        lines.push(Vec::new());
                        current_len = 0;
                    }

                    if let Some(line) = lines.last_mut() {
                        if !line.is_empty() {
                            line.push(Span::raw(" "));
                        }
                        line.push(Span::styled(token, color));
                        current_len += token_len;
                    }
                }

                lines
                    .into_iter()
                    .take(available_lines.max(1))
                    .map(Line::from)
                    .collect()
            }
        };

        Paragraph::new(core_lines)
            .wrap(Wrap { trim: false })
            .render(chunks[2], buf);
    }
}
