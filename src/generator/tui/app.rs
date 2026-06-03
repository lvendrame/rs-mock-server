//! Shared wizard state and screen dispatch.

use std::io;

use crossterm::event::KeyEvent;

use crate::generator::{
    domain::{
        FieldKind, GeneratedFileType, GeneratorFlow, MainConfigSelection, RouteKind,
        RouteSelection, WritePlan,
    },
    main_config, paths,
    tui::components,
    writer,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    TopMenu,
    RouteKind,
    RoutePath,
    RouteOptions,
    RouteCollection,
    RouteFileType,
    FieldEditor,
    FieldEdit,
    MainConfig,
    Review,
}

#[derive(Debug, Default)]
pub struct WizardApp {
    pub screen: Screen,
    pub menu_index: usize,
    pub selected_flow: Option<GeneratorFlow>,
    pub route_selection: RouteSelection,
    pub main_config_selection: MainConfigSelection,
    pub input: String,
    pub plan: WritePlan,
    pub force: bool,
    pub field_index: usize,
    pub editing_field_index: Option<usize>,
    pub editing_field_name: String,
    pub editing_field_kind: FieldKind,
    pub review_scroll: u16,
    pub status: String,
    pub last_status: Option<String>,
    pub done: bool,
    folder: String,
}

impl WizardApp {
    pub fn new(folder: &str) -> Self {
        let mut app = Self::empty(folder);
        components::enter_screen(&mut app, Screen::TopMenu);
        app
    }

