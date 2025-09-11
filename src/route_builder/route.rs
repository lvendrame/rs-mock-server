use std::cmp::Ordering;

use fosk::IdType;

use crate::{app::App, route_builder::{
    PrintRoute, RouteAuth, RouteBasic, RouteGenerator, RouteParams, RoutePublic, RouteRest, RouteUpload
}};

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionConfig {
    pub name: String,
    pub id_key: String,
    pub id_type: IdType,
}

#[derive(Debug, Default, PartialEq)]
pub enum Route {
    #[default]
    None,
    Auth(RouteAuth),
    Basic(RouteBasic),
    Rest(RouteRest),
    Public(RoutePublic),
    Upload(RouteUpload),
}

impl Route {
    pub fn is_none(&self) -> bool {
        *self == Route::None
    }

    pub fn is_some(&self) -> bool {
        *self != Route::None
    }

    pub fn try_parse(route_params: &RouteParams) -> Route {
        if route_params.file_name.starts_with(".") || route_params.file_name.ends_with(".toml") {
            return Route::None;
        }

        if route_params.is_dir {
            let route = RoutePublic::try_parse(route_params.clone());
            if route.is_some() {
                return route;
            }

            let route = RouteUpload::try_parse(route_params.clone());
            if route.is_some() {
                return route;
            }

            return Route::None;
        }

        let route = RouteRest::try_parse(route_params.clone());
        if route.is_some() {
            return route;
        }

        let route = RouteAuth::try_parse(route_params.clone());
        if route.is_some() {
            return route;
        }

        let route = RouteBasic::try_parse(route_params.clone());
        if route.is_some() {
            return route;
        }

        Route::None
    }



    pub fn make_routes_and_print(&self, app: &mut App){
        if self.is_some() {
            self.make_routes(app);
            self.println();
        }
    }
}

impl RouteGenerator for Route {
    fn make_routes(&self, app: &mut App) {
        match self {
            Route::None => (),
            Route::Auth(route_auth) => route_auth.make_routes(app),
            Route::Basic(route_basic) => route_basic.make_routes(app),
            Route::Public(route_public) => route_public.make_routes(app),
            Route::Rest(route_rest) => route_rest.make_routes(app),
            Route::Upload(route_upload) => route_upload.make_routes(app),
        }
    }
}

