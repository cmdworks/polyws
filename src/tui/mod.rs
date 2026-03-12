/// Full-screen interactive TUI for polyws.
///
/// Launched when `polyws` is invoked with no sub-command from inside a workspace.
/// Navigation: Tab/Shift-Tab or number keys switch tabs.
/// Each tab shows a table; actions are driven by the keys shown in the bottom bar.
use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod helpers;
mod input;
mod render;
mod types;

use app::App;
use input::handle_key;
use render::draw;

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        crossterm::terminal::SetTitle("polyws"),
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Force dark background so we never render on a light terminal
    terminal.clear()?;

    let mut app = App::new();
    app.refresh_statuses();

    let tick = Duration::from_millis(500);
    loop {
        terminal.draw(|f| draw(f, &mut app))?;
        app.tick();

        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                if handle_key(&mut app, key.code, key.modifiers) {
                    break;
                }
            }
        }

        // Clear status after 4 seconds.
        if app.status_msg.is_some() && app.last_tick.elapsed() > Duration::from_secs(4) {
            app.status_msg = None;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
