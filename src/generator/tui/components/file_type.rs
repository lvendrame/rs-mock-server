//! Route file type menu component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::generator::{
    domain::{GeneratedFileType, RouteKind},
    tui::{
        app::{self, WizardApp},
        components::common::{self, WizardComponent},
    },
};

pub struct FileTypeMenu;

impl WizardComponent for FileTypeMenu {
    fn enter(&self, app: &mut WizardApp) {
        app.menu_index = 0;
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        common::render_menu(frame, area, title(app), labels(app), app.menu_index);
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if common::quit_on_escape(app, key)
            || common::go_back(app, key)
            || common::move_menu(app, key, last_index(app))
        {
            return Ok(());
        }
        if matches!(key.code, KeyCode::Enter) {
            select(app);
        }
        Ok(())
    }
}

const BASIC_FILE_TYPES: &[&str] = &["JSON", "JGD", "Text"];
const DATA_FILE_TYPES: &[&str] = &["JSON", "JGD"];

fn last_index(app: &WizardApp) -> usize {
    labels(app).len() - 1
}

fn select(app: &mut WizardApp) {
    app.route_selection.file_type = file_type_from_component(app);
    app.transition_to(app::next_file_type_screen(app.route_selection.file_type));
}

fn file_type_from_component(app: &WizardApp) -> GeneratedFileType {
    match app.route_selection.kind {
        RouteKind::Basic => match app.menu_index {
            1 => GeneratedFileType::Jgd,
            2 => GeneratedFileType::Text,
            _ => GeneratedFileType::Json,
        },
        RouteKind::Rest | RouteKind::Graphql => match app.menu_index {
            1 => GeneratedFileType::Jgd,
            _ => GeneratedFileType::Json,
        },
        _ => app::file_type_from_index(app.menu_index),
    }
}

fn labels(app: &WizardApp) -> &'static [&'static str] {
    match app.route_selection.kind {
        RouteKind::Basic => BASIC_FILE_TYPES,
        RouteKind::Rest | RouteKind::Graphql => DATA_FILE_TYPES,
        _ => DATA_FILE_TYPES,
    }
}

fn title(app: &WizardApp) -> &'static str {
    match app.route_selection.kind {
        RouteKind::Rest => "REST data source",
        RouteKind::Graphql => "GraphQL seed source",
        _ => "Response type",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{
        domain::GeneratedFileType,
        tui::app::{Screen, WizardApp},
    };

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn enter_resets_menu_index() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 3;
        FileTypeMenu.enter(&mut app);
        assert_eq!(app.menu_index, 0);
    }

    #[test]
    fn selecting_jgd_opens_field_editor() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 1;
        FileTypeMenu
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.route_selection.file_type, GeneratedFileType::Jgd);
        assert_eq!(app.screen, Screen::FieldEditor);
    }

    #[test]
    fn selecting_text_opens_review() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 2;
        FileTypeMenu
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.route_selection.file_type, GeneratedFileType::Text);
        assert_eq!(app.screen, Screen::Review);
    }

    #[test]
    fn left_goes_back_to_route_options() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::RoutePath);
        app.transition_to(Screen::RouteOptions);
        app.transition_to(Screen::RouteFileType);
        FileTypeMenu
            .handle_key(&mut app, key(KeyCode::Left))
            .unwrap();
        assert_eq!(app.screen, Screen::RouteOptions);
    }
}
