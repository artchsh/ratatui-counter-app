mod widgets;
mod app;

use std::io;

fn main() -> io::Result<()> {
    let mut app = app::App::new();
    ratatui::run(|terminal| app.run(terminal))
}