use std::ffi::OsString;

use crate::{app::App, route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}};

#[derive(Debug, Clone, PartialEq)]
pub struct  RoutePublic {
    pub path: OsString,
    pub route: String,
    pub is_protected: bool,
}

static PUBLIC_ROUTE_NAME: &str = "public";

impl RoutePublic {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if route_params.file_stem == PUBLIC_ROUTE_NAME || route_params.file_stem.starts_with(&format!("{}-", PUBLIC_ROUTE_NAME)) {
            let public_route = if let Some((_, to)) = route_params.file_stem.split_once('-') {
                if to.is_empty() {
                    PUBLIC_ROUTE_NAME
                } else {
                    to
                }
            } else {
                PUBLIC_ROUTE_NAME
            };


            let route = format!("{}/{}", route_params.parent_route, public_route);

            let route_public = Self {
                path: route_params.file_path,
                route,
                is_protected: false,
            };

            return Route::Public(route_public);
        }

        Route::None
    }
}

impl RouteGenerator for RoutePublic {
    fn make_routes(&self, app: &mut App) {
        app.build_public_router_v2(&self.path, &self.route);
    }
}

impl PrintRoute for RoutePublic {
    fn println(&self) {
        println!("✔️ Built public routes from folder {} to {}", self.path.to_string_lossy(), self.route);
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

    fn create_test_dir(dir: &Path, dirname: &str) -> std::fs::DirEntry {
        let dir_path = dir.join(dirname);
        std::fs::create_dir(&dir_path).unwrap();
        let mut entries = dir.read_dir().unwrap();
        entries.find(|entry| {
            entry.as_ref().unwrap().file_name() == dirname
        }).unwrap().unwrap()
    }

    fn create_test_file(dir: &Path, filename: &str) -> std::fs::DirEntry {
        let file_path = dir.join(filename);
        File::create(&file_path).unwrap();
        let mut entries = dir.read_dir().unwrap();
        entries.find(|entry| {
            entry.as_ref().unwrap().file_name() == filename
        }).unwrap().unwrap()
    }

    #[test]
    fn test_try_parse_basic_public_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/api/public");
                assert!(!route_public.is_protected);
                let expected_path = temp_dir.path().join("public").into_os_string();
                assert_eq!(route_public.path, expected_path);
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_public_with_custom_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-static");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/api/static");
                assert!(!route_public.is_protected);
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_public_with_nested_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-assets");
        let route_params = RouteParams::new("/api/v1", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/api/v1/assets");
                assert!(!route_public.is_protected);
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_public_with_hyphenated_name() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-my-assets");
        let route_params = RouteParams::new("", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/my-assets");
                assert!(!route_public.is_protected);
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_public_file_instead_of_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "public.json");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/api/public");
                assert!(!route_public.is_protected);
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_public_with_empty_parent_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-images");
        let route_params = RouteParams::new("", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/images");
                assert!(!route_public.is_protected);
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_public_protection_always_false() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-secure");
        // Even when parent is protected, public routes should not be protected
        let route_params = RouteParams::new("/api/admin", &entry, Config::default().with_protect(true), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/api/admin/secure");
                assert!(!route_public.is_protected); // Should always be false
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_non_public_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "private");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::None => {
                // This is expected
            }
            _ => panic!("Expected Route::None for non-public directory"),
        }
    }

    #[test]
    fn test_try_parse_partial_public_match() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "publicity");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::None => {
                // This is expected - "publicity" should not match "public"
            }
            _ => panic!("Expected Route::None for publicity (should not match public)"),
        }
    }

    #[test]
    fn test_try_parse_public_with_multiple_hyphens() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-api-v1-docs");
        let route_params = RouteParams::new("", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                // Should only split on the first hyphen
                assert_eq!(route_public.route, "/api-v1-docs");
                assert!(!route_public.is_protected);
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_public_case_sensitive() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "Public");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::None => {
                // Should be case sensitive - "Public" != "public"
            }
            _ => panic!("Expected Route::None for case-sensitive mismatch"),
        }
    }

    #[test]
    fn test_try_parse_file_path_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-media");
        let route_params = RouteParams::new("/content", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                let expected_path = temp_dir.path().join("public-media").into_os_string();
                assert_eq!(route_public.path, expected_path);
                assert_eq!(route_public.route, "/content/media");
            }
            _ => panic!("Expected Route::Public"),
        }
    }

    #[test]
    fn test_try_parse_edge_case_public_only() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "public-");
        let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RoutePublic::try_parse(route_params);

        match result {
            Route::Public(route_public) => {
                assert_eq!(route_public.route, "/api/public");
                assert!(!route_public.is_protected);
            }
            _ => panic!("Expected Route::Public"),
        }
    }
}
