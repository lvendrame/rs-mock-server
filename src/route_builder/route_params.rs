use std::{ffi::{OsStr, OsString}, fs::DirEntry};

use crate::route_builder::config::{Config, ConfigStore};

#[derive(Debug, Clone)]
pub struct RouteParams {
    pub parent_route: String,
    pub full_route: String,
    pub file_name: String,
    pub file_stem: String,
    pub file_path: OsString,
    pub file_extension: String,
    pub config: Config,
    pub is_dir: bool,
}

impl RouteParams {
    pub fn new(parent_route: &str, entry: &DirEntry, config: Config, config_store: &ConfigStore) -> Self {
        let mut effective_config = config.clone();
        let parent_route = parent_route.to_string();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_stem = file_name.split('.').next().unwrap_or("").to_string();
        let file_extension = entry.path().extension().and_then(OsStr::to_str).unwrap_or_default().to_string();

        let is_dir = entry.file_type().unwrap().is_dir();


        let full_route = if is_dir {
            let config_store = ConfigStore::try_from_dir(entry.path().to_str().unwrap())
                .unwrap_or_else(|_| {
                    println!("Unable to read configs from folder {:?}", entry.path());
                    ConfigStore::default()
                });

            if file_name.starts_with("$") {
                effective_config = effective_config.with_protect(true);
            }
            if let Some(config) = config_store.get("config") {
                effective_config = config.merge_with_ref(&effective_config);
            }
            let end_point = file_name.replace("$", "");
            format!("{}/{}", parent_route, end_point)
        } else {
            if let Some(config) = config_store.get(&file_stem) {
                effective_config = config.merge_with_ref(&effective_config);
            }
            parent_route.clone()
        };

        let file_path = entry.path().into_os_string();

        Self {
            parent_route,
            full_route,
            file_name,
            file_path,
            file_stem,
            file_extension,
            config: effective_config,
            is_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, filename: &str) -> DirEntry {
        let file_path = dir.join(filename);
        File::create(&file_path).unwrap();

        let entries: Vec<DirEntry> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy() == filename)
            .collect();

        entries.into_iter().next().unwrap()
    }

    fn create_test_dir(parent_dir: &Path, dirname: &str) -> DirEntry {
        let dir_path = parent_dir.join(dirname);
        fs::create_dir(&dir_path).unwrap();

        let entries: Vec<DirEntry> = fs::read_dir(parent_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy() == dirname)
            .collect();

        entries.into_iter().next().unwrap()
    }

    #[test]
    fn test_new_with_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get.json");

        let params = RouteParams::new("/api/users", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api/users");
        assert_eq!(params.full_route, "/api/users");
        assert_eq!(params.file_name, "get.json");
        assert_eq!(params.file_stem, "get");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_protected_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "$get.json");

