use std::{ffi::OsString};

use fosk::IdType;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{app::App, handlers::build_rest_routes, route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}};

static RE_FILE_REST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?rest(\{(.+)\})?$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_DESCRIPTOR: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct  RouteRest {
    pub path: OsString,
    pub route: String,
    pub id_key: String,
    pub id_type: IdType,
    pub is_protected: bool,
}

impl RouteRest {
    fn get_rest_options(descriptor: &str) -> (&str, IdType) {
        let parts: Vec<&str> = descriptor.split(':').collect();

        if parts.len() == 1 {
            // Single value like "uuid", "int", "id", "_id"
            let part = parts[0];
            match part {
                "none" => ("id", IdType::None),
                "uuid" => ("id", IdType::Uuid),
                "int" => ("id", IdType::Int),
                id_key => (id_key, IdType::Uuid), // Default fallback
            }
        } else if parts.len() == 2 {
            // Key:type format like "id:uuid", "_id:int"
            let id_key = parts[0];
            let type_str = parts[1];
            let id_type = match type_str {
                "none" => IdType::None,
                "uuid" => IdType::Uuid,
                "int" => IdType::Int,
                _ => IdType::Uuid, // Default to UUID
            };
            (id_key, id_type)
        } else {
            // Invalid format, return defaults
            ("id", IdType::Uuid)
        }
    }

    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_FILE_REST.captures(&route_params.file_stem) {
            let config = route_params.config.clone();
            let route_config = config.route.clone().unwrap_or_default();

            let is_protected = route_config.protect.unwrap_or(false);
            let is_protected = is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let descriptor = if let Some(pattern) = captures.get(ELEMENT_DESCRIPTOR) {
                pattern.as_str()
            } else {
                "id:uuid"
            };

            let (id_key, id_type) = Self::get_rest_options(descriptor);

            let route_rest = Self {
                path: route_params.file_path,
                route: route_config.remap.unwrap_or(route_params.full_route),
                id_key: id_key.to_string(),
                id_type,
                is_protected,
            };

            return Route::Rest(route_rest);
        }

        Route::None
    }
}

impl RouteGenerator for RouteRest {
    fn make_routes(&self, app: &mut App) {
        build_rest_routes(app,
            &self.route,
            &self.path,
             &self.id_key,
            self.id_type,
            self.is_protected,
            None
        );
    }
}

impl PrintRoute for RouteRest {
    fn println(&self) {
        println!("✔️ Built REST routes for {}", self.route);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_builder::config::{Config, ConfigStore};
    use crate::route_builder::route_params::RouteParams;
    use std::fs::File;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, filename: &str) -> std::fs::DirEntry {
        let file_path = dir.join(filename);
        File::create(&file_path).unwrap();
        let mut entries = dir.read_dir().unwrap();
        entries.find(|entry| {
            entry.as_ref().unwrap().file_name() == filename
        }).unwrap().unwrap()
    }