    fn empty(folder: &str) -> Self {
        Self {
            folder: folder.to_string(),
            ..Self::default()
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> io::Result<()> {
        components::handle_key(self, key)
    }

    pub(crate) fn build_route_review(&mut self) {
        self.plan = paths::build_route_plan(&self.folder, &self.route_selection);
    }

    pub(crate) fn build_config_review(&mut self) {
        self.main_config_selection.include_route_defaults = true;
        self.plan = main_config::build_main_config_plan(&self.main_config_selection);
    }

    pub(crate) fn transition_to(&mut self, screen: Screen) {
        self.screen = screen;
        components::enter_screen(self, screen);
    }

    pub(crate) fn go_back(&mut self) {
        if let Some(screen) = previous_screen(self) {
            self.transition_to(screen);
        }
    }

    pub(crate) fn start_field_edit(&mut self) {
        let index = self.field_index.min(self.route_selection.fields.len() - 1);
        let field = self.route_selection.fields[index].clone();
        self.editing_field_index = Some(index);
        self.editing_field_name = field.name;
        self.editing_field_kind = field.kind;
        self.transition_to(Screen::FieldEdit);
    }

    pub(crate) fn save_field_edit(&mut self) {
        let Some(index) = self.editing_field_index else {
            return;
        };
        let old_name = self.route_selection.fields[index].name.clone();
        let new_name = sanitized_field_name(&self.editing_field_name);
        self.route_selection.fields[index].name = new_name.clone();
        self.route_selection.fields[index].kind = self.editing_field_kind;
        self.update_id_field(&old_name, &new_name);
        self.transition_to(Screen::FieldEditor);
    }

    pub(crate) fn cancel_field_edit(&mut self) {
        self.editing_field_index = None;
        self.transition_to(Screen::FieldEditor);
    }

    pub(crate) fn toggle_force(&mut self) {
        self.force = !self.force;
        self.status = format!("Force overwrite: {}", self.force);
    }

    pub(crate) fn move_review_scroll_down(&mut self) {
        self.review_scroll = self.review_scroll.saturating_add(1);
    }

    pub(crate) fn move_review_scroll_up(&mut self) {
        self.review_scroll = self.review_scroll.saturating_sub(1);
    }

    pub(crate) fn write_plan(&mut self) -> io::Result<()> {
        writer::apply_plan(&self.plan, self.force)?;
        self.last_status = Some("Files generated".to_string());
        self.transition_to(Screen::TopMenu);
        Ok(())
    }

    pub(crate) fn update_route_from_input(&mut self) {
        if !self.input.trim().is_empty() {
            self.route_selection.route = normalize_route(&self.input);
        }
    }

    pub(crate) fn update_config_folder(&mut self) {
        if !self.input.trim().is_empty() {
            self.main_config_selection.folder = self.input.trim().to_string();
        }
    }

    pub(crate) fn finish(&mut self) {
        self.done = true;
    }

    fn update_id_field(&mut self, old_name: &str, new_name: &str) {
        if old_name == self.route_selection.id_key {
            self.route_selection.id_key = new_name.to_string();
            self.route_selection.id_type = id_type_for_field_kind(self.editing_field_kind);
        }
    }
}

pub fn route_kind_from_index(index: usize) -> RouteKind {
    match index {
        1 => RouteKind::Rest,
        2 => RouteKind::Auth,
        3 => RouteKind::Upload,
        4 => RouteKind::Public,
        5 => RouteKind::Graphql,
        6 => RouteKind::Sql,
        _ => RouteKind::Basic,
    }
}

pub fn file_type_from_index(index: usize) -> GeneratedFileType {
    match index {
        1 => GeneratedFileType::Jgd,
        2 => GeneratedFileType::Text,
        3 => GeneratedFileType::Sql,
        _ => GeneratedFileType::Json,
    }
}

pub fn normalize_route(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    }
}

pub(crate) fn next_route_screen(kind: RouteKind) -> Screen {
    if matches!(kind, RouteKind::Auth) {
        Screen::FieldEditor
    } else if matches!(kind, RouteKind::Public) {
        Screen::Review
    } else if matches!(kind, RouteKind::Sql) {
        Screen::RouteCollection
    } else {
        Screen::RouteOptions
    }
}

pub(crate) fn next_options_screen(kind: RouteKind) -> Screen {
    if needs_file_type(kind) {
        Screen::RouteFileType
    } else {
        Screen::Review
    }
}

pub(crate) fn cycle_field_kind(kind: FieldKind) -> FieldKind {
    match kind {
        FieldKind::String => FieldKind::Integer,
        FieldKind::Integer => FieldKind::Boolean,
        FieldKind::Boolean => FieldKind::DateTime,
        FieldKind::DateTime => FieldKind::Uuid,
        FieldKind::Uuid => FieldKind::String,
    }
}

pub(crate) fn id_type_for_field_kind(kind: FieldKind) -> crate::generator::domain::IdType {
    match kind {
        FieldKind::Integer => crate::generator::domain::IdType::Int,
        FieldKind::Uuid => crate::generator::domain::IdType::Uuid,
        _ => crate::generator::domain::IdType::None,
    }
}

pub(crate) fn sanitized_field_name(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        "field".to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn next_file_type_screen(file_type: GeneratedFileType) -> Screen {
    if matches!(file_type, GeneratedFileType::Text | GeneratedFileType::Sql) {
        Screen::Review
    } else {
        Screen::FieldEditor
    }
}

pub(crate) fn needs_file_type(kind: RouteKind) -> bool {
    matches!(
        kind,
        RouteKind::Basic | RouteKind::Rest | RouteKind::Graphql
    )
}

fn previous_screen(app: &WizardApp) -> Option<Screen> {
    match app.screen {
        Screen::TopMenu => None,
        Screen::RouteKind | Screen::MainConfig => Some(Screen::TopMenu),
        Screen::RoutePath => Some(Screen::RouteKind),
        Screen::RouteCollection => Some(Screen::RoutePath),
        Screen::RouteOptions => Some(previous_options_screen(app)),
        Screen::RouteFileType => Some(Screen::RouteOptions),
        Screen::FieldEditor => Some(previous_field_screen(app)),
        Screen::FieldEdit => Some(Screen::FieldEditor),
        Screen::Review => Some(previous_review_screen(app)),
    }
}

fn previous_field_screen(app: &WizardApp) -> Screen {
    if needs_file_type(app.route_selection.kind) {
        Screen::RouteFileType
    } else if matches!(app.route_selection.kind, RouteKind::Auth) {
        Screen::RoutePath
    } else {
        Screen::RouteOptions
    }
}

fn previous_review_screen(app: &WizardApp) -> Screen {
    match app.selected_flow {
        Some(GeneratorFlow::MainConfig) => Screen::MainConfig,
        _ => previous_route_review_screen(app),
    }
}

fn previous_route_review_screen(app: &WizardApp) -> Screen {
    if route_review_came_from_file_type(app) {
        Screen::RouteFileType
    } else if matches!(app.route_selection.kind, RouteKind::Public) {
        Screen::RoutePath
    } else if matches!(app.route_selection.kind, RouteKind::Auth) {
        Screen::FieldEditor
    } else if matches!(app.route_selection.kind, RouteKind::Upload | RouteKind::Sql) {
        Screen::RouteOptions
    } else {
        Screen::FieldEditor
    }
}

fn route_review_came_from_file_type(app: &WizardApp) -> bool {
    needs_file_type(app.route_selection.kind)
        && matches!(
            app.route_selection.file_type,
            GeneratedFileType::Text | GeneratedFileType::Sql
        )
}

fn previous_options_screen(app: &WizardApp) -> Screen {
    if matches!(app.route_selection.kind, RouteKind::Sql) {
        Screen::RouteCollection
    } else {
        Screen::RoutePath
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn route_kind_menu_maps_all_options() {
        assert_eq!(route_kind_from_index(0), RouteKind::Basic);
        assert_eq!(route_kind_from_index(6), RouteKind::Sql);
    }

    #[test]
    fn normalize_route_adds_leading_slash() {
        assert_eq!(normalize_route("api/users"), "/api/users");
        assert_eq!(normalize_route("/api/users"), "/api/users");
    }

    #[test]
    fn wizard_builds_route_write_plan() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.route = "/api/users/{id}".to_string();
        app.build_route_review();
        assert_eq!(
            app.plan.operations[0].path,
            std::path::PathBuf::from("mocks/api/users/get{id}.json")
        );
    }

    #[test]
    fn review_screen_sets_write_status() {
        let mut app = WizardApp::new("mocks");
        app.selected_flow = Some(GeneratorFlow::MainConfig);
        app.transition_to(Screen::Review);
        assert_eq!(app.status, "Review plan, then press Enter or w to write");
        assert_eq!(app.review_scroll, 0);
    }

    #[test]
    fn successful_write_returns_to_top_menu() {
        let temp = tempfile::tempdir().unwrap();
        let mut app = WizardApp::new(temp.path().to_str().unwrap());
        app.route_selection.route = "/api/users".to_string();
        app.build_route_review();
        app.write_plan().unwrap();
        assert_eq!(app.screen, Screen::TopMenu);
        assert_eq!(app.status, "Files generated");
    }

    #[test]
    fn route_flow_can_go_back_to_previous_steps() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::RouteKind);
        app.transition_to(Screen::RoutePath);
        app.go_back();
        assert_eq!(app.screen, Screen::RouteKind);
    }

