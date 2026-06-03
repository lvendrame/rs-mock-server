//! Page layout for the generator wizard.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::generator::tui::{app::WizardApp, components};

pub fn render(frame: &mut Frame<'_>, app: &WizardApp) {
    let chunks = page_chunks(frame.area());
    components::render_header(frame, chunks[0]);
    components::render_screen(frame, chunks[1], app);
    components::render_status(frame, chunks[2], app);
}

fn page_chunks(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(area)
}
