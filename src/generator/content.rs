//! Pure content generation for generated mock files.

use serde_json::{Map, Value, json};

use crate::generator::domain::{FieldKind, FieldSpec, IdType, RouteSelection};

/// Renders sample JSON object content.
pub fn render_json_object(fields: &[FieldSpec]) -> String {
    pretty_json(&Value::Object(sample_object(fields)))
}

/// Renders sample REST JSON array content.
pub fn render_json_array(fields: &[FieldSpec]) -> String {
    pretty_json(&Value::Array(vec![Value::Object(sample_object(fields))]))
}

/// Renders an auth user seed array.
pub fn render_auth_json(fields: &[FieldSpec]) -> String {
    let mut user = sample_object(fields);
    user.insert("username".to_string(), json!("admin"));
    user.insert("password".to_string(), json!("admin123"));
    user.insert("roles".to_string(), json!(["admin"]));
    pretty_json(&Value::Array(vec![Value::Object(user)]))
}

/// Renders a JGD document for the selected fields.
pub fn render_jgd(selection: &RouteSelection) -> String {
    let fields = selection
        .fields
        .iter()
        .map(|field| (field.name.clone(), jgd_value_for_field(field, selection)))
        .collect::<Map<String, Value>>();

    pretty_json(&json!({
        "$format": "jgd/v1",
        "version": "1.0.0",
        "root": {
            "count": 10,
            "fields": fields
        }
    }))
}

/// Renders a SQL query template for a collection.
pub fn render_sql(collection_name: &str, has_id_param: bool) -> String {
    if has_id_param {
        format!("select * from {} where id = ?;", collection_name)
    } else {
        format!("select * from {};", collection_name)
    }
}

/// Renders editable plain text sample content.
pub fn render_text(route: &str) -> String {
    format!("Mock response for {}", route)
}

fn sample_object(fields: &[FieldSpec]) -> Map<String, Value> {
    fields
        .iter()
        .map(|field| (field.name.clone(), sample_value_for_kind(field.kind)))
        .collect()
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap()
}

fn sample_value_for_kind(kind: FieldKind) -> Value {
    match kind {
        FieldKind::String => json!("sample value"),
        FieldKind::Integer => json!(1),
        FieldKind::Boolean => json!(true),
        FieldKind::DateTime => json!("2026-01-01T00:00:00Z"),
        FieldKind::Uuid => json!("550e8400-e29b-41d4-a716-446655440000"),
    }
}

fn jgd_value_for_field(field: &FieldSpec, selection: &RouteSelection) -> Value {
    if field.name == selection.id_key {
        return jgd_id_value(selection.id_type);
    }

    match field.kind {
        FieldKind::String => json!("${lorem.words(2,4)}"),
        FieldKind::Integer => json!({
            "number": {
                "min": 1,
                "max": 100,
                "integer": true
            }
        }),
        FieldKind::Boolean => json!("${boolean.boolean}"),
        FieldKind::DateTime => json!("${chrono.dateTime}"),
        FieldKind::Uuid => json!("${uuid.v4}"),
    }
}

fn jgd_id_value(id_type: IdType) -> Value {
    match id_type {
        IdType::None | IdType::Int => json!("${index}"),
        IdType::Uuid => json!("${uuid.v4}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::domain::{GeneratedFileType, RouteKind, default_fields};

    #[test]
    fn json_object_contains_default_fields() {
        let content = render_json_object(&default_fields());
        assert!(content.contains("\"description\""));
        assert!(content.contains("\"created_at\""));
    }

    #[test]
    fn rest_json_is_array() {
        let value: Value = serde_json::from_str(&render_json_array(&default_fields())).unwrap();
        assert!(value.as_array().unwrap()[0].get("updated_by").is_some());
    }

    #[test]
    fn auth_json_contains_required_auth_fields() {
        let content = render_auth_json(&default_fields());
        assert!(content.contains("\"username\""));
        assert!(content.contains("\"password\""));
        assert!(content.contains("\"roles\""));
    }

    #[test]
    fn jgd_uses_selected_id_type() {
        let selection = RouteSelection {
            kind: RouteKind::Rest,
            file_type: GeneratedFileType::Jgd,
            id_type: IdType::Int,
            ..Default::default()
        };
        let content = render_jgd(&selection);
        assert!(content.contains("\"$format\""));
        assert!(content.contains("\"${index}\""));
    }

    #[test]
    fn sql_template_can_bind_id() {
        assert_eq!(
            render_sql("users", true),
            "select * from users where id = ?;"
        );
    }

    #[test]
    fn json_object_renders_all_field_kinds() {
        let content = render_json_object(&[
            FieldSpec::new("name", FieldKind::String),
            FieldSpec::new("age", FieldKind::Integer),
            FieldSpec::new("active", FieldKind::Boolean),
            FieldSpec::new("created_at", FieldKind::DateTime),
            FieldSpec::new("external_id", FieldKind::Uuid),
        ]);
        let value: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(value["name"], json!("sample value"));
        assert_eq!(value["age"], json!(1));
        assert_eq!(value["active"], json!(true));
        assert_eq!(value["created_at"], json!("2026-01-01T00:00:00Z"));
        assert_eq!(
            value["external_id"],
            json!("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn json_object_duplicate_field_names_use_last_field_kind() {
        let content = render_json_object(&[
            FieldSpec::new("value", FieldKind::String),
            FieldSpec::new("value", FieldKind::Integer),
        ]);
        let value: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(value["value"], json!(1));
    }

    #[test]
    fn auth_json_overrides_conflicting_user_fields() {
        let content = render_auth_json(&[
            FieldSpec::new("username", FieldKind::Integer),
            FieldSpec::new("password", FieldKind::Boolean),
            FieldSpec::new("roles", FieldKind::String),
        ]);
        let value: Value = serde_json::from_str(&content).unwrap();
        let user = &value.as_array().unwrap()[0];
        assert_eq!(user["username"], json!("admin"));
        assert_eq!(user["password"], json!("admin123"));
        assert_eq!(user["roles"], json!(["admin"]));
    }

    #[test]
    fn jgd_renders_custom_id_key_and_non_id_field_generators() {
        let selection = RouteSelection {
            id_key: "user_id".to_string(),
            id_type: IdType::Uuid,
            fields: vec![
                FieldSpec::new("user_id", FieldKind::Uuid),
                FieldSpec::new("enabled", FieldKind::Boolean),
                FieldSpec::new("score", FieldKind::Integer),
            ],
            ..Default::default()
        };
        let value: Value = serde_json::from_str(&render_jgd(&selection)).unwrap();
        let fields = &value["root"]["fields"];
        assert_eq!(fields["user_id"], json!("${uuid.v4}"));
        assert_eq!(fields["enabled"], json!("${boolean.boolean}"));
        assert!(fields["score"]["number"]["integer"].as_bool().unwrap());
    }

    #[test]
    fn jgd_none_id_type_uses_index_generator() {
        let selection = RouteSelection {
            id_type: IdType::None,
            ..Default::default()
        };
        let value: Value = serde_json::from_str(&render_jgd(&selection)).unwrap();
        assert_eq!(value["root"]["fields"]["id"], json!("${index}"));
    }

    #[test]
    fn text_content_preserves_route_context() {
        assert_eq!(
            render_text("/api/users/{id}"),
            "Mock response for /api/users/{id}"
        );
    }

    #[test]
    fn sql_template_without_id_uses_collection_only() {
        assert_eq!(render_sql("orders", false), "select * from orders;");
    }
}
