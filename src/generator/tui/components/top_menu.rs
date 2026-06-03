//! Top-level generator menu component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::generator::{
    domain::GeneratorFlow,
    tui::{
        app::{Screen, WizardApp},
        components::common::{self, WizardComponent},
    },
};

pub struct TopMenu;

impl WizardComponent for TopMenu {
    fn enter(&self, app: &mut WizardApp) {
        app.menu_index = 0;
        app.status = top_menu_status(app);
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        common::render_menu(frame, area, "Top menu", MENU, app.menu_index);
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if common::quit_on_escape(app, key) || common::move_menu(app, key, MENU.len() - 1) {
            return Ok(());
        }
        if matches!(key.code, KeyCode::Enter) {
            select(app);
        }
        Ok(())
    }
}

const MENU: &[&str] = &["Generate Route", "Generate Main Configuration", "Exit"];

fn top_menu_status(app: &mut WizardApp) -> String {
    app.last_status
        .take()
        .unwrap_or_else(|| "Select what to generate".to_string())
}

fn select(app: &mut WizardApp) {
    match app.menu_index {
        0 => start_route(app),
        1 => start_config(app),
        _ => app.finish(),
    }
}

fn start_route(app: &mut WizardApp) {
    app.selected_flow = Some(GeneratorFlow::Route);
    app.transition_to(Screen::RouteKind);
}

fn start_config(app: &mut WizardApp) {
    app.selected_flow = Some(GeneratorFlow::MainConfig);
    app.transition_to(Screen::MainConfig);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::tui::app::WizardApp;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn enter_consumes_last_status_once() {
        let mut app = WizardApp::new("mocks");
        app.last_status = Some("Files generated".to_string());
        TopMenu.enter(&mut app);
        assert_eq!(app.status, "Files generated");
        TopMenu.enter(&mut app);
        assert_eq!(app.status, "Select what to generate");
    }

    #[test]
    fn selecting_route_starts_route_flow() {
        let mut app = WizardApp::new("mocks");
        TopMenu.handle_key(&mut app, key(KeyCode::Enter)).unwrap();
        assert_eq!(app.selected_flow, Some(GeneratorFlow::Route));
        assert_eq!(app.screen, Screen::RouteKind);
    }

    #[test]
    fn selecting_main_config_starts_config_flow() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 1;
        TopMenu.handle_key(&mut app, key(KeyCode::Enter)).unwrap();
        assert_eq!(app.selected_flow, Some(GeneratorFlow::MainConfig));
        assert_eq!(app.screen, Screen::MainConfig);
    }

    #[test]
    fn selecting_exit_finishes_app() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 2;
        TopMenu.handle_key(&mut app, key(KeyCode::Enter)).unwrap();
        assert!(app.done);
    }
}
