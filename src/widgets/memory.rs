use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Gauge, Widget},
};
use sysinfo::System;

pub struct MemoryWidget<'a> {
    system: &'a System,
}

impl<'a> MemoryWidget<'a> {
    pub fn new(system: &'a System) -> Self {
        Self { system }
    }
}

impl Widget for MemoryWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Memory ").bold())
            .border_set(ratatui::symbols::border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let swap_total = self.system.total_swap();
        let swap_used = self.system.used_swap();

        let usage_percent = if total > 0 {
            (used as f64 / total as f64 * 100.0) as u16
        } else {
            0
        };

        let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
        let swap_total_gb = swap_total as f64 / 1024.0 / 1024.0 / 1024.0;
        let swap_used_gb = swap_used as f64 / 1024.0 / 1024.0 / 1024.0;

        let color = if usage_percent > 90 {
            Color::Red
        } else if usage_percent > 70 {
            Color::Yellow
        } else {
            Color::Cyan
        };

        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

        let label = ratatui::text::Span::styled(
            format!(" RAM: {:.1}/{:.1}GB ({:>3}%) ", used_gb, total_gb, usage_percent),
            (Color::Black, color),
        );

        Gauge::default()
            .gauge_style(color)
            .percent(usage_percent)
            .label(label)
            .use_unicode(true)
            .render(chunks[0], buf);

        if swap_total > 0 {
            let swap_usage = if swap_total > 0 {
                (swap_used as f64 / swap_total as f64 * 100.0) as u16
            } else {
                0
            };
            let swap_label = ratatui::text::Span::styled(
                format!(" SWP: {:.1}/{:.1}GB ({:>3}%) ", swap_used_gb, swap_total_gb, swap_usage),
                (Color::Black, Color::Magenta),
            );
            Gauge::default()
                .gauge_style(Color::Magenta)
                .percent(swap_usage)
                .label(swap_label)
                .use_unicode(true)
                .render(chunks[1], buf);
        }
    }
}
