//! Route field editor component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{List, ListItem},
};

use crate::generator::{
    domain::{FieldKind, FieldSpec},
    tui::{
        app::{self, Screen, WizardApp},
        components::common::{self, WizardComponent},
    },
};

pub struct FieldEditor;

impl WizardComponent for FieldEditor {
    fn enter(&self, app: &mut WizardApp) {
        app.status = "Edit fields or press Enter to review".to_string();
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        let fields = field_items(app);
        frame.render_widget(List::new(fields).block(common::block(title())), area);
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if common::quit_on_escape(app, key) || common::go_back(app, key) || move_selection(app, key)
        {
            return Ok(());
        }
        handle_field_key(app, key);
        Ok(())
    }
}

fn handle_field_key(app: &mut WizardApp, key: KeyEvent) {
    match key.code {
        KeyCode::Char('a') => add_field(app),
        KeyCode::Char('r') => remove_field(app),
        KeyCode::Char('e') => app.start_field_edit(),
        KeyCode::Char('i') => cycle_selected_field_type(app),
        KeyCode::Enter => app.transition_to(Screen::Review),
        _ => {}
    }
}

fn move_selection(app: &mut WizardApp, key: KeyEvent) -> bool {
    let max = app.route_selection.fields.len().saturating_sub(1);
    common::move_index(&mut app.field_index, key, max)
}

fn add_field(app: &mut WizardApp) {
    let next = app.route_selection.fields.len() + 1;
    app.route_selection.fields.push(new_field(next));
    app.field_index = app.route_selection.fields.len() - 1;
}

fn new_field(index: usize) -> FieldSpec {
    FieldSpec::new(format!("custom_field_{}", index), FieldKind::String)
}

fn remove_field(app: &mut WizardApp) {
    if app.route_selection.fields.len() > 1 {
        let removed = app.route_selection.fields[app.field_index].name.clone();
        app.route_selection.fields.remove(app.field_index);
        app.field_index = app.field_index.min(app.route_selection.fields.len() - 1);
        repair_id_after_remove(app, &removed);
    }
}

fn repair_id_after_remove(app: &mut WizardApp, removed: &str) {
    if removed != app.route_selection.id_key {
        return;
    }
    let field = &app.route_selection.fields[app.field_index];
    app.route_selection.id_key = field.name.clone();
    app.route_selection.id_type = app::id_type_for_field_kind(field.kind);
}

fn cycle_selected_field_type(app: &mut WizardApp) {
    let field = &mut app.route_selection.fields[app.field_index];
    field.kind = app::cycle_field_kind(field.kind);
    if field.name == app.route_selection.id_key {
        app.route_selection.id_type = app::id_type_for_field_kind(field.kind);
    }
}

fn field_items(app: &WizardApp) -> Vec<ListItem<'static>> {
    app.route_selection
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| field_item(field, index == app.field_index))
        .collect()
}

fn field_item(field: &FieldSpec, selected: bool) -> ListItem<'static> {
    ListItem::new(format!(
        "{}{}: {:?}",
        marker(selected),
        field.name,
        field.kind
    ))
}

fn marker(selected: bool) -> &'static str {
    if selected { "> " } else { "  " }
}

fn title() -> &'static str {
    "Fields: Up/Down select, e edit, a add, r remove, i type, Enter review"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        generator::domain::{FieldKind, IdType},
        generator::tui::app::Screen,
    };

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn enter_sets_field_editor_status() {
        let mut app = WizardApp::new("mocks");
        FieldEditor.enter(&mut app);
        assert_eq!(app.status, "Edit fields or press Enter to review");
    }

    #[test]
    fn down_moves_selected_field() {
        let mut app = WizardApp::new("mocks");
        FieldEditor
            .handle_key(&mut app, key(KeyCode::Down))
            .unwrap();
        assert_eq!(app.field_index, 1);
    }

    #[test]
    fn add_field_selects_new_field() {
        let mut app = WizardApp::new("mocks");
        let original_len = app.route_selection.fields.len();
        FieldEditor
            .handle_key(&mut app, key(KeyCode::Char('a')))
            .unwrap();
        assert_eq!(app.route_selection.fields.len(), original_len + 1);
        assert_eq!(app.field_index, original_len);
    }

    #[test]
    fn remove_selected_id_field_repairs_id_metadata() {
        let mut app = WizardApp::new("mocks");
        FieldEditor
            .handle_key(&mut app, key(KeyCode::Char('r')))
            .unwrap();
        assert_ne!(app.route_selection.id_key, "id");
        assert_eq!(app.route_selection.id_type, IdType::None);
    }

    #[test]
    fn cycle_selected_field_type_updates_id_metadata() {
        let mut app = WizardApp::new("mocks");
        FieldEditor
            .handle_key(&mut app, key(KeyCode::Char('i')))
            .unwrap();
        assert_eq!(app.route_selection.fields[0].kind, FieldKind::String);
        assert_eq!(app.route_selection.id_type, IdType::None);
    }

    #[test]
    fn edit_key_opens_field_edit_screen() {
        let mut app = WizardApp::new("mocks");
        FieldEditor
            .handle_key(&mut app, key(KeyCode::Char('e')))
            .unwrap();
        assert_eq!(app.screen, Screen::FieldEdit);
    }

    #[test]
    fn enter_opens_review_screen() {
        let mut app = WizardApp::new("mocks");
        FieldEditor
            .handle_key(&mut app, key(KeyCode::Enter))
            .unwrap();
        assert_eq!(app.screen, Screen::Review);
    }

    #[test]
    fn field_items_marks_selected_row() {
        let mut app = WizardApp::new("mocks");
        app.field_index = 1;
        let items = field_items(&app)
            .into_iter()
            .map(|item| format!("{item:?}"))
            .collect::<Vec<_>>();
        assert!(items[1].contains("> description"));
    }
}
