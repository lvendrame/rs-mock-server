use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{handlers::build_upload_routes, route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}};

static RE_DIR_UPLOAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?\{upload\}(\{temp\})?(-(.+))?$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_IS_TEMPORARY: usize = 2;
const ELEMENT_ROUTE: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct  RouteUpload {
    pub path: OsString,
    pub route: String,
    pub is_temporary: bool,
    pub is_protected: bool,
}

impl RouteUpload {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_DIR_UPLOAD.captures(&route_params.file_name) {
            let is_protected = route_params.is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let is_temporary = captures.get(ELEMENT_IS_TEMPORARY).is_some();
            let uploads_route = if let Some(route) = captures.get(ELEMENT_ROUTE) {
                route.as_str()
            } else {
                "upload"
            };

            let route = format!("{}/{}", route_params.parent_route, uploads_route);

            let route_upload = Self {
                path: route_params.file_path,
                route: route.to_string(),
                is_temporary,
                is_protected,
            };

            return Route::Upload(route_upload);
        }

        Route::None
    }
}

impl RouteGenerator for RouteUpload {
    fn make_routes(&self, app: &mut crate::app::App) {
        let path = self.path.to_string_lossy();
        app.push_uploads_config(path.to_string(), self.is_temporary);

        build_upload_routes(app, path.to_string(), &self.route);
    }
}