        let params = RouteParams::new("/api/users", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api/users");
        assert_eq!(params.full_route, "/api/users");
        assert_eq!(params.file_name, "$get.json");
        assert_eq!(params.file_stem, "$get");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true)); // Protection is determined by parent context
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_inherited_protection() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get.json");

        let params = RouteParams::new("/api/users", &entry, Config::default().with_protect(true), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api/users");
        assert_eq!(params.full_route, "/api/users");
        assert_eq!(params.file_name, "get.json");
        assert_eq!(params.file_stem, "get");
        assert!(params.config.route.unwrap_or_default().protect.unwrap_or(false)); // Inherited from parent
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_regular_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "products");

        let params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api");
        assert_eq!(params.full_route, "/api/products");
        assert_eq!(params.file_name, "products");
        assert_eq!(params.file_stem, "products");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(params.is_dir);
    }

    #[test]
    fn test_new_with_protected_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "$admin");

        let params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api");
        assert_eq!(params.full_route, "/api/admin"); // $ is stripped from route
        assert_eq!(params.file_name, "$admin");
        assert_eq!(params.file_stem, "$admin");
        assert!(params.config.route.unwrap_or_default().protect.unwrap_or(false));
        assert!(params.is_dir);
    }

    #[test]
    fn test_new_with_auth_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "{auth}.json");

        let params = RouteParams::new("/api/auth", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api/auth");
        assert_eq!(params.full_route, "/api/auth");
        assert_eq!(params.file_name, "{auth}.json");
        assert_eq!(params.file_stem, "{auth}");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_rest_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{_id:int}.json");

        let params = RouteParams::new("/api/products", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api/products");
        assert_eq!(params.full_route, "/api/products");
        assert_eq!(params.file_name, "rest{_id:int}.json");
        assert_eq!(params.file_stem, "rest{_id:int}");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_upload_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}");

        let params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api");
        assert_eq!(params.full_route, "/api/{upload}");
        assert_eq!(params.file_name, "{upload}");
        assert_eq!(params.file_stem, "{upload}");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(params.is_dir);
    }

    #[test]
    fn test_new_with_upload_temp_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}{temp}-docs");

        let params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api");
        assert_eq!(params.full_route, "/api/{upload}{temp}-docs");
        assert_eq!(params.file_name, "{upload}{temp}-docs");
        assert_eq!(params.file_stem, "{upload}{temp}-docs");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(params.is_dir);
    }

    #[test]
    fn test_new_with_empty_parent_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "index.json");

        let params = RouteParams::new("", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "");
        assert_eq!(params.full_route, "");
        assert_eq!(params.file_name, "index.json");
        assert_eq!(params.file_stem, "index");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_root_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "api");

        let params = RouteParams::new("", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "");
        assert_eq!(params.full_route, "/api");
        assert_eq!(params.file_name, "api");
        assert_eq!(params.file_stem, "api");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(params.is_dir);
    }

    #[test]
    fn test_new_with_complex_filename() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get{id}.json");

        let params = RouteParams::new("/api/users", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api/users");
        assert_eq!(params.full_route, "/api/users");
        assert_eq!(params.file_name, "get{id}.json");
        assert_eq!(params.file_stem, "get{id}");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_range_filename() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get{1-5}.json");

        let params = RouteParams::new("/api/products", &entry, Config::default().with_protect(false), &ConfigStore::default());

        assert_eq!(params.parent_route, "/api/products");
        assert_eq!(params.full_route, "/api/products");
        assert_eq!(params.file_name, "get{1-5}.json");
        assert_eq!(params.file_stem, "get{1-5}");
        assert!(!params.config.route.unwrap_or_default().protect.unwrap_or(true));
        assert!(!params.is_dir);
    }

    #[test]
    fn test_new_with_nested_protected_structure() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "$admin");

        let params = RouteParams::new("/api", &entry, Config::default().with_protect(true), &ConfigStore::default()); // Already protected parent

        assert_eq!(params.parent_route, "/api");
        assert_eq!(params.full_route, "/api/admin");
        assert_eq!(params.file_name, "$admin");
        assert_eq!(params.file_stem, "$admin");
        assert!(params.config.route.unwrap_or_default().protect.unwrap_or(false)); // Inherited protection
        assert!(params.is_dir);
    }

    #[test]
    fn test_file_stem_extraction() {
        let temp_dir = TempDir::new().unwrap();

        // Test various file extensions
        let test_cases = vec![
            ("test.json", "test"),
            ("test.txt", "test"),
            ("test.html", "test"),
            ("test", "test"),
            ("test.backup.json", "test"), // Only first part before dot
            ("{auth}.json", "{auth}"),
            ("$get.json", "$get"),
        ];

        for (filename, expected_stem) in test_cases {
            let entry = create_test_file(temp_dir.path(), filename);
            let params = RouteParams::new("/test", &entry, Config::default().with_protect(false), &ConfigStore::default());

            assert_eq!(params.file_stem, expected_stem, "Failed for filename: {}", filename);
        }
    }
}
