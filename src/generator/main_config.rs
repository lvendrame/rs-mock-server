//! Pure main configuration generation.

use toml::Value;
use toml::map::Map;

use crate::generator::domain::{MainConfigSelection, WriteOperation, WritePlan};

/// Builds a write plan for `rs-mock-server.toml`.
pub fn build_main_config_plan(selection: &MainConfigSelection) -> WritePlan {
    let mut plan = WritePlan::default();
    plan.push(WriteOperation::file(
        "rs-mock-server.toml",
        render_main_config(selection),
    ));
    plan
}

/// Renders `rs-mock-server.toml`.
pub fn render_main_config(selection: &MainConfigSelection) -> String {
    let mut root = Map::new();
    root.insert("server".to_string(), Value::Table(server_table(selection)));
    if selection.include_route_defaults {
        root.insert("route".to_string(), Value::Table(route_table(selection)));
    }
    toml::to_string_pretty(&Value::Table(root)).unwrap()
}

fn server_table(selection: &MainConfigSelection) -> Map<String, Value> {
    let mut server = Map::new();
    server.insert("port".to_string(), Value::Integer(selection.port.into()));
    server.insert(
        "folder".to_string(),
        Value::String(selection.folder.clone()),
    );
    server.insert(
        "enable_cors".to_string(),
        Value::Boolean(selection.enable_cors),
    );
    insert_string(&mut server, "allowed_origin", &selection.allowed_origin);
    server
}

fn route_table(selection: &MainConfigSelection) -> Map<String, Value> {
    let mut route = Map::new();
    insert_integer(&mut route, "delay", selection.delay);
    insert_string(&mut route, "remap", &selection.remap);
    insert_bool(&mut route, "protect", selection.protect);
    route
}

fn insert_string(table: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        table.insert(key.to_string(), Value::String(value.clone()));
    }
}

fn insert_integer(table: &mut Map<String, Value>, key: &str, value: Option<u16>) {
    if let Some(value) = value {
        table.insert(key.to_string(), Value::Integer(value.into()));
    }
}

fn insert_bool(table: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        table.insert(key.to_string(), Value::Boolean(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_server_config() {
        let content = render_main_config(&MainConfigSelection::default());
        assert!(content.contains("[server]"));
        assert!(content.contains("port = 4520"));
        assert!(content.contains("folder = \"mocks\""));
    }

    #[test]
    fn renders_optional_route_defaults() {
        let content = render_main_config(&MainConfigSelection {
            include_route_defaults: true,
            delay: Some(50),
            remap: Some("/api".to_string()),
            protect: Some(true),
            ..Default::default()
        });
        assert!(content.contains("[route]"));
        assert!(content.contains("delay = 50"));
        assert!(content.contains("protect = true"));
    }

    #[test]
    fn omits_route_table_when_route_defaults_are_disabled() {
        let content = render_main_config(&MainConfigSelection {
            delay: Some(50),
            remap: Some("/api".to_string()),
            protect: Some(true),
            ..Default::default()
        });
        assert!(!content.contains("[route]"));
        assert!(!content.contains("delay = 50"));
        assert!(!content.contains("remap ="));
    }

    #[test]
    fn renders_cors_and_allowed_origin_conflict_explicitly() {
        let content = render_main_config(&MainConfigSelection {
            enable_cors: false,
            allowed_origin: Some("https://example.test".to_string()),
            ..Default::default()
        });
        assert!(content.contains("enable_cors = false"));
        assert!(content.contains("allowed_origin = \"https://example.test\""));
    }

    #[test]
    fn renders_partial_route_defaults_without_empty_options() {
        let content = render_main_config(&MainConfigSelection {
            include_route_defaults: true,
            protect: Some(false),
            ..Default::default()
        });
        assert!(content.contains("[route]"));
        assert!(content.contains("protect = false"));
        assert!(!content.contains("delay ="));
        assert!(!content.contains("remap ="));
    }

    #[test]
    fn config_plan_targets_root_config_file() {
        let plan = build_main_config_plan(&MainConfigSelection::default());
        assert_eq!(plan.operations.len(), 1);
        assert_eq!(
            plan.operations[0].path,
            std::path::PathBuf::from("rs-mock-server.toml")
        );
        assert!(
            plan.operations[0]
                .content
                .as_ref()
                .unwrap()
                .contains("[server]")
        );
    }
}