impl PrintRoute for RouteUpload {
    fn println(&self) {
        println!("✔️ Mapped uploads from folder {} to {}", self.path.to_string_lossy(), self.route);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_builder::route_params::RouteParams;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_dir(dir: &Path, dirname: &str) -> std::fs::DirEntry {
        let dir_path = dir.join(dirname);
        fs::create_dir(&dir_path).unwrap();
        let mut entries = dir.read_dir().unwrap();
        entries.find(|entry| {
            entry.as_ref().unwrap().file_name() == dirname
        }).unwrap().unwrap()
    }

    #[test]
    fn test_try_parse_basic_upload_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/upload");
                assert!(!route_upload.is_temporary);
                assert!(!route_upload.is_protected);
                let expected_path = temp_dir.path().join("{upload}").into_os_string();
                assert_eq!(route_upload.path, expected_path);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_protected_upload_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "${upload}");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/upload");
                assert!(!route_upload.is_temporary);
                assert!(route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_temporary_upload_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}{temp}");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/upload");
                assert!(route_upload.is_temporary);
                assert!(!route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_protected_temporary_upload() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "${upload}{temp}");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/upload");
                assert!(route_upload.is_temporary);
                assert!(route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_upload_with_custom_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}-files");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/files");
                assert!(!route_upload.is_temporary);
                assert!(!route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_temporary_upload_with_custom_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}{temp}-media");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/media");
                assert!(route_upload.is_temporary);
                assert!(!route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_protected_upload_with_custom_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "${upload}-documents");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/documents");
                assert!(!route_upload.is_temporary);
                assert!(route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_full_featured_upload() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "${upload}{temp}-assets");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/assets");
                assert!(route_upload.is_temporary);
                assert!(route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_inherited_protection() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}");
        let route_params = RouteParams::new("/api/admin", &entry, true);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/admin/upload");
                assert!(!route_upload.is_temporary);
                assert!(route_upload.is_protected); // Inherited from parent
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_nested_route_paths() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}-images");
        let route_params = RouteParams::new("/api/v1/content", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/api/v1/content/images");
                assert!(!route_upload.is_temporary);
                assert!(!route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_upload_with_empty_parent_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}-data");
        let route_params = RouteParams::new("", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                assert_eq!(route_upload.route, "/data");
                assert!(!route_upload.is_temporary);
                assert!(!route_upload.is_protected);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_complex_custom_route_names() {
        let temp_dir = TempDir::new().unwrap();
        let test_cases = vec![
            ("{upload}-user-avatars", "/api/user-avatars"),
            ("{upload}-file_storage", "/api/file_storage"),
            ("{upload}-temp_files", "/api/temp_files"),
            ("{upload}-backup-data", "/api/backup-data"),
        ];

        for (dirname, expected_route) in test_cases {
            let entry = create_test_dir(temp_dir.path(), dirname);
            let route_params = RouteParams::new("/api", &entry, false);
            let result = RouteUpload::try_parse(route_params);

            match result {
                Route::Upload(route_upload) => {
                    assert_eq!(route_upload.route, expected_route);
                    assert!(!route_upload.is_temporary);
                    assert!(!route_upload.is_protected);
                }
                _ => panic!("Expected Route::Upload for {}", dirname),
            }
        }
    }

    #[test]
    fn test_try_parse_non_upload_directory() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "regular_folder");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::None => {
                // This is expected for non-upload directories
            }
            _ => panic!("Expected Route::None for non-upload directory"),
        }
    }

    #[test]
    fn test_try_parse_partial_upload_match() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "upload_folder");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::None => {
                // Should not match "upload_folder" as it doesn't match the pattern
            }
            _ => panic!("Expected Route::None for partial upload match"),
        }
    }

    #[test]
    fn test_try_parse_malformed_upload_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let malformed_patterns = vec![
            "upload{temp}",     // Missing braces around upload
            "{uploads}",        // Wrong name (uploads vs upload)
            "{upload{temp}}",   // Nested braces
            "upload-files",     // No braces at all
        ];

        for pattern in malformed_patterns {
            let entry = create_test_dir(temp_dir.path(), pattern);
            let route_params = RouteParams::new("/api", &entry, false);
            let result = RouteUpload::try_parse(route_params);

            match result {
                Route::None => {
                    // Expected for malformed patterns
                }
                _ => panic!("Expected Route::None for malformed pattern: {}", pattern),
            }
        }
    }

    #[test]
    fn test_try_parse_file_path_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}{temp}-cache");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::Upload(route_upload) => {
                let expected_path = temp_dir.path().join("{upload}{temp}-cache").into_os_string();
                assert_eq!(route_upload.path, expected_path);
                assert_eq!(route_upload.route, "/api/cache");
                assert!(route_upload.is_temporary);
            }
            _ => panic!("Expected Route::Upload"),
        }
    }

    #[test]
    fn test_try_parse_regex_pattern_validation() {
        let temp_dir = TempDir::new().unwrap();

        // Valid patterns that should match
        let valid_patterns = vec![
            "{upload}",
            "${upload}",
            "{upload}{temp}",
            "${upload}{temp}",
            "{upload}-files",
            "${upload}-secure",
            "{upload}{temp}-temp",
            "${upload}{temp}-secure-temp",
        ];

        for pattern in valid_patterns {
            let entry = create_test_dir(temp_dir.path(), pattern);
            let route_params = RouteParams::new("/test", &entry, false);
            let result = RouteUpload::try_parse(route_params);

            match result {
                Route::Upload(_) => {
                    // Expected for valid patterns
                }
                _ => panic!("Expected Route::Upload for valid pattern: {}", pattern),
            }
        }
    }

    #[test]
    fn test_try_parse_edge_case_empty_custom_route() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_dir(temp_dir.path(), "{upload}-");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteUpload::try_parse(route_params);

        match result {
            Route::None => {
                // The regex pattern requires at least one character after the hyphen
                // So "{upload}-" (with empty string after hyphen) should not match
            }
            _ => panic!("Expected Route::None for empty custom route"),
        }
    }

    #[test]
    fn test_try_parse_combination_matrix() {
        let temp_dir = TempDir::new().unwrap();

        // Test all combinations of protection, temporary, and custom routes
        let combinations = vec![
            ("{upload}", false, false, "/api/upload"),
            ("${upload}", true, false, "/api/upload"),
            ("{upload}{temp}", false, true, "/api/upload"),
            ("${upload}{temp}", true, true, "/api/upload"),
            ("{upload}-custom", false, false, "/api/custom"),
            ("${upload}-custom", true, false, "/api/custom"),
            ("{upload}{temp}-custom", false, true, "/api/custom"),
            ("${upload}{temp}-custom", true, true, "/api/custom"),
        ];

        for (pattern, expected_protected, expected_temporary, expected_route) in combinations {
            let entry = create_test_dir(temp_dir.path(), pattern);
            let route_params = RouteParams::new("/api", &entry, false);
            let result = RouteUpload::try_parse(route_params);

            match result {
                Route::Upload(route_upload) => {
                    assert_eq!(route_upload.route, expected_route, "Route mismatch for {}", pattern);
                    assert_eq!(route_upload.is_protected, expected_protected, "Protection mismatch for {}", pattern);
                    assert_eq!(route_upload.is_temporary, expected_temporary, "Temporary mismatch for {}", pattern);
                }
                _ => panic!("Expected Route::Upload for pattern: {}", pattern),
            }
        }
    }
}