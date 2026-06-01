use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    app::App,
    handlers::build_graphql_routes,
    route_builder::{PrintRoute, Route, RouteGenerator, route_params::RouteParams},
};

static RE_FOLDER_GRAPHQL: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\$)?graphql$").unwrap());

const ELEMENT_IS_PROTECTED: usize = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct RouteGraphQL {
    pub path: OsString,
    pub route: String,
    pub delay: Option<u16>,
    pub is_protected: bool,
}

impl RouteGraphQL {
    pub fn new(path: OsString, route: String, is_protected: bool, delay: Option<u16>) -> Self {
        Self {
            path,
            route,
            is_protected,
            delay,
        }
    }

    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_FOLDER_GRAPHQL.captures(&route_params.file_stem) {
            let config = route_params.config.clone();
            let route_config = config.route.clone().unwrap_or_default();

            let delay = route_config.delay;
            let is_protected = route_config.protect.unwrap_or(false);
            let is_protected = is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();

            let route = route_config.remap.unwrap_or(route_params.full_route);

            let route_graphql = Self {
                path: route_params.file_path,
                route,
                delay,
                is_protected,
            };

            return Route::GraphQL(route_graphql);
        }

        Route::None
    }
}

impl RouteGenerator for RouteGraphQL {
    fn make_routes(&self, app: &mut App) {
        build_graphql_routes(app, self);
    }
}

impl PrintRoute for RouteGraphQL {
    fn println(&self) {
        println!("✔️ Built GraphQL routes");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_builder::config::{Config, ConfigStore};
    use tempfile::TempDir;

    fn dir_entry(dir: &std::path::Path, name: &str) -> std::fs::DirEntry {
        std::fs::create_dir(dir.join(name)).unwrap();
        std::fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.file_name() == name)
            .unwrap()
    }

    #[test]
    fn new_stores_graphql_route_configuration() {
        let route = RouteGraphQL::new("graphql".into(), "/graphql".to_string(), true, Some(5));
        assert_eq!(route.path, OsString::from("graphql"));
        assert_eq!(route.route, "/graphql");
        assert!(route.is_protected);
        assert_eq!(route.delay, Some(5));
        route.println();
    }

    #[test]
    fn try_parse_accepts_graphql_and_protected_graphql_folders() {
        let temp_dir = TempDir::new().unwrap();

        let entry = dir_entry(temp_dir.path(), "graphql");
        let route = RouteGraphQL::try_parse(RouteParams::new(
            "/api/graphql",
            &entry,
            Config::default(),
            &ConfigStore::default(),
        ));
        match route {
            Route::GraphQL(graphql) => {
                assert_eq!(graphql.route, "/api/graphql/graphql");
                assert!(!graphql.is_protected);
            }
            _ => panic!("Expected GraphQL route"),
        }

        let entry = dir_entry(temp_dir.path(), "$graphql");
        let route = RouteGraphQL::try_parse(RouteParams::new(
            "/secure/graphql",
            &entry,
            Config::default(),
            &ConfigStore::default(),
        ));
        match route {
            Route::GraphQL(graphql) => assert!(graphql.is_protected),
            _ => panic!("Expected protected GraphQL route"),
        }
    }

    #[test]
    fn try_parse_rejects_non_graphql_folder() {
        let temp_dir = TempDir::new().unwrap();
        let entry = dir_entry(temp_dir.path(), "api");
        assert!(
            RouteGraphQL::try_parse(RouteParams::new(
                "/api",
                &entry,
                Config::default(),
                &ConfigStore::default(),
            ))
            .is_none()
        );
    }

    #[test]
    fn make_routes_delegates_to_graphql_builder() {
        let temp_dir = TempDir::new().unwrap();
        let route = RouteGraphQL::new(
            temp_dir.path().as_os_str().to_os_string(),
            "/graphql".to_string(),
            false,
            None,
        );
        let mut app = App::default();
        route.make_routes(&mut app);
        assert!(
            app.pages
                .lock()
                .unwrap()
                .render_index()
                .contains("/graphql")
        );
    }
}
