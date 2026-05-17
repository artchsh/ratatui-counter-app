use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
    DefaultTerminal, Frame,
};
use sysinfo::{Components, Disks, Networks, System};

use crate::widgets::cpu::CpuWidget;
use crate::widgets::gpu::GpuWidget;
use crate::widgets::memory::MemoryWidget;
use crate::widgets::network::NetworkWidget;
use crate::widgets::storage::StorageWidget;
use crate::widgets::temperature::TemperatureWidget;

pub struct App {
    system: System,
    networks: Networks,
    components: Components,
    disks: Disks,
    gpu: GpuWidget,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_cpu_all();
        system.refresh_memory();

        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        Self {
            system,
            networks,
            components,
            disks,
            gpu: GpuWidget::new(),
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_secs(1);
        let mut last_tick = Instant::now();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.refresh();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.networks.refresh(true);
        self.components.refresh(true);
        self.disks.refresh(true);
        self.gpu.refresh();
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Char('q') {
            self.exit = true;
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" System Monitor ".bold());
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::PLAIN);

        let inner = block.inner(area);
        block.render(area, buf);

        let vertical = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(4),
        ]).split(inner);

        let top_row = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(vertical[0]);

        let mid_row = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(vertical[1]);

        let bot_row = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(vertical[2]);

        CpuWidget::new(&self.system).render(top_row[0], buf);
        MemoryWidget::new(&self.system).render(top_row[1], buf);
        GpuWidget::new().render(mid_row[0], buf);
        NetworkWidget::new(&self.networks).render(mid_row[1], buf);
        StorageWidget::new(&self.disks).render(bot_row[0], buf);
        TemperatureWidget::new(&self.components).render(bot_row[1], buf);
    }
}