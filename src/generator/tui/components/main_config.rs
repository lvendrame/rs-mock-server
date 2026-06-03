//! Main configuration input component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

use crate::generator::tui::{
    app::{Screen, WizardApp},
    components::common::{self, WizardComponent},
};

pub struct MainConfigInput;

impl WizardComponent for MainConfigInput {
    fn enter(&self, app: &mut WizardApp) {
        app.input = app.main_config_selection.folder.clone();
        app.status = "Edit mock folder, then press Enter".to_string();
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        common::render_input(frame, area, "Mock folder", &app.input);
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
    app.update_config_folder();
    app.transition_to(Screen::Review);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::tui::app::WizardApp;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn enter_loads_current_folder() {
        let mut app = WizardApp::new("mocks");
        app.main_config_selection.folder = "fixtures".to_string();
        MainConfigInput.enter(&mut app);
        assert_eq!(app.input, "fixtures");
    }

    #[test]
    fn printable_key_edits_folder_input() {
        let mut app = WizardApp::new("mocks");
        app.input = "mock".to_string();
        MainConfigInput
            .handle_key(&mut app, key(KeyCode::Char('s')))
            .unwrap();
        assert_eq!(app.input, "mocks");
    }

    #[test]
    fn submit_updates_folder_and_opens_review() {
        let mut app = WizardApp::new("mocks");
        app.input = "fixtures".to_string();
        MainConfigInput
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.main_config_selection.folder, "fixtures");
        assert_eq!(app.screen, Screen::Review);
    }

    #[test]
    fn left_returns_to_top_menu() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::MainConfig);
        MainConfigInput
            .handle_key(&mut app, key(KeyCode::Left))
            .unwrap();
        assert_eq!(app.screen, Screen::TopMenu);
    }
}
