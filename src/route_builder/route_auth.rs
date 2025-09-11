use std::ffi::OsString;

use fosk::IdType;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    app::App,
    handlers::build_auth_routes,
    route_builder::{route_params::RouteParams, CollectionConfig, PrintRoute, Route, RouteGenerator}
};

static RE_FILE_AUTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\{auth\}$").unwrap()
});

static ID_FIELD: &str = "id";
static TOKEN_FIELD: &str = "token";
static USERNAME_FIELD: &str = "username";
static PASSWORD_FIELD: &str = "password";
static ROLES_FIELD: &str = "roles";
static JWT_SECRET: &str = "1!2@3#4$5%6â7&8*9(0)-_=+±§";
static COOKIE_NAME: &str = "auth_token";

pub static USER_COLLECTION: &str = "internal_auth_users";
pub static TOKEN_COLLECTION: &str = "internal_auth_tokens";

pub static LOGIN_ENDPOINT: &str = "/login";
pub static LOGOUT_ENDPOINT: &str = "/logout";
pub static USERS_ENDPOINT: &str = "/users";

#[derive(Debug, Clone, PartialEq)]
pub struct  RouteAuth {
    pub path: OsString,
    pub route: String,
    pub delay: Option<u16>,
    pub login_endpoint: String,
    pub logout_endpoint: String,
    pub users_route: String,
    pub token_collection: CollectionConfig,
    pub user_collection: CollectionConfig,
    pub username_field: String,
    pub password_field: String,
    pub roles_field: String,
    pub jwt_secret: String,
    pub cookie_name: String,
    pub encrypt_password: bool,
}

impl RouteAuth {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if RE_FILE_AUTH.is_match(&route_params.file_stem) {
            let config = route_params.config.clone();
            let route_config = config.route.clone().unwrap_or_default();
            let auth_config = config.auth.clone().unwrap_or_default();
            let token_coll_config = auth_config.token_collection.clone().unwrap_or_default();
            let users_coll_config = auth_config.user_collection.clone().unwrap_or_default();

            let route = route_config.remap.unwrap_or(route_params.full_route);

            let route_auth = Self {
                path: route_params.file_path,
                route: route.clone(),
                delay: route_config.delay,
                login_endpoint: auth_config.login_endpoint.unwrap_or(LOGIN_ENDPOINT.into()),
                logout_endpoint: auth_config.logout_endpoint.unwrap_or(LOGOUT_ENDPOINT.into()),
                users_route: auth_config.users_route.unwrap_or(format!("{}{}", route, USERS_ENDPOINT)),
                token_collection: CollectionConfig {
                    name: token_coll_config.name.unwrap_or(TOKEN_COLLECTION.into()),
                    id_key: token_coll_config.id_key.unwrap_or(TOKEN_FIELD.into()),
                    id_type: token_coll_config.id_type.unwrap_or(IdType::None),
                },
                user_collection: CollectionConfig {
                    name: users_coll_config.name.unwrap_or(USER_COLLECTION.into()),
                    id_key: users_coll_config.id_key.unwrap_or(ID_FIELD.into()),
                    id_type: users_coll_config.id_type.unwrap_or_default(),
                },
                username_field: auth_config.username_field.unwrap_or(USERNAME_FIELD.into()),
                password_field: auth_config.password_field.unwrap_or(PASSWORD_FIELD.into()),
                roles_field: auth_config.roles_field.unwrap_or(ROLES_FIELD.into()),
                cookie_name: auth_config.cookie_name.unwrap_or(COOKIE_NAME.into()),
                jwt_secret: auth_config.jwt_secret.unwrap_or(JWT_SECRET.into()),
                encrypt_password: auth_config.encrypt_password.unwrap_or(false),
            };

            return Route::Auth(route_auth);
        }

        Route::None
    }
}

impl RouteGenerator for RouteAuth {
    fn make_routes(&self, app: &mut App) {
        build_auth_routes(app, self);
    }
}

impl PrintRoute for RouteAuth {
    fn println(&self) {
        println!("✔️ Built AUTH route for {}/login", self.route);
        println!("✔️ Built logout routes for {}/logout", self.route);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::Path;
    use tempfile::TempDir;
    use crate::route_builder::config::{Config, ConfigStore};
    use crate::route_builder::route_params::RouteParams;

    fn create_test_file(dir: &Path, filename: &str) -> std::fs::DirEntry {
        let file_path = dir.join(filename);
        File::create(&file_path).unwrap();

        let entries: Vec<std::fs::DirEntry> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy() == filename)
            .collect();

        entries.into_iter().next().unwrap()
    }

