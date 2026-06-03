//! Route path input component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::generator::domain::{GeneratedFileType, RouteKind};
use crate::generator::tui::{
    app::{self, WizardApp},
    components::common::{self, WizardComponent},
};

pub struct RoutePathInput;

impl WizardComponent for RoutePathInput {
    fn enter(&self, app: &mut WizardApp) {
        app.input = route_input_value(app);
        app.status = route_status(app).to_string();
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        common::render_input(frame, area, route_title(app), &app.input);
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

fn submit(app: &mut WizardApp) {
    update_route_from_component_input(app);
    app.transition_to(app::next_route_screen(app.route_selection.kind));
}

fn route_input_value(app: &WizardApp) -> String {
    match app.route_selection.kind {
        RouteKind::Graphql => route_leaf(&app.route_selection.route, "users"),
        RouteKind::Upload => route_leaf(&app.route_selection.route, "upload"),
        RouteKind::Public => route_leaf(&app.route_selection.route, "public"),
        _ => app.route_selection.route.clone(),
    }
}

fn route_title(app: &WizardApp) -> &'static str {
    match app.route_selection.kind {
        RouteKind::Graphql => "GraphQL service name",
        RouteKind::Upload => "Upload route name",
        RouteKind::Public => "Public route alias",
        RouteKind::Sql => "SQL GET route",
        RouteKind::Auth => "Auth base route",
        RouteKind::Rest => "REST resource route",
        RouteKind::Basic => "Route path",
    }
}

fn route_status(app: &WizardApp) -> &'static str {
    match app.route_selection.kind {
        RouteKind::Graphql => "Enter collection/service name, for example users",
        RouteKind::Upload => "Enter upload route name, for example upload or documents",
        RouteKind::Public => "Enter public route alias, for example public or assets",
        RouteKind::Sql => "Enter SQL route path, then press Enter",
        RouteKind::Auth => "Enter auth base route, for example /auth",
        RouteKind::Rest => "Enter REST resource route, for example /api/users",
        RouteKind::Basic => "Enter file-backed route path, then press Enter",
    }
}

fn update_route_from_component_input(app: &mut WizardApp) {
    match app.route_selection.kind {
        RouteKind::Graphql => update_named_route(app, "users"),
        RouteKind::Upload => update_named_route(app, "upload"),
        RouteKind::Public => update_named_route(app, "public"),
        RouteKind::Sql => {
            app.route_selection.file_type = GeneratedFileType::Sql;
            app.update_route_from_input();
        }
        _ => app.update_route_from_input(),
    }
}

fn update_named_route(app: &mut WizardApp, default_name: &str) {
    let name = app.input.trim();
    let name = if name.is_empty() { default_name } else { name };
    app.route_selection.route = format!("/{}", name.trim_matches('/'));
}

fn route_leaf(route: &str, default_name: &str) -> String {
    route
        .split('/')
        .filter(|segment| !segment.is_empty())
        .next_back()
        .unwrap_or(default_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{
        domain::RouteKind,
        tui::app::{Screen, WizardApp},
    };

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn enter_loads_route_into_input() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.route = "/api/orders".to_string();
        RoutePathInput.enter(&mut app);
        assert_eq!(app.input, "/api/orders");
    }

    #[test]
    fn printable_key_edits_route_input() {
        let mut app = WizardApp::new("mocks");
        app.input = "/api/user".to_string();
        RoutePathInput
            .handle_key(&mut app, key(KeyCode::Char('s')))
            .unwrap();
        assert_eq!(app.input, "/api/users");
    }

    #[test]
    fn submit_basic_route_goes_to_options() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Basic;
        app.input = "api/users".to_string();
        RoutePathInput
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.route_selection.route, "/api/users");
        assert_eq!(app.screen, Screen::RouteOptions);
    }

    #[test]
    fn submit_upload_route_goes_to_options() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Upload;
        RoutePathInput
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.screen, Screen::RouteOptions);
    }
}
