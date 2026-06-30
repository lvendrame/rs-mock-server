//! Generation review component.

use std::io;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect, text::Line, widgets::Paragraph};

use crate::generator::{
    domain::{GeneratorFlow, WriteOperation},
    tui::{
        app::WizardApp,
        components::common::{self, WizardComponent},
    },
};

pub struct Review;

impl WizardComponent for Review {
    fn enter(&self, app: &mut WizardApp) {
        build_plan(app);
        app.review_scroll = 0;
        app.status = "Review plan, then press Enter or w to write".to_string();
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
        render_review(frame, area, app);
    }

    fn handle_key(&self, app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
        if common::quit_on_escape(app, key) || common::go_back(app, key) {
            return Ok(());
        }
        handle_review_key(app, key)
    }
}

fn build_plan(app: &mut WizardApp) {
    match app.selected_flow {
        Some(GeneratorFlow::MainConfig) => app.build_config_review(),
        _ => app.build_route_review(),
    }
}

fn handle_review_key(app: &mut WizardApp, key: KeyEvent) -> io::Result<()> {
    match key.code {
        KeyCode::Up => app.move_review_scroll_up(),
        KeyCode::Down => app.move_review_scroll_down(),
        KeyCode::Char('f') => app.toggle_force(),
        KeyCode::Enter => app.write_plan()?,
        KeyCode::Char('w') => app.write_plan()?,
        _ => {}
    }
    Ok(())
}

fn render_review(frame: &mut Frame<'_>, area: Rect, app: &WizardApp) {
    let widget = Paragraph::new(lines(app))
        .block(common::block("Review"))
        .scroll((app.review_scroll, 0));
    frame.render_widget(widget, area);
}

pub(crate) fn lines(app: &WizardApp) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(format!("Force overwrite: {}", app.force))];
    lines.extend(app.plan.operations.iter().flat_map(operation_lines));
    lines
}

fn operation_lines(operation: &WriteOperation) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(operation_label(operation))];
    lines.extend(preview_lines(operation));
    lines
}

fn preview_lines(operation: &WriteOperation) -> Vec<Line<'static>> {
    operation.content.as_deref().map_or_else(Vec::new, preview)
}

fn preview(content: &str) -> Vec<Line<'static>> {
    content.lines().map(preview_line).collect()
}

fn operation_label(operation: &WriteOperation) -> String {
    format!("{} {}", operation_kind(operation), operation.path.display())
}

fn operation_kind(operation: &WriteOperation) -> &'static str {
    if operation.content.is_some() {
        "file"
    } else {
        "dir"
    }
}

fn preview_line(line: &str) -> Line<'static> {
    Line::from(format!("  {}", line))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::domain::{WriteOperation, WritePlan};

    #[test]
    fn review_lines_include_full_file_content() {
        let mut app = WizardApp::new("mocks");
        app.plan = WritePlan {
            operations: vec![WriteOperation::file("file.json", "one\ntwo\nthree")],
        };
        let text = lines(&app)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(text.contains(&"  one".to_string()));
        assert!(text.contains(&"  two".to_string()));
        assert!(text.contains(&"  three".to_string()));
    }

    #[test]
    fn enter_builds_config_plan_and_resets_scroll() {
        let mut app = WizardApp::new("mocks");
        app.selected_flow = Some(crate::generator::domain::GeneratorFlow::MainConfig);
        app.review_scroll = 5;
        Review.enter(&mut app);
        assert_eq!(app.review_scroll, 0);
        assert!(app.plan.operations[0].path.ends_with("rs-mock-server.toml"));
    }

    #[test]
    fn down_and_up_adjust_review_scroll() {
        let mut app = WizardApp::new("mocks");
        Review
            .handle_key(
                &mut app,
                KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE),
            )
            .unwrap();
        Review
            .handle_key(
                &mut app,
                KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE),
            )
            .unwrap();
        assert_eq!(app.review_scroll, 2);
        Review
            .handle_key(
                &mut app,
                KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE),
            )
            .unwrap();
        assert_eq!(app.review_scroll, 1);
    }

    #[test]
    fn force_key_toggles_force_status() {
        let mut app = WizardApp::new("mocks");
        Review
            .handle_key(
                &mut app,
                KeyEvent::new(KeyCode::Char('f'), crossterm::event::KeyModifiers::NONE),
            )
            .unwrap();
        assert!(app.force);
        assert_eq!(app.status, "Force overwrite: true");
    }
}