    #[test]
    fn test_try_parse_with_valid_auth_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "{auth}.json");
        let route_params = RouteParams::new("/api/auth", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteAuth::try_parse(route_params);

        match result {
            Route::Auth(auth_route) => {
                assert_eq!(auth_route.route, "/api/auth");
                assert!(auth_route.path.to_string_lossy().contains("{auth}.json"));
            }
            _ => panic!("Expected Route::Auth, got {:?}", result),
        }
    }

    #[test]
    fn test_try_parse_with_auth_file_different_extension() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "{auth}.txt");
        let route_params = RouteParams::new("/account", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteAuth::try_parse(route_params);

        match result {
            Route::Auth(auth_route) => {
                assert_eq!(auth_route.route, "/account");
                assert!(auth_route.path.to_string_lossy().contains("{auth}.txt"));
            }
            _ => panic!("Expected Route::Auth, got {:?}", result),
        }
    }

    #[test]
    fn test_try_parse_with_auth_file_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "{auth}");
        let route_params = RouteParams::new("/login", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteAuth::try_parse(route_params);

        match result {
            Route::Auth(auth_route) => {
                assert_eq!(auth_route.route, "/login");
                assert!(auth_route.path.to_string_lossy().contains("{auth}"));
            }
            _ => panic!("Expected Route::Auth, got {:?}", result),
        }
    }

    #[test]
    fn test_try_parse_with_root_level_auth() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "{auth}.json");
        let route_params = RouteParams::new("", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteAuth::try_parse(route_params);

        match result {
            Route::Auth(auth_route) => {
                assert_eq!(auth_route.route, "");
                assert!(auth_route.path.to_string_lossy().contains("{auth}.json"));
            }
            _ => panic!("Expected Route::Auth, got {:?}", result),
        }
    }

    #[test]
    fn test_try_parse_with_protected_auth_file() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "{auth}.json");
        let route_params = RouteParams::new("/api/auth", &entry, Config::default().with_protect(true), &ConfigStore::default()); // Protected context

        let result = RouteAuth::try_parse(route_params);

        match result {
            Route::Auth(auth_route) => {
                assert_eq!(auth_route.route, "/api/auth");
                assert!(auth_route.path.to_string_lossy().contains("{auth}.json"));
            }
            _ => panic!("Expected Route::Auth, got {:?}", result),
        }
    }

    #[test]
    fn test_try_parse_with_invalid_auth_filename() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "auth.json");
        let route_params = RouteParams::new("/api/auth", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteAuth::try_parse(route_params);

        match result {
            Route::None => {
                // Expected - should not match without curly braces
            }
            _ => panic!("Expected Route::None, got {:?}", result),
        }
    }

    #[test]
    fn test_try_parse_with_partial_auth_match() {
        let temp_dir = TempDir::new().unwrap();
        let entry = create_test_file(temp_dir.path(), "{auth}extra.json");
        let route_params = RouteParams::new("/api/auth", &entry, Config::default().with_protect(false), &ConfigStore::default());

        let result = RouteAuth::try_parse(route_params);

        match result {
            Route::None => {
                // Expected - should only match exact "{auth}" pattern
            }
            _ => panic!("Expected Route::None, got {:?}", result),
        }
    }

    #[test]
    fn test_try_parse_with_similar_but_invalid_patterns() {
        let temp_dir = TempDir::new().unwrap();

        let test_cases = vec![
            "auth.json",           // Missing braces
            "{authenticate}.json", // Different word
            "{auth.json",          // Missing closing brace
            "auth}.json",          // Missing opening brace
            "{AUTH}.json",         // Wrong case
            "{auth }.json",        // Extra space
            "{ auth}.json",        // Extra space
            "{auth}{test}.json",   // Extra content
        ];

        for filename in test_cases {
            let entry = create_test_file(temp_dir.path(), filename);
            let route_params = RouteParams::new("/api", &entry, Config::default().with_protect(false), &ConfigStore::default());

            let result = RouteAuth::try_parse(route_params);

            match result {
                Route::None => {
                    // Expected for all invalid patterns
                }
                _ => panic!("Expected Route::None for filename '{}', got {:?}", filename, result),
            }
        }
    }

    #[test]
    fn test_try_parse_with_regular_files() {
        let temp_dir = TempDir::new().unwrap();

        let test_cases = vec![
            "get.json",
            "post.json",
            "rest.json",
            "{upload}.json",
            "get{id}.json",
            "get{1-5}.json",
        ];

        for filename in test_cases {
            let entry = create_test_file(temp_dir.path(), filename);
            let route_params = RouteParams::new("/api/test", &entry, Config::default().with_protect(false), &ConfigStore::default());

            let result = RouteAuth::try_parse(route_params);

            match result {
                Route::None => {
                    // Expected - these should not match auth pattern
                }
                _ => panic!("Expected Route::None for filename '{}', got {:?}", filename, result),
            }
        }
    }

    #[test]
    fn test_regex_pattern_directly() {
        // Test the regex pattern directly to ensure it's working correctly
        assert!(RE_FILE_AUTH.is_match("{auth}"));
        assert!(!RE_FILE_AUTH.is_match("auth"));
        assert!(!RE_FILE_AUTH.is_match("{auth}extra"));
        assert!(!RE_FILE_AUTH.is_match("prefix{auth}"));
        assert!(!RE_FILE_AUTH.is_match("{AUTH}"));
        assert!(!RE_FILE_AUTH.is_match("{auth }"));
        assert!(!RE_FILE_AUTH.is_match("{ auth}"));
        assert!(!RE_FILE_AUTH.is_match("{authenticate}"));
        assert!(!RE_FILE_AUTH.is_match("{auth}{test}"));
    }
}
