//! Startup loading helpers for Fosk collection seed files.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use fosk::Db;
use jgd_rs::generate_jgd_from_file;

use crate::{
    DEFAULT_COLLECTIONS_FOLDER,
    handlers::{is_jgd, is_json},
    route_builder::config::Config,
};

/// Effective collection loading configuration with defaults applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCollectionsConfig {
    /// Folder containing collection seed files.
    pub folder: PathBuf,
}

/// Resolves collection loading configuration against the configured mock root.
pub fn resolve_collections_config(config: &Config) -> ResolvedCollectionsConfig {
    let server = config.server.clone().unwrap_or_default();
    let collections = config.collections.clone().unwrap_or_default();
    let mock_root = server
        .folder
        .unwrap_or_else(|| crate::DEFAULT_FOLDER.to_string());
    let folder = collections
        .folder
        .unwrap_or_else(|| DEFAULT_COLLECTIONS_FOLDER.to_string());
    let folder_path = PathBuf::from(&folder);
    let folder = if folder_path.is_absolute() {
        folder_path
    } else {
        Path::new(&mock_root).join(folder_path)
    };

    ResolvedCollectionsConfig { folder }
}

/// Loads collection seed files from the configured collection folder, if it exists.
pub fn load_collection_files(db: &Arc<Db>, config: &Config) -> Result<Vec<String>, String> {
    let resolved = resolve_collections_config(config);
    if !resolved.folder.exists() {
        return Ok(vec![]);
    }
    if !resolved.folder.is_dir() {
        return Err(format!(
            "Collection path {} is not a directory",
            resolved.folder.to_string_lossy()
        ));
    }

    let mut entries = fs::read_dir(&resolved.folder)
        .map_err(|err| format!("Could not read collection folder: {err}"))?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    let mut loaded = Vec::new();
    for entry in entries {
        let path = entry.path();
        if !path.is_file() || !(is_jgd(&entry.file_name()) || is_json(&entry.file_name())) {
            continue;
        }

        loaded.push(load_collection_file(db, &path)?);
    }

    Ok(loaded)
}

fn load_collection_file(db: &Arc<Db>, path: &Path) -> Result<String, String> {
    let collection_name = collection_name_from_path(path)?;
    let collection = db.create(&collection_name);

    if is_jgd(&path_to_os_string(path)) {
        let jgd_json = generate_jgd_from_file(&path.to_path_buf()).map_err(|error| {
            format!(
                "Error to generate JGD JSON for file {}. Details: {}",
                path.to_string_lossy(),
                error
            )
        })?;
        let items = collection
            .load_from_json(jgd_json, false)
            .map_err(|error| {
                format!(
                    "Error to load JSON for file {}. Details: {}",
                    path.to_string_lossy(),
                    error
                )
            })?;
        return Ok(format!(
            "✔️ Loaded collection {} with {} initial items from {}",
            collection_name,
            items.len(),
            path.to_string_lossy()
        ));
    }

    collection
        .load_from_file(&path_to_os_string(path))
        .map_err(|error| error.to_string())
}

fn collection_name_from_path(path: &Path) -> Result<String, String> {
    path.file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("Invalid collection file name: {}", path.to_string_lossy()))
}

fn path_to_os_string(path: &Path) -> OsString {
    OsString::from(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ServerConfig,
        route_builder::config::{CollectionsConfig, Config},
    };
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn resolves_collection_defaults_under_mock_root() {
        let config = Config {
            server: Some(ServerConfig {
                folder: Some("mock-root".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let resolved = resolve_collections_config(&config);
        assert_eq!(
            resolved.folder,
            PathBuf::from("mock-root").join("{collections}")
        );
    }

    #[test]
    fn resolves_configured_collection_folder_under_mock_root() {
        let config = Config {
            server: Some(ServerConfig {
                folder: Some("mock-root".to_string()),
                ..Default::default()
            }),
            collections: Some(CollectionsConfig {
                folder: Some("seed-data".to_string()),
            }),
            ..Default::default()
        };

        let resolved = resolve_collections_config(&config);
        assert_eq!(
            resolved.folder,
            PathBuf::from("mock-root").join("seed-data")
        );
    }

    #[test]
    fn loads_json_and_jgd_collection_files() {
        let temp_dir = TempDir::new().unwrap();
        let collections = temp_dir.path().join("mocks").join("{collections}");
        fs::create_dir_all(&collections).unwrap();
        fs::write(
            collections.join("warehouse_locations.json"),
            json!([
                { "id": "loc-1", "name": "Lisbon" },
                { "id": "loc-2", "name": "Porto" }
            ])
            .to_string(),
        )
        .unwrap();
        fs::write(
            collections.join("warehouse_assets.jgd"),
            json!({
                "$format": "jgd/v1",
                "version": "1.1",
                "root": {
                    "count": 3,
                    "fields": {
                        "id": "${uuid.v4}",
                        "serial": "${number.numberWithFormat(AST-####)}",
                        "status": "${lorem.word}"
                    }
                }
            })
            .to_string(),
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
        let loaded = load_collection_files(&db, &config).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(db.get("warehouse_locations").unwrap().count().unwrap(), 2);
        assert_eq!(db.get("warehouse_assets").unwrap().count().unwrap(), 3);
    }

    #[test]
    fn ignores_unsupported_collection_files() {
        let temp_dir = TempDir::new().unwrap();
        let collections = temp_dir.path().join("mocks").join("{collections}");
        fs::create_dir_all(&collections).unwrap();
        fs::write(collections.join("notes.txt"), "ignore me").unwrap();

        let db = Db::new_arc();
        let config = Config {
            server: Some(ServerConfig {
                folder: Some(temp_dir.path().join("mocks").to_string_lossy().into_owned()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let loaded = load_collection_files(&db, &config).unwrap();

        assert!(loaded.is_empty());
        assert!(db.list_collections().is_empty());
    }

    #[test]
    fn rejects_collection_path_that_is_not_a_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mocks = temp_dir.path().join("mocks");
        fs::create_dir_all(&mocks).unwrap();
        fs::write(mocks.join("{collections}"), "not a folder").unwrap();

        let db = Db::new_arc();
        let config = Config {
            server: Some(ServerConfig {
                folder: Some(mocks.to_string_lossy().into_owned()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let error = load_collection_files(&db, &config).unwrap_err();

        assert!(error.contains("is not a directory"));
    }
}
