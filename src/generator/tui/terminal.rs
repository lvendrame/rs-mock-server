//! Terminal setup and event loop for the generator wizard.

use std::{io, time::Duration};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::generator::tui::{app::WizardApp, render};

type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub fn run(folder: &str) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, folder);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut AppTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run_loop(terminal: &mut AppTerminal, folder: &str) -> io::Result<()> {
    let mut app = WizardApp::new(folder);
    while !app.done {
        draw_once(terminal, &app)?;
        handle_next_event(&mut app)?;
    }
    Ok(())
}

fn draw_once(terminal: &mut AppTerminal, app: &WizardApp) -> io::Result<()> {
    terminal.draw(|frame| render::render(frame, app))?;
    Ok(())
}

fn handle_next_event(app: &mut WizardApp) -> io::Result<()> {
    if event::poll(Duration::from_millis(250))? {
        handle_polled_event(app)?;
    }
    Ok(())
}

fn handle_polled_event(app: &mut WizardApp) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {
        app.handle_key(key)?;
    }
    Ok(())
}
