//! Field edit dialog component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect, text::Line};

use crate::generator::tui::{
    app::{self, WizardApp},
    components::common::{self, WizardComponent},
};

pub struct FieldEdit;

impl WizardComponent for FieldEdit {
    fn enter(&self, app: &mut WizardApp) {
        app.input = app.editing_field_name.clone();
        app.status = "Edit field name, Tab type, Enter save, Esc cancel".to_string();
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        common::render_lines(frame, area, "Edit field", lines(app));
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if cancel(app, key) || handle_field_edit_key(app, key) {
            return Ok(());
        }
        common::edit_input(app, key);
        app.editing_field_name = app.input.clone();
        Ok(())
    }
}

fn cancel(app: &mut WizardApp, key: KeyEvent) -> bool {
    if matches!(key.code, KeyCode::Esc | KeyCode::Left) {
        app.cancel_field_edit();
        return true;
    }
    false
}

fn handle_field_edit_key(app: &mut WizardApp, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => save(app),
        KeyCode::Tab => cycle_kind(app),
        _ => return false,
    }
    true
}

fn save(app: &mut WizardApp) {
    app.editing_field_name = app.input.clone();
    app.save_field_edit();
}

fn cycle_kind(app: &mut WizardApp) {
    app.editing_field_kind = app::cycle_field_kind(app.editing_field_kind);
}

fn lines(app: &WizardApp) -> Vec<Line<'static>> {
    vec![
        Line::from(format!("Name: {}", app.input)),
        Line::from(format!("Type: {:?}", app.editing_field_kind)),
        Line::from(format!("ID field: {}", is_id_field(app))),
    ]
}

fn is_id_field(app: &WizardApp) -> bool {
    app.editing_field_index
        .and_then(|index| app.route_selection.fields.get(index))
        .is_some_and(|field| field.name == app.route_selection.id_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{
        domain::FieldKind,
        tui::app::{Screen, WizardApp},
    };

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn enter_loads_field_name_into_input() {
        let mut app = WizardApp::new("mocks");
        app.editing_field_name = "description".to_string();
        FieldEdit.enter(&mut app);
        assert_eq!(app.input, "description");
        assert!(app.status.contains("Edit field name"));
    }

    #[test]
    fn tab_cycles_field_kind_without_editing_name() {
        let mut app = WizardApp::new("mocks");
        app.editing_field_kind = FieldKind::String;
        app.input = "name".to_string();
        FieldEdit.handle_key(&mut app, key(KeyCode::Tab)).unwrap();
        assert_eq!(app.input, "name");
        assert_eq!(app.editing_field_kind, FieldKind::Integer);
    }

    #[test]
    fn printable_char_updates_editing_name() {
        let mut app = WizardApp::new("mocks");
        app.input = "name".to_string();
        FieldEdit
            .handle_key(&mut app, key(KeyCode::Char('2')))
            .unwrap();
        assert_eq!(app.editing_field_name, "name2");
    }

    #[test]
    fn escape_cancels_to_field_editor() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::FieldEdit);
        FieldEdit.handle_key(&mut app, key(KeyCode::Esc)).unwrap();
        assert_eq!(app.screen, Screen::FieldEditor);
    }

    #[test]
    fn lines_marks_current_id_field() {
        let mut app = WizardApp::new("mocks");
        app.editing_field_index = Some(0);
        app.input = "id".to_string();
        let text = lines(&app)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(text.contains(&"ID field: true".to_string()));
    }
}
