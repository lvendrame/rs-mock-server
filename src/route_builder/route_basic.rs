use std::{ffi::OsString, fmt::Display};

use http::Method;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{handlers::build_method_router, route_builder::{method_from_str, route_params::RouteParams, PrintRoute, Route, RouteGenerator, RouteRegistrator}};

static RE_FILE_METHODS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?(get|post|put|patch|delete|options)(\{(.+)\})?$").unwrap()
});

static RE_FILE_PARAM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?(.+?)(\{(.+)\})$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_METHOD: usize = 2;
const ELEMENT_DESCRIPTOR: usize = 4;

const ELEMENT_ROUTE_NAME: usize = 2;
const ELEMENT_PARAM: usize = 3;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum SubRoute {
    #[default]
    None,
    Id,
    Range(u32, u32),
    Static(String)
}

impl SubRoute {
    pub fn from(pattern: Option<regex::Match<'_>>) -> Self {
        if pattern.is_none() {
            return Self::None;
        }

        let pattern = pattern.unwrap().as_str();
        if pattern == "id" {
            return Self::Id;
        }

        if pattern.contains('-') {
            if let Some((start_str, end_str)) = pattern.split_once('-') {
                if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
                    return Self::Range(start, end);
                }
            }
        }

        Self::Static(pattern.to_string())
    }
}

impl Display for SubRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubRoute::None => write!(f, ""),
            SubRoute::Id => write!(f, "/{{id}}"),
            SubRoute::Static(value) => write!(f, "/{{{}}}", value),
            SubRoute::Range(start, end) => write!(f, "/{{{}-{}}}", start, end),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteBasic {
    pub path: OsString,
    pub method: Method,
    pub route: String,
    pub sub_route: SubRoute,
    pub is_protected: bool,
}

impl RouteBasic {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_FILE_METHODS.captures(&route_params.file_stem) {
            let is_protected = route_params.is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let method = captures.get(ELEMENT_METHOD).unwrap().as_str();
            let pattern = captures.get(ELEMENT_DESCRIPTOR);

            let route_basic = Self {
                path: route_params.file_path,
                method: method_from_str(method),
                route: route_params.full_route,
                sub_route: SubRoute::from(pattern),
                is_protected,
            };

            return Route::Basic(route_basic);
        }

        if let Some(captures) = RE_FILE_PARAM.captures(&route_params.file_stem) {
            let is_protected = route_params.is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let route = captures.get(ELEMENT_ROUTE_NAME).unwrap().as_str();
            let param = captures.get(ELEMENT_PARAM);

            let route_basic = Self {
                path: route_params.file_path,
                method: method_from_str(route),
                route: format!("{}/{}", route_params.full_route, route_params.file_stem.replace(param.unwrap().as_str(), "")),
                sub_route: SubRoute::from(param),
                is_protected,
            };

            return Route::Basic(route_basic);
        }

        let route_basic = Self {
            path: route_params.file_path,
            method: Method::GET,
            route: format!("{}/{}", route_params.full_route, route_params.file_stem),
            sub_route: SubRoute::None,
            is_protected: route_params.is_protected,
        };

        Route::Basic(route_basic)
    }
}

impl RouteGenerator for RouteBasic {
    fn make_routes(&self, app: &mut crate::app::App) {
        let method = self.method.as_str();

        match &self.sub_route {
            SubRoute::None => {
                let router = build_method_router(app, &self.path, method);
                app.push_route(&self.route, router, Some(method), self.is_protected, None);
            },
            SubRoute::Id => {
                let route_path = format!("{}/{}", self.route, "{id}");
                let router = build_method_router(app, &self.path, method);
                app.push_route(&route_path, router, Some(method), self.is_protected, None);
            },
            SubRoute::Range(start, end) => {
                for i in *start..=*end {
                    let route_path = format!("{}/{}", self.route, i);
                    let router = build_method_router(app, &self.path, method);
                    app.push_route(&route_path, router, Some(method), self.is_protected, None);
                }
            },
            SubRoute::Static(end_point) => {
                let route_path = format!("{}/{}", self.route, end_point);
                let router = build_method_router(app, &self.path, method);
                app.push_route(&route_path, router, Some(method), self.is_protected, None);
            },
        }

    }
}