    #[test]
    fn test_try_parse_basic_rest_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest.json");
        let route_params = RouteParams::new("/api/users", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/users");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Uuid);
                assert!(!route_rest.is_protected);
                let expected_path = temp_dir.path().join("rest.json").into_os_string();
                assert_eq!(route_rest.path, expected_path);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_protected_rest_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "$rest.json");
        let route_params = RouteParams::new("/api/admin", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/admin");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Uuid);
                assert!(route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_rest_with_none_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{none}.json");
        let route_params = RouteParams::new("/api/products", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/products");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::None);
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_rest_with_uuid_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{uuid}.json");
        let route_params = RouteParams::new("/api/products", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/products");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Uuid);
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_rest_with_int_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{int}.json");
        let route_params = RouteParams::new("/api/orders", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/orders");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Int);
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_rest_with_custom_id_key() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{_id}.json");
        let route_params = RouteParams::new("/api/documents", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/documents");
                assert_eq!(route_rest.id_key, "_id");
                assert_eq!(route_rest.id_type, IdType::Uuid); // Default fallback
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_rest_with_id_key_and_type() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{_id:int}.json");
        let route_params = RouteParams::new("/api/items", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/items");
                assert_eq!(route_rest.id_key, "_id");
                assert_eq!(route_rest.id_type, IdType::Int);
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_rest_with_custom_key_uuid_type() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{user_id:uuid}.json");
        let route_params = RouteParams::new("/api/profiles", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/profiles");
                assert_eq!(route_rest.id_key, "user_id");
                assert_eq!(route_rest.id_type, IdType::Uuid);
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_protected_rest_with_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "$rest{id:int}.json");
        let route_params = RouteParams::new("/api/secure", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/secure");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Int);
                assert!(route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_inherited_protection() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest.json");
        let route_params = RouteParams::new("/api/admin", &entry, Config::default().with_protect(true), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/admin");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Uuid);
                assert!(route_rest.is_protected); // Inherited from parent
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_invalid_type_defaults_to_uuid() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{id:invalid}.json");
        let route_params = RouteParams::new("/api/test", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/test");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Uuid); // Should default to UUID
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_malformed_descriptor_uses_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{id:uuid:extra}.json");
        let route_params = RouteParams::new("/api/malformed", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/malformed");
                assert_eq!(route_rest.id_key, "id"); // Should use defaults
                assert_eq!(route_rest.id_type, IdType::Uuid);
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_non_rest_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "config.json");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::None => {
                // This is expected for non-rest files
            }
            _ => panic!("Expected Route::None for non-rest file"),
        }
    }

    #[test]
    fn test_try_parse_partial_rest_match() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "restaurant.json");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::None => {
                // Should not match "restaurant" as it doesn't exactly match "rest"
            }
            _ => panic!("Expected Route::None for partial rest match"),
        }
    }

    #[test]
    fn test_try_parse_complex_descriptor_formats() {
        let temp_dir = TempDir::new().unwrap();
        let test_cases = vec![
            ("rest{company_id:none}.json", "company_id", IdType::None),
            ("rest{product_id:int}.json", "product_id", IdType::Int),
            ("rest{order_uuid:uuid}.json", "order_uuid", IdType::Uuid),
            ("rest{user_pk:int}.json", "user_pk", IdType::Int),
            ("rest{document_id:uuid}.json", "document_id", IdType::Uuid),
        ];

        for (filename, expected_key, expected_type) in test_cases {
            let entry = create_test_file(temp_dir.path(), filename);
            let route_params = RouteParams::new("/api/complex", &entry, Config::default().with_protect(false), &ConfigStore::default());
            let result = RouteRest::try_parse(route_params);

            match result {
                Route::Rest(route_rest) => {
                    assert_eq!(route_rest.route, "/api/complex");
                    assert_eq!(route_rest.id_key, expected_key);
                    assert_eq!(route_rest.id_type, expected_type);
                    assert!(!route_rest.is_protected);
                }
                _ => panic!("Expected Route::Rest for {}", filename),
            }
        }
    }

    #[test]
    fn test_try_parse_nested_route_paths() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest{id:int}.json");
        let route_params = RouteParams::new("/api/v1/users/profile", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                assert_eq!(route_rest.route, "/api/v1/users/profile");
                assert_eq!(route_rest.id_key, "id");
                assert_eq!(route_rest.id_type, IdType::Int);
                assert!(!route_rest.is_protected);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_try_parse_file_path_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "rest.json");
        let route_params = RouteParams::new("/api/data", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteRest::try_parse(route_params);

        match result {
            Route::Rest(route_rest) => {
                let expected_path = temp_dir.path().join("rest.json").into_os_string();
                assert_eq!(route_rest.path, expected_path);
            }
            _ => panic!("Expected Route::Rest"),
        }
    }

    #[test]
    fn test_get_rest_options_single_values() {
        assert_eq!(RouteRest::get_rest_options("none"), ("id", IdType::None));
        assert_eq!(RouteRest::get_rest_options("uuid"), ("id", IdType::Uuid));
        assert_eq!(RouteRest::get_rest_options("int"), ("id", IdType::Int));
        assert_eq!(RouteRest::get_rest_options("_id"), ("_id", IdType::Uuid));
        assert_eq!(RouteRest::get_rest_options("user_id"), ("user_id", IdType::Uuid));
    }

    #[test]
    fn test_get_rest_options_key_type_pairs() {
        assert_eq!(RouteRest::get_rest_options("id:none"), ("id", IdType::None));
        assert_eq!(RouteRest::get_rest_options("id:uuid"), ("id", IdType::Uuid));
        assert_eq!(RouteRest::get_rest_options("id:int"), ("id", IdType::Int));
        assert_eq!(RouteRest::get_rest_options("_id:none"), ("_id", IdType::None));
        assert_eq!(RouteRest::get_rest_options("_id:uuid"), ("_id", IdType::Uuid));
        assert_eq!(RouteRest::get_rest_options("user_id:int"), ("user_id", IdType::Int));
    }

    #[test]
    fn test_get_rest_options_invalid_formats() {
        // Invalid type should default to UUID
        assert_eq!(RouteRest::get_rest_options("id:invalid"), ("id", IdType::Uuid));

        // Too many parts should use defaults
        assert_eq!(RouteRest::get_rest_options("id:uuid:extra"), ("id", IdType::Uuid));
    }
}
