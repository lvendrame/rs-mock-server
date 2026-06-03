//! SQL collection-name input component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::generator::tui::{
    app::{Screen, WizardApp},
    components::common,
};

pub struct RouteCollectionInput;

impl common::WizardComponent for RouteCollectionInput {
    fn enter(&self, app: &mut WizardApp) {
        app.input = input_value(app);
        app.status = "Enter SQL collection/table name, then press Enter".to_string();
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        common::render_input(frame, area, "SQL collection", &app.input);
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if common::quit_on_escape(app, key) || common::edit_input_or_go_back(app, key) {
            return Ok(());
        }
        if matches!(key.code, KeyCode::Enter) {
            submit(app);
        }
        Ok(())
    }
}

fn input_value(app: &WizardApp) -> String {
    app.route_selection
        .collection_name
        .clone()
        .unwrap_or_else(|| route_leaf(&app.route_selection.route))
}

fn submit(app: &mut WizardApp) {
    if !app.input.trim().is_empty() {
        app.route_selection.collection_name = Some(app.input.trim().to_string());
    }
    app.transition_to(Screen::RouteOptions);
}

fn route_leaf(route: &str) -> String {
    route
        .split('/')
        .filter(|segment| !segment.is_empty())
        .filter(|segment| !(segment.starts_with('{') && segment.ends_with('}')))
        .next_back()
        .and_then(|segment| segment.split('{').next())
        .unwrap_or("items")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::tui::components::common::WizardComponent;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn enter_uses_collection_override() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.collection_name = Some("orders".to_string());
        RouteCollectionInput.enter(&mut app);
        assert_eq!(app.input, "orders");
    }

    #[test]
    fn enter_falls_back_to_route_leaf() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.route = "/reports/companies{id}".to_string();
        RouteCollectionInput.enter(&mut app);
        assert_eq!(app.input, "companies");
    }

    #[test]
    fn submit_stores_collection_and_opens_options() {
        let mut app = WizardApp::new("mocks");
        app.input = "archived_orders".to_string();
        submit(&mut app);
        assert_eq!(
            app.route_selection.collection_name,
            Some("archived_orders".to_string())
        );
        assert_eq!(app.screen, Screen::RouteOptions);
    }

    #[test]
    fn left_returns_to_route_path() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::RoutePath);
        app.transition_to(Screen::RouteCollection);
        RouteCollectionInput
            .handle_key(&mut app, key(KeyCode::Left))
            .unwrap();
        assert_eq!(app.screen, Screen::RoutePath);
    }
}
