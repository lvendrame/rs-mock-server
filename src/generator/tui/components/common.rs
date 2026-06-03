//! Common TUI component contracts and widgets.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::generator::tui::app::WizardApp;

pub trait WizardComponent {
    fn enter(&self, app: &mut WizardApp);
    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp);
    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()>;
}

pub fn render_header(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(
        Paragraph::new("rs-mock-server generator").block(block("Generate")),
        area,
    );
}

pub fn render_status(frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
    let text = vec![Line::from(app.status.clone()), Line::from(help_text())];
    frame.render_widget(Paragraph::new(text).block(block("Status")), area);
}

pub fn render_menu(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &'static str,
    labels: &[&str],
    selected: usize,
) {
    frame.render_widget(
        List::new(menu_items(labels, selected)).block(block(title)),
        area,
    );
}

pub fn render_input(frame: &mut Frame<'_>, area: Rect, title: &'static str, value: &str) {
    let widget = Paragraph::new(value.to_string()).block(block(title));
    frame.render_widget(widget, area);
}

pub fn render_lines(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &'static str,
    lines: Vec<Line<'static>>,
) {
    frame.render_widget(Paragraph::new(lines).block(block(title)), area);
}

pub fn move_menu(app: &mut WizardApp, key: KeyEvent, max: usize) -> bool {
    match key.code {
        KeyCode::Up => app.menu_index = app.menu_index.saturating_sub(1),
        KeyCode::Down => app.menu_index = (app.menu_index + 1).min(max),
        _ => return false,
    }
    true
}

pub fn move_index(index: &mut usize, key: KeyEvent, max: usize) -> bool {
    match key.code {
        KeyCode::Up => *index = index.saturating_sub(1),
        KeyCode::Down => *index = (*index + 1).min(max),
        _ => return false,
    }
    true
}

pub fn edit_input(app: &mut WizardApp, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Backspace => app.input.pop().is_some(),
        KeyCode::Char(ch) => {
            app.input.push(ch);
            true
        }
        _ => false,
    }
}

pub fn go_back(app: &mut WizardApp, key: KeyEvent) -> bool {
    if is_back_key(key) {
        app.go_back();
        return true;
    }
    false
}

pub fn edit_input_or_go_back(app: &mut WizardApp, key: KeyEvent) -> bool {
    if matches!(key.code, KeyCode::Left) {
        app.go_back();
        return true;
    }
    if is_delete_key(key) && app.input.is_empty() {
        app.go_back();
        return true;
    }
    edit_input(app, key)
}

pub fn quit_on_escape(app: &mut WizardApp, key: KeyEvent) -> bool {
    if matches!(key.code, KeyCode::Esc) {
        app.finish();
        return true;
    }
    false
}

pub fn block(title: &'static str) -> Block<'static> {
    Block::default().borders(Borders::ALL).title(title)
}

fn menu_items(labels: &[&str], selected: usize) -> Vec<ListItem<'static>> {
    labels
        .iter()
        .enumerate()
        .map(|(index, label)| ListItem::new(menu_line(label, index == selected)))
        .collect()
}

fn menu_line(label: &str, selected: bool) -> Line<'static> {
    match selected {
        true => Line::from(vec![Span::styled(format!("> {}", label), selected_style())]),
        false => Line::from(format!("  {}", label)),
    }
}

fn selected_style() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

fn is_back_key(key: KeyEvent) -> bool {
    matches!(
        key.code,
        KeyCode::Left | KeyCode::Char('b') | KeyCode::Backspace
    )
}

fn is_delete_key(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Backspace)
}

fn help_text() -> &'static str {
    "Enter: next/write | Left: back | b: back outside inputs | Esc: quit"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::tui::app::{Screen, WizardApp};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn move_menu_bounds_selection() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 1;
        assert!(move_menu(&mut app, key(KeyCode::Up), 2));
        assert_eq!(app.menu_index, 0);
        assert!(move_menu(&mut app, key(KeyCode::Up), 2));
        assert_eq!(app.menu_index, 0);
        assert!(move_menu(&mut app, key(KeyCode::Down), 1));
        assert_eq!(app.menu_index, 1);
        assert!(move_menu(&mut app, key(KeyCode::Down), 1));
        assert_eq!(app.menu_index, 1);
    }

    #[test]
    fn edit_input_keeps_printable_back_key_as_text() {
        let mut app = WizardApp::new("mocks");
        assert!(edit_input(&mut app, key(KeyCode::Char('b'))));
        assert_eq!(app.input, "b");
    }

    #[test]
    fn edit_input_or_go_back_uses_left_as_back() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::RouteKind);
        app.transition_to(Screen::RoutePath);
        app.input = "/api/users".to_string();
        assert!(edit_input_or_go_back(&mut app, key(KeyCode::Left)));
        assert_eq!(app.screen, Screen::RouteKind);
    }

    #[test]
    fn escape_marks_app_done() {
        let mut app = WizardApp::new("mocks");
        assert!(quit_on_escape(&mut app, key(KeyCode::Esc)));
        assert!(app.done);
    }
}