impl PrintRoute for RouteBasic {
    fn println(&self) {
        let path = &self.path.to_string_lossy();
        let method = self.method.as_str();
        let route = &self.route;
        let subroute = self.sub_route.to_string();

        println!("✔️ Mapped {} to {} {}{}", path, method, route, subroute);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_try_parse_get_method() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get.json");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_post_method() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "post.json");
        let route_params = RouteParams::new("/api/users", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::POST);
                assert_eq!(route_basic.route, "/api/users");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_protected_method() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "$put.json");
        let route_params = RouteParams::new("/api/data", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::PUT);
                assert_eq!(route_basic.route, "/api/data");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_method_with_id_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get{id}.json");
        let route_params = RouteParams::new("/api/items", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/items");
                assert_eq!(route_basic.sub_route, SubRoute::Id);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_method_with_range_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "delete{1-5}.json");
        let route_params = RouteParams::new("/api/resources", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::DELETE);
                assert_eq!(route_basic.route, "/api/resources");
                assert_eq!(route_basic.sub_route, SubRoute::Range(1, 5));
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_method_with_static_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "patch{admin}.json");
        let route_params = RouteParams::new("/api/config", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::PATCH);
                assert_eq!(route_basic.route, "/api/config");
                assert_eq!(route_basic.sub_route, SubRoute::Static("admin".to_string()));
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_protected_with_descriptor() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "$options{special}.json");
        let route_params = RouteParams::new("/api/auth", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::OPTIONS);
                assert_eq!(route_basic.route, "/api/auth");
                assert_eq!(route_basic.sub_route, SubRoute::Static("special".to_string()));
                assert!(route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_inherited_protection() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get.json");
        let route_params = RouteParams::new("/api/secure", &entry, true);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/secure");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_all_http_methods() {
        let temp_dir = TempDir::new().unwrap();
        let methods = vec![
            ("get.json", Method::GET),
            ("post.json", Method::POST),
            ("put.json", Method::PUT),
            ("patch.json", Method::PATCH),
            ("delete.json", Method::DELETE),
            ("options.json", Method::OPTIONS),
        ];

        for (filename, expected_method) in methods {
            let entry = create_test_file(temp_dir.path(), filename);
            let route_params = RouteParams::new("/api/test", &entry, false);
            let result = RouteBasic::try_parse(route_params);

            match result {
                Route::Basic(route_basic) => {
                    assert_eq!(route_basic.method, expected_method, "Failed for {}", filename);
                }
                _ => panic!("Expected Route::Basic for {}", filename),
            }
        }
    }

    #[test]
    fn test_try_parse_non_method_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "config.json");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                // Now it should create a static GET route with the file stem
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/config");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic for non-method file"),
        }
    }

    #[test]
    fn test_try_parse_complex_range() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get{10-20}.json");
        let route_params = RouteParams::new("/api/pages", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/pages");
                assert_eq!(route_basic.sub_route, SubRoute::Range(10, 20));
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    #[test]
    fn test_try_parse_file_path_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "get.json");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                let expected_path = temp_dir.path().join("get.json").into_os_string();
                assert_eq!(route_basic.path, expected_path);
            }
            _ => panic!("Expected Route::Basic"),
        }
    }

    // Tests for new static route behavior (non-method files)

    #[test]
    fn test_try_parse_static_route_simple() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "users.json");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/users");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic for static route"),
        }
    }

    #[test]
    fn test_try_parse_static_route_with_protection() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "admin.json");
        let route_params = RouteParams::new("/api", &entry, true);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/admin");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(route_basic.is_protected); // Should inherit protection
            }
            _ => panic!("Expected Route::Basic for protected static route"),
        }
    }

    #[test]
    fn test_try_parse_static_route_different_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let test_cases = vec![
            ("data.json", "data"),
            ("info.xml", "info"),
            ("config.yaml", "config"),
            ("settings.txt", "settings"),
            ("readme.md", "readme"),
        ];

        for (filename, expected_stem) in test_cases {
            let entry = create_test_file(temp_dir.path(), filename);
            let route_params = RouteParams::new("/files", &entry, false);
            let result = RouteBasic::try_parse(route_params);

            match result {
                Route::Basic(route_basic) => {
                    assert_eq!(route_basic.method, Method::GET);
                    assert_eq!(route_basic.route, format!("/files/{}", expected_stem));
                    assert_eq!(route_basic.sub_route, SubRoute::None);
                    assert!(!route_basic.is_protected);
                }
                _ => panic!("Expected Route::Basic for {}", filename),
            }
        }
    }

    #[test]
    fn test_try_parse_static_route_complex_names() {
        let temp_dir = TempDir::new().unwrap();
        let test_cases = vec![
            ("user-profile.json", "user-profile"),
            ("api_documentation.md", "api_documentation"),
            ("system.config.xml", "system"),
            ("data-2024.csv", "data-2024"),
        ];

        for (filename, expected_stem) in test_cases {
            let entry = create_test_file(temp_dir.path(), filename);
            let route_params = RouteParams::new("/resources", &entry, false);
            let result = RouteBasic::try_parse(route_params);

            match result {
                Route::Basic(route_basic) => {
                    assert_eq!(route_basic.method, Method::GET);
                    assert_eq!(route_basic.route, format!("/resources/{}", expected_stem));
                    assert_eq!(route_basic.sub_route, SubRoute::None);
                    assert!(!route_basic.is_protected);
                }
                _ => panic!("Expected Route::Basic for {}", filename),
            }
        }
    }

    #[test]
    fn test_try_parse_static_route_nested_paths() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "statistics.json");
        let route_params = RouteParams::new("/api/v1/reports", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/v1/reports/statistics");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic for nested static route"),
        }
    }

    #[test]
    fn test_try_parse_static_route_with_empty_parent() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "index.html");
        let route_params = RouteParams::new("", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/index");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic for static route with empty parent"),
        }
    }

    #[test]
    fn test_try_parse_static_route_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "readme");
        let route_params = RouteParams::new("/docs", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/docs/readme");
                assert_eq!(route_basic.sub_route, SubRoute::None);
                assert!(!route_basic.is_protected);
            }
            _ => panic!("Expected Route::Basic for file without extension"),
        }
    }

    #[test]
    fn test_try_parse_static_route_vs_method_precedence() {
        let temp_dir = TempDir::new().unwrap();

        // Method files should still work as before
        let method_entry = create_test_file(temp_dir.path(), "get.json");
        let method_params = RouteParams::new("/api", &method_entry, false);
        let method_result = RouteBasic::try_parse(method_params);

        match method_result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api");
                assert_eq!(route_basic.sub_route, SubRoute::None);
            }
            _ => panic!("Expected Route::Basic for method file"),
        }

        // Non-method files should create static routes
        let static_entry = create_test_file(temp_dir.path(), "getconfig.json");
        let static_params = RouteParams::new("/api", &static_entry, false);
        let static_result = RouteBasic::try_parse(static_params);

        match static_result {
            Route::Basic(route_basic) => {
                assert_eq!(route_basic.method, Method::GET);
                assert_eq!(route_basic.route, "/api/getconfig");
                assert_eq!(route_basic.sub_route, SubRoute::None);
            }
            _ => panic!("Expected Route::Basic for static file"),
        }
    }

    #[test]
    fn test_try_parse_static_route_file_path_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "metadata.json");
        let route_params = RouteParams::new("/api", &entry, false);

        let result = RouteBasic::try_parse(route_params);

        match result {
            Route::Basic(route_basic) => {
                let expected_path = temp_dir.path().join("metadata.json").into_os_string();
                assert_eq!(route_basic.path, expected_path);
                assert_eq!(route_basic.route, "/api/metadata");
            }
            _ => panic!("Expected Route::Basic"),
        }
    }
}
