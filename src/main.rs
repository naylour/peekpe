pub mod app;
pub mod pe;
pub mod tui;
pub mod ui;

use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = app::App::new().and_then(|mut app| app.run(terminal));
    ratatui::restore();
    result
}