impl PrintRoute for Route {
    fn println(&self) {
        match self {
            Route::None => (),
            Route::Auth(route_auth) => route_auth.println(),
            Route::Basic(route_basic) => route_basic.println(),
            Route::Public(route_public) => route_public.println(),
            Route::Rest(route_rest) => route_rest.println(),
            Route::Upload(route_upload) => route_upload.println(),
        }
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {

        // First compare by enum discriminant order
        let self_order = match self {
            Route::None => 0,
            Route::Auth(_) => 1,
            Route::Basic(_) => 2,
            Route::Rest(_) => 3,
            Route::Public(_) => 4,
            Route::Upload(_) => 5,
        };
        let other_order = match other {
            Route::None => 0,
            Route::Auth(_) => 1,
            Route::Basic(_) => 2,
            Route::Rest(_) => 3,
            Route::Public(_) => 4,
            Route::Upload(_) => 5,
        };

        match self_order.cmp(&other_order) {
            Ordering::Equal => {
                // Same enum variant, compare by path then method
                match (self, other) {
                    (Route::None, Route::None) => Some(Ordering::Equal),
                    (Route::Auth(a), Route::Auth(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    (Route::Basic(a), Route::Basic(b)) => {
                        match a.path.cmp(&b.path) {
                            Ordering::Equal => a.method.to_string().partial_cmp(&b.method.to_string()),
                            other => Some(other),
                        }
                    },
                    (Route::Rest(a), Route::Rest(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    (Route::Public(a), Route::Public(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    (Route::Upload(a), Route::Upload(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    _ => unreachable!(),
                }
            },
            other => Some(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::route_builder::config::{Config, ConfigStore};

    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, DirEntry};
    use std::path::Path;

    fn create_test_route_params(file_name: &str, is_dir: bool, is_protected: bool) -> RouteParams {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let entry = if is_dir {
            let dir_path = base_path.join(file_name);
            fs::create_dir_all(&dir_path).unwrap();
            get_dir_entry(&dir_path)
        } else {
            let file_path = base_path.join(file_name);
            fs::write(&file_path, "test content").unwrap();
            get_dir_entry(&file_path)
        };

        RouteParams::new("/test", &entry, Config::default().with_protect(is_protected), &ConfigStore::default())
    }

    fn get_dir_entry(path: &Path) -> DirEntry {
        path.parent()
            .unwrap()
            .read_dir()
            .unwrap()
            .find(|entry| {
                entry.as_ref().unwrap().path() == path
            })
            .unwrap()
            .unwrap()
    }

    #[test]
    fn test_try_parse_hidden_files() {
        let route_params = create_test_route_params(".hidden", false, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);

        let route_params = create_test_route_params(".gitignore", false, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);

        let route_params = create_test_route_params(".env", false, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);
    }

    #[test]
    fn test_try_parse_directories_public() {
        // Test public directory
        let route_params = create_test_route_params("public", true, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Public(_)));

        // Test public-static directory
        let route_params = create_test_route_params("public-static", true, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Public(_)));
    }

    #[test]
    fn test_try_parse_directories_upload() {
        // Test upload directory - should use {upload} pattern
        let route_params = create_test_route_params("{upload}", true, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Upload(_)));

        // Test upload-temp directory - should use {upload}{temp} pattern
        let route_params = create_test_route_params("{upload}{temp}", true, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Upload(_)));

        // Test upload-images directory - should use {upload}-images pattern
        let route_params = create_test_route_params("{upload}-images", true, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Upload(_)));

        // Regular "upload" directory without braces should return None
        let route_params = create_test_route_params("upload", true, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);
    }

    #[test]
    fn test_try_parse_directories_none() {
        // Test regular directory that doesn't match public or upload patterns
        let route_params = create_test_route_params("regular_dir", true, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);

        let route_params = create_test_route_params("some_folder", true, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);

        let route_params = create_test_route_params("data", true, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);
    }

    #[test]
    fn test_try_parse_files_rest() {
        // Test REST files - only rest.json creates REST routes
        let route_params = create_test_route_params("rest.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Rest(_)));

        // These patterns with HTTP methods should be Basic routes, not REST
        let route_params = create_test_route_params("get{id}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("post{uuid}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("put{userId}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("delete{customId}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));
    }

    #[test]
    fn test_try_parse_files_rest_range() {
        // Test REST files - only rest.json creates REST routes
        let route_params = create_test_route_params("rest.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Rest(_)));

        // These patterns with HTTP methods should be Basic routes, not REST
        let route_params = create_test_route_params("get{1-5}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("post{10-20}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));
    }

    #[test]
    fn test_try_parse_files_rest_specific_values() {
        // Test REST files - only rest.json creates REST routes
        let route_params = create_test_route_params("rest.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Rest(_)));

        // These patterns with HTTP methods should be Basic routes, not REST
        let route_params = create_test_route_params("get{123}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("post{admin}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));
    }

    #[test]
    fn test_try_parse_files_auth() {
        // Test auth files
        let route_params = create_test_route_params("{auth}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Auth(_)));

        // Auth with different extensions
        let route_params = create_test_route_params("{auth}.html", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Auth(_)));
    }

    #[test]
    fn test_try_parse_files_basic_http_methods() {
        // Test basic HTTP method files
        let route_params = create_test_route_params("get.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("post.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("put.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("delete.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("patch.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("head.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("options.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));
    }

    #[test]
    fn test_try_parse_files_basic_static_routes() {
        // Test files that should become static GET routes
        let route_params = create_test_route_params("index.html", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("about.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("data.xml", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));
    }

    #[test]
    fn test_try_parse_priority_order() {
        // Test that REST parsing has priority over Auth
        /* This test had an error because get{auth} is a basic Route only rest.json is for rest, so i fixed it */
        let route_params = create_test_route_params("rest.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Rest(_)), "REST should have priority over Auth");

        // Test that Auth parsing has priority over Basic
        let route_params = create_test_route_params("{auth}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Auth(_)), "Auth should have priority over Basic");
    }

    #[test]
    fn test_try_parse_protected_routes() {
        // Test that protection flag is passed through to Basic routes
        let route_params = create_test_route_params("get.json", false, true);
        let route = Route::try_parse(&route_params);
        if let Route::Basic(basic_route) = route {
            assert!(basic_route.is_protected);
        } else {
            panic!("Expected Basic route");
        }

        // Test protected flag with $ prefix
        let route_params = create_test_route_params("$post.json", false, false);
        let route = Route::try_parse(&route_params);
        if let Route::Basic(basic_route) = route {
            assert!(basic_route.is_protected);
        } else {
            panic!("Expected Basic route with protection");
        }

        // Test Auth route creation
        let route_params = create_test_route_params("{auth}.json", false, true);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Auth(_)), "Should create Auth route");
    }

    #[test]
    fn test_try_parse_edge_cases() {
        // Test file with only extension
        let route_params = create_test_route_params(".json", false, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);

        // Test very long file names
        let long_name = "a".repeat(100) + ".json";
        let route_params = create_test_route_params(&long_name, false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        // Test file with just dots
        let route_params = create_test_route_params("...", false, false);
        let route = Route::try_parse(&route_params);
        assert_eq!(route, Route::None);
    }

    #[test]
    fn test_try_parse_various_extensions() {
        // Test different file extensions for basic routes
        let extensions = vec!["json", "html", "xml", "txt", "jpg", "png", "css", "js"];

        for ext in extensions {
            let filename = format!("test.{}", ext);
            let route_params = create_test_route_params(&filename, false, false);
            let route = Route::try_parse(&route_params);
            assert!(matches!(route, Route::Basic(_)), "File {} should create Basic route", filename);
        }
    }

    #[test]
    fn test_try_parse_complex_rest_patterns() {
        // Test REST files - only rest.json creates REST routes
        let route_params = create_test_route_params("rest.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Rest(_)));

        // These patterns with HTTP methods should be Basic routes, not REST
        // The comment was correct: get{*}, post{*} and put{*} are basic Route only rest.json is for rest
        let route_params = create_test_route_params("get{user-id}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("post{item_id}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("put{snake_case_id}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));
    }

    #[test]
    fn test_try_parse_malformed_patterns() {
        // Test malformed patterns that should fall back to Basic
        let route_params = create_test_route_params("get{.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("get}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("get{}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));
    }

    #[test]
    fn test_try_parse_case_sensitivity() {
        // Test case sensitivity for HTTP methods
        let route_params = create_test_route_params("GET.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        let route_params = create_test_route_params("Post.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)));

        // Auth should be case sensitive
        let route_params = create_test_route_params("{AUTH}.json", false, false);
        let route = Route::try_parse(&route_params);
        assert!(matches!(route, Route::Basic(_)), "Auth pattern should be case sensitive");
    }

    #[test]
    fn test_route_is_none_and_is_some() {
        let none_route = Route::None;
        assert!(none_route.is_none());
        assert!(!none_route.is_some());

        let route_params = create_test_route_params("get.json", false, false);
        let some_route = Route::try_parse(&route_params);
        assert!(!some_route.is_none());
        assert!(some_route.is_some());
    }
}

