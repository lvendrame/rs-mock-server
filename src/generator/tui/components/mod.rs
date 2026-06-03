//! Component registry for generator wizard screens.

use std::io;

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use crate::generator::tui::app::{Screen, WizardApp};

mod common;
mod field_edit;
mod field_editor;
mod file_type;
mod main_config;
mod review;
mod route_collection;
mod route_kind;
mod route_options;
mod route_path;
mod top_menu;

use common::WizardComponent;

pub fn enter_screen(app: &mut WizardApp, screen: Screen) {
    component(screen).enter(app);
}

pub fn handle_key(app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
    component(app.screen).handle_key(app, key)
}

pub fn render_screen(frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
    component(app.screen).render(frame, area, app);
}

pub fn render_header(frame: &mut Frame<'_>, area: Rect) {
    common::render_header(frame, area);
}

pub fn render_status(frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
    common::render_status(frame, area, app);
}

fn component(screen: Screen) -> &'static dyn WizardComponent {
    match screen {
        Screen::TopMenu => &top_menu::TopMenu,
        Screen::RouteKind => &route_kind::RouteKindMenu,
        Screen::RoutePath => &route_path::RoutePathInput,
        Screen::RouteOptions => &route_options::RouteOptions,
        Screen::RouteCollection => &route_collection::RouteCollectionInput,
        Screen::RouteFileType => &file_type::FileTypeMenu,
        Screen::FieldEditor => &field_editor::FieldEditor,
        Screen::FieldEdit => &field_edit::FieldEdit,
        Screen::MainConfig => &main_config::MainConfigInput,
        Screen::Review => &review::Review,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn enter_screen_dispatches_to_component_enter() {
        let mut app = WizardApp::new("mocks");
        app.menu_index = 3;
        enter_screen(&mut app, Screen::RouteKind);
        assert_eq!(app.menu_index, 0);
    }

    #[test]
    fn handle_key_dispatches_to_current_component() {
        let mut app = WizardApp::new("mocks");
        app.screen = Screen::TopMenu;
        handle_key(&mut app, key(KeyCode::Down)).unwrap();
        assert_eq!(app.menu_index, 1);
    }
}
