//! Route-kind-specific option component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::generator::{
    domain::{IdType, RouteKind},
    tui::{
        app::{self, WizardApp},
        components::common,
    },
};

pub struct RouteOptions;

const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

impl common::WizardComponent for RouteOptions {
    fn enter(&self, app: &mut WizardApp) {
        app.menu_index = 0;
        app.status = status(app.route_selection.kind).to_string();
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        let rows = option_rows(app);
        let labels = rows.iter().map(String::as_str).collect::<Vec<_>>();
        common::render_menu(
            frame,
            area,
            title(app.route_selection.kind),
            &labels,
            app.menu_index,
        );
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if common::quit_on_escape(app, key) || common::go_back(app, key) {
            return Ok(());
        }
        if common::move_menu(app, key, last_index(app)) {
            return Ok(());
        }
        handle_action(app, key);
        Ok(())
    }
}

fn handle_action(app: &mut WizardApp, key: KeyEvent) {
    match key.code {
        KeyCode::Char(' ') => cycle_selected(app),
        KeyCode::Enter => app.transition_to(app::next_options_screen(app.route_selection.kind)),
        _ => {}
    }
}

fn cycle_selected(app: &mut WizardApp) {
    match (app.route_selection.kind, app.menu_index) {
        (RouteKind::Basic, 0) => cycle_method(app),
        (RouteKind::Rest, 0) => cycle_id_type(app),
        (RouteKind::Upload, 0) => {
            app.route_selection.temporary_upload = !app.route_selection.temporary_upload;
        }
        (_, _) => app.route_selection.protected = !app.route_selection.protected,
    }
}

fn option_rows(app: &WizardApp) -> Vec<String> {
    match app.route_selection.kind {
        RouteKind::Basic => basic_rows(app),
        RouteKind::Rest => rest_rows(app),
        RouteKind::Upload => upload_rows(app),
        RouteKind::Graphql => protected_rows("GraphQL", app.route_selection.protected),
        RouteKind::Sql => protected_rows("SQL file", app.route_selection.protected),
        _ => protected_rows("Route", app.route_selection.protected),
    }
}

fn basic_rows(app: &WizardApp) -> Vec<String> {
    vec![
        format!(
            "Method: {}",
            app.route_selection.method.to_ascii_uppercase()
        ),
        protected_label(app.route_selection.protected),
    ]
}

fn rest_rows(app: &WizardApp) -> Vec<String> {
    vec![
        format!(
            "ID strategy: {}",
            id_type_label(app.route_selection.id_type)
        ),
        protected_label(app.route_selection.protected),
    ]
}

fn upload_rows(app: &WizardApp) -> Vec<String> {
    vec![
        format!(
            "Storage: {}",
            upload_storage(app.route_selection.temporary_upload)
        ),
        protected_label(app.route_selection.protected),
    ]
}

fn protected_rows(label: &str, protected: bool) -> Vec<String> {
    vec![format!("{} access: {}", label, access_label(protected))]
}

fn cycle_method(app: &mut WizardApp) {
    let next = method_index(&app.route_selection.method) + 1;
    app.route_selection.method = METHODS[next % METHODS.len()].to_string();
}

fn cycle_id_type(app: &mut WizardApp) {
    app.route_selection.id_type = match app.route_selection.id_type {
        IdType::Uuid => IdType::Int,
        IdType::Int => IdType::None,
        IdType::None => IdType::Uuid,
    };
}

fn method_index(method: &str) -> usize {
    METHODS.iter().position(|item| *item == method).unwrap_or(0)
}

fn last_index(app: &WizardApp) -> usize {
    option_rows(app).len().saturating_sub(1)
}

fn title(kind: RouteKind) -> &'static str {
    match kind {
        RouteKind::Basic => "Basic route options",
        RouteKind::Rest => "REST collection options",
        RouteKind::Upload => "Upload route options",
        RouteKind::Graphql => "GraphQL service options",
        RouteKind::Sql => "SQL route options",
        _ => "Route options",
    }
}

fn status(kind: RouteKind) -> &'static str {
    match kind {
        RouteKind::Rest => "Space: cycle ID/protection | Enter: data source",
        RouteKind::Upload => "Space: toggle storage/protection | Enter: review",
        RouteKind::Sql => "Space: toggle protection | Enter: review",
        _ => "Space: change selected option | Enter: next",
    }
}

fn id_type_label(id_type: IdType) -> &'static str {
    match id_type {
        IdType::Uuid => "uuid id",
        IdType::Int => "integer id",
        IdType::None => "no generated id",
    }
}

fn upload_storage(temporary: bool) -> &'static str {
    if temporary { "temporary" } else { "persistent" }
}

fn protected_label(protected: bool) -> String {
    format!("Access: {}", access_label(protected))
}

fn access_label(protected: bool) -> &'static str {
    if protected { "protected" } else { "public" }
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
    fn basic_space_cycles_method_before_protection() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Basic;
        RouteOptions.enter(&mut app);
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Char(' ')))
            .unwrap();
        assert_eq!(app.route_selection.method, "post");
    }

    #[test]
    fn basic_second_row_toggles_protection() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Basic;
        app.menu_index = 1;
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Char(' ')))
            .unwrap();
        assert!(app.route_selection.protected);
    }

    #[test]
    fn rest_space_cycles_id_type() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Rest;
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Char(' ')))
            .unwrap();
        assert_eq!(app.route_selection.id_type, IdType::Int);
    }

    #[test]
    fn upload_first_row_toggles_temporary_only() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Upload;
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Char(' ')))
            .unwrap();
        assert!(app.route_selection.temporary_upload);
        assert!(!app.route_selection.protected);
    }

    #[test]
    fn upload_second_row_toggles_protection() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Upload;
        app.menu_index = 1;
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Char(' ')))
            .unwrap();
        assert!(app.route_selection.protected);
    }

    #[test]
    fn graphql_enter_opens_file_type() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Graphql;
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.screen, app::Screen::RouteFileType);
    }

    #[test]
    fn sql_enter_opens_review() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Sql;
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.screen, app::Screen::Review);
    }

    #[test]
    fn left_returns_to_previous_screen() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.kind = RouteKind::Sql;
        app.transition_to(app::Screen::RouteCollection);
        app.transition_to(app::Screen::RouteOptions);
        RouteOptions
            .handle_key(&mut app, key(KeyCode::Left))
            .unwrap();
        assert_eq!(app.screen, app::Screen::RouteCollection);
    }
}