    #[test]
    fn route_kinds_have_different_first_screens() {
        let cases = [
            (RouteKind::Basic, Screen::RouteOptions),
            (RouteKind::Rest, Screen::RouteOptions),
            (RouteKind::Auth, Screen::FieldEditor),
            (RouteKind::Upload, Screen::RouteOptions),
            (RouteKind::Public, Screen::Review),
            (RouteKind::Graphql, Screen::RouteOptions),
            (RouteKind::Sql, Screen::RouteCollection),
        ];

        for (kind, expected) in cases {
            assert_eq!(next_route_screen(kind), expected);
        }
    }

    #[test]
    fn route_options_open_kind_specific_next_screens() {
        let file_type_kinds = [RouteKind::Basic, RouteKind::Rest, RouteKind::Graphql];
        for kind in file_type_kinds {
            assert_eq!(next_options_screen(kind), Screen::RouteFileType);
        }
        assert_eq!(next_options_screen(RouteKind::Upload), Screen::Review);
        assert_eq!(next_options_screen(RouteKind::Sql), Screen::Review);
    }

    #[test]
    fn review_goes_back_to_route_specific_previous_screen() {
        let cases = [
            (RouteKind::Upload, Screen::RouteOptions),
            (RouteKind::Sql, Screen::RouteOptions),
            (RouteKind::Public, Screen::RoutePath),
            (RouteKind::Auth, Screen::FieldEditor),
        ];

        for (kind, expected) in cases {
            let mut app = WizardApp::new("mocks");
            app.route_selection.kind = kind;
            app.transition_to(Screen::Review);
            app.go_back();
            assert_eq!(app.screen, expected);
        }
    }

