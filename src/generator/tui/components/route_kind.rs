//! Route type menu component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::generator::tui::{
    app::{self, Screen, WizardApp},
    components::common::{self, WizardComponent},
};

pub struct RouteKindMenu;

impl WizardComponent for RouteKindMenu {
    fn enter(&self, app: &mut WizardApp) {
        app.menu_index = 0;
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        common::render_menu(frame, area, "Route type", ROUTE_KINDS, app.menu_index);
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if common::quit_on_escape(app, key)
            || common::go_back(app, key)
            || common::move_menu(app, key, last_index())
        {
            return Ok(());
        }
        if matches!(key.code, KeyCode::Enter) {
            select(app);
        }
        Ok(())
    }
}

const ROUTE_KINDS: &[&str] = &[
    "Basic", "REST", "Auth", "Upload", "Public", "GraphQL", "SQL",
];

fn last_index() -> usize {
    ROUTE_KINDS.len() - 1
}

fn select(app: &mut WizardApp) {
    app.route_selection.kind = app::route_kind_from_index(app.menu_index);
    app.transition_to(Screen::RoutePath);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{domain::RouteKind, tui::app::WizardApp};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn enter_resets_menu_index() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 5;
        RouteKindMenu.enter(&mut app);
        assert_eq!(app.menu_index, 0);
    }

    #[test]
    fn down_moves_until_last_route_kind() {
        let mut app = WizardApp::new("mocks");
        for _ in 0..10 {
            RouteKindMenu
                .handle_key(&mut app, key(KeyCode::Down))
                .unwrap();
        }
        assert_eq!(app.menu_index, 6);
    }

    #[test]
    fn selecting_graphql_opens_route_path() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 5;
        RouteKindMenu
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.route_selection.kind, RouteKind::Graphql);
        assert_eq!(app.screen, Screen::RoutePath);
    }

    #[test]
    fn left_returns_to_top_menu() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::RouteKind);
        RouteKindMenu
            .handle_key(&mut app, key(KeyCode::Left))
            .unwrap();
        assert_eq!(app.screen, Screen::TopMenu);
    }
}
