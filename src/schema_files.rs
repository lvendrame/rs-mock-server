//! Startup loading and serialization helpers for compact Fosk schema files.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use fosk::{Db, DbCollection, IdType, JsonPrimitive, SchemaDict};
use serde_json::{Map, Value};

use crate::{DEFAULT_SCHEMAS_DB_FILE, DEFAULT_SCHEMAS_FOLDER, route_builder::config::Config};

/// Effective schema loading configuration with defaults applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSchemasConfig {
    /// Folder containing schema files.
    pub folder: PathBuf,
    /// Full database schema file name.
    pub db_schema: String,
}

/// Resolves schema loading configuration against the configured mock root.
pub fn resolve_schemas_config(config: &Config) -> ResolvedSchemasConfig {
    let server = config.server.clone().unwrap_or_default();
    let schemas = config.schemas.clone().unwrap_or_default();
    let mock_root = server
        .folder
        .unwrap_or_else(|| crate::DEFAULT_FOLDER.to_string());
    let folder = schemas
        .folder
        .unwrap_or_else(|| DEFAULT_SCHEMAS_FOLDER.to_string());
    let folder_path = PathBuf::from(&folder);
    let folder = if folder_path.is_absolute() {
        folder_path
    } else {
        Path::new(&mock_root).join(folder_path)
    };

    ResolvedSchemasConfig {
        folder,
        db_schema: schemas
            .db_schema
            .unwrap_or_else(|| DEFAULT_SCHEMAS_DB_FILE.to_string()),
    }
}

/// Loads schema files from the configured schema folder, if it exists.
pub fn load_schema_files(db: &Arc<Db>, config: &Config) -> Result<Vec<String>, String> {
    let resolved = resolve_schemas_config(config);
    if !resolved.folder.exists() {
        return Ok(vec![]);
    }
    if !resolved.folder.is_dir() {
        return Err(format!(
            "Schema path {} is not a directory",
            resolved.folder.to_string_lossy()
        ));
    }

    let mut loaded = Vec::new();
    let db_schema_path = resolved.folder.join(&resolved.db_schema);
    if db_schema_path.is_file() {
        loaded.push(db.load_schemas_from_file(&path_to_os_string(&db_schema_path))?);
    }

    let mut entries = fs::read_dir(&resolved.folder)
        .map_err(|err| format!("Could not read schema folder: {err}"))?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.is_file()
            || path.file_name().and_then(|name| name.to_str()) == Some(&resolved.db_schema)
        {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            continue;
        }

        loaded.push(db.load_collection_schema_from_file(&path_to_os_string(&path))?);
    }

    Ok(loaded)
}

fn path_to_os_string(path: &Path) -> OsString {
    OsString::from(path.to_string_lossy().into_owned())
}

fn json_primitive_name(ty: &JsonPrimitive) -> &'static str {
    match ty {
        JsonPrimitive::Null => "Null",
        JsonPrimitive::Bool => "Bool",
        JsonPrimitive::Int => "Int",
        JsonPrimitive::Float => "Float",
        JsonPrimitive::String => "String",
        JsonPrimitive::Object => "Object",
        JsonPrimitive::Array => "Array",
    }
}

fn regular_type_spec(ty: &JsonPrimitive, nullable: bool) -> String {
    let name = json_primitive_name(ty);
    if nullable {
        name.to_string()
    } else {
        format!("{name}!")
    }
}

/// Serializes one collection's current schema to Fosk's compact schema format.
pub fn collection_schema_to_compact_json(collection: &DbCollection) -> Option<Value> {
    let schema = collection.schema()?;
    Some(schema_to_compact_json(&schema, collection))
}

fn schema_to_compact_json(schema: &SchemaDict, collection: &DbCollection) -> Value {
    let config = collection.get_config();
    let mut fields = Map::new();

    for (name, field_info) in &schema.fields {
        let type_spec = if name == &config.id_key {
            match config.id_type {
                IdType::Int => "Id".to_string(),
                IdType::Uuid => "Uuid".to_string(),
                IdType::None => format!("None:{}", json_primitive_name(&field_info.ty)),
            }
        } else {
            regular_type_spec(&field_info.ty, field_info.nullable)
        };
        fields.insert(name.clone(), Value::String(type_spec));
    }

    Value::Object(fields)
}

/// Serializes all loaded collection schemas to a reloadable compact DB schema.
pub fn db_schemas_to_compact_json(db: &Db) -> Value {
    let mut schemas = Map::new();
    for collection_name in db.list_collections() {
        let Some(collection) = db.get(&collection_name) else {
            continue;
        };
        let Some(schema) = collection_schema_to_compact_json(&collection) else {
            continue;
        };
        schemas.insert(collection_name, schema);
    }
    Value::Object(schemas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerConfig;
    use fosk::DbConfig;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn resolves_schema_defaults_under_mock_root() {
        let config = Config {
            server: Some(ServerConfig {
                folder: Some("mock-root".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let resolved = resolve_schemas_config(&config);
        assert_eq!(
            resolved.folder,
            PathBuf::from("mock-root").join("{schemas}")
        );
        assert_eq!(resolved.db_schema, "db.schema");
    }

    #[test]
    fn loads_db_schema_before_single_collection_files() {
        let temp_dir = TempDir::new().unwrap();
        let schemas = temp_dir.path().join("mocks").join("{schemas}");
        fs::create_dir_all(&schemas).unwrap();
        fs::write(
            schemas.join("db.schema"),
            json!({
                "users": { "user_id": "Id", "name": "String!" }
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            schemas.join("orders"),
            json!({ "order_id": "Uuid", "user_id": "Int!", "total": "Float!" }).to_string(),
        )
        .unwrap();

        let db = Db::new_arc();
        let config = Config {
            server: Some(ServerConfig {
                folder: Some(temp_dir.path().join("mocks").to_string_lossy().into_owned()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let loaded = load_schema_files(&db, &config).unwrap();

        assert_eq!(loaded.len(), 2);
        assert!(db.get("users").is_some());
        assert!(db.get("orders").is_some());
        assert!(db.get_collection_column_ref("orders", "user_id").is_some());
    }

    #[test]
    fn serializes_collection_schema_to_reloadable_compact_json() {
        let db = Db::new();
        db.create_with_config("users", DbConfig::int("user_id"))
            .load_schema_from_json(json!({
                "user_id": "Id",
                "name": "String!",
                "age": "Int"
            }))
            .unwrap();

        let collection = db.get("users").unwrap();
        let schema = collection_schema_to_compact_json(&collection).unwrap();

        assert_eq!(
            schema,
            json!({
                "user_id": "Id",
                "name": "String!",
                "age": "Int"
            })
        );
    }
}