    #[test]
    fn review_goes_back_to_main_config_input() {
        let mut app = WizardApp::new("mocks");
        app.selected_flow = Some(GeneratorFlow::MainConfig);
        app.transition_to(Screen::Review);
        app.go_back();
        assert_eq!(app.screen, Screen::MainConfig);
    }

    #[test]
    fn endpoint_input_uses_left_arrow_to_go_back() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::RouteKind);
        app.transition_to(Screen::RoutePath);
        app.input = "/api/users".to_string();
        app.handle_key(key(KeyCode::Left)).unwrap();
        assert_eq!(app.screen, Screen::RouteKind);
        assert_eq!(app.route_selection.route, "/api/example");
    }

    #[test]
    fn endpoint_input_backspace_edits_until_empty_then_goes_back() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::RouteKind);
        app.transition_to(Screen::RoutePath);
        app.input = "x".to_string();
        app.handle_key(key(KeyCode::Backspace)).unwrap();
        assert_eq!(app.screen, Screen::RoutePath);
        assert_eq!(app.input, "");
        app.handle_key(key(KeyCode::Backspace)).unwrap();
        assert_eq!(app.screen, Screen::RouteKind);
    }

    #[test]
    fn field_editor_moves_selection_and_adds_selected_field() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::FieldEditor);
        app.handle_key(key(KeyCode::Down)).unwrap();
        assert_eq!(app.field_index, 1);
        app.handle_key(key(KeyCode::Char('a'))).unwrap();
        assert_eq!(app.field_index, app.route_selection.fields.len() - 1);
    }

    #[test]
    fn field_editor_cycles_selected_id_field_type() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::FieldEditor);
        app.handle_key(key(KeyCode::Char('i'))).unwrap();
        assert_eq!(app.route_selection.fields[0].kind, FieldKind::String);
        assert_eq!(
            app.route_selection.id_type,
            crate::generator::domain::IdType::None
        );
    }

    #[test]
    fn field_edit_saves_name_and_type() {
        let mut app = WizardApp::new("mocks");
        app.transition_to(Screen::FieldEditor);
        app.handle_key(key(KeyCode::Char('e'))).unwrap();
        app.input.clear();
        app.handle_key(key(KeyCode::Char('u'))).unwrap();
        app.handle_key(key(KeyCode::Char('s'))).unwrap();
        app.handle_key(key(KeyCode::Char('e'))).unwrap();
        app.handle_key(key(KeyCode::Char('r'))).unwrap();
        app.handle_key(key(KeyCode::Char('_'))).unwrap();
        app.handle_key(key(KeyCode::Char('i'))).unwrap();
        app.handle_key(key(KeyCode::Char('d'))).unwrap();
        app.handle_key(key(KeyCode::Tab)).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(app.screen, Screen::FieldEditor);
        assert_eq!(app.route_selection.fields[0].name, "user_id");
        assert_eq!(app.route_selection.id_key, "user_id");
        assert_eq!(app.route_selection.fields[0].kind, FieldKind::String);
    }

    #[test]
    fn added_field_is_in_generated_review_content() {
        let mut app = WizardApp::new("mocks");
        app.route_selection.route = "/api/users".to_string();
        app.route_selection
            .fields
            .push(crate::generator::domain::FieldSpec::new(
                "external_code",
                FieldKind::String,
            ));
        app.build_route_review();
        let content = app.plan.operations[0].content.as_ref().unwrap();
        assert!(content.contains("\"external_code\""));
    }

    #[test]
    fn review_screen_supports_scrolling() {
        let mut app = WizardApp::new("mocks");
        app.selected_flow = Some(GeneratorFlow::MainConfig);
        app.transition_to(Screen::Review);
        app.handle_key(key(KeyCode::Down)).unwrap();
        app.handle_key(key(KeyCode::Down)).unwrap();
        assert_eq!(app.review_scroll, 2);
        app.handle_key(key(KeyCode::Up)).unwrap();
        assert_eq!(app.review_scroll, 1);
    }
}
