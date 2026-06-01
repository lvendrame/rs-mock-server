use std::fs::{self, DirEntry};

use crate::{
    app::App,
    route_builder::{
        Route, RouteGenerator, RouteParams,
        config::{Config, ConfigStore, Mergeable},
    },
};

#[derive(Debug, Default)]
pub struct RouteManager {
    pub auth_route: Route,
    pub routes: Vec<Route>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self {
            auth_route: Route::None,
            routes: vec![],
        }
    }

    pub fn from_dir(root_path: &str, config: Option<Config>) -> Self {
        let start_time = std::time::Instant::now();
        println!("Start - Loading routes");

        let parent_route = config
            .clone()
            .unwrap_or_default()
            .route
            .unwrap_or_default()
            .remap
            .unwrap_or("".into());

        let mut manager = Self::new();
        manager.load_dir(&parent_route, root_path, config);
        manager.sort();

        println!(
            "Finish - Loading routes. Routes loaded in {:?}",
            start_time.elapsed()
        );

        manager
    }

    fn load_dir(&mut self, parent_route: &str, entries_path: &str, config: Option<Config>) {
        let config_store = ConfigStore::try_from_dir(entries_path).unwrap_or_else(|err| {
            panic!(
                "Unable to load configs from {}. Error: {:?}",
                entries_path, err
            )
        });

        let config = config_store.get("config").merge(config);

        let entries = fs::read_dir(entries_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            self.load_entry(parent_route, &entry, &config, &config_store);
        }
    }

    fn load_entry(
        &mut self,
        parent_route: &str,
        entry: &DirEntry,
        config: &Option<Config>,
        config_store: &ConfigStore,
    ) {
        let route_params = RouteParams::new(
            parent_route,
            entry,
            config.clone().unwrap_or_default(),
            config_store,
        );
        if route_params.file_extension == "toml" {
            return;
        }

        let route = Route::try_parse(&route_params);

        if route.is_none() {
            if route_params.is_dir {
                self.load_dir(
                    &route_params.full_route,
                    &route_params.file_path.to_string_lossy(),
                    Some(route_params.config.clone()),
                );
            }
            return;
        }

        if let Route::Auth(_) = route {
            if self.auth_route.is_some() {
                panic!("Only one auth route is allowed");
            }
            self.auth_route = route;
        } else {
            self.routes.push(route);
        }
    }

    fn sort(&mut self) {
        self.routes
            .sort_by(|ra, rb| ra.partial_cmp(rb).unwrap_or(std::cmp::Ordering::Equal));
    }
}

impl RouteGenerator for RouteManager {
    fn make_routes(&self, app: &mut App) {
        self.auth_route.make_routes_and_print(app);

        for route in self.routes.iter() {
            route.make_routes_and_print(app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_builder::config::{Config, RouteConfig};
    use tempfile::TempDir;

    #[test]
    fn from_dir_loads_routes_recursively_and_skips_toml_files() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("users")).unwrap();
        std::fs::write(temp_dir.path().join("users").join("get.json"), "{}").unwrap();
        std::fs::write(
            temp_dir.path().join("users").join("config.toml"),
            "[route]\nprotect = true",
        )
        .unwrap();
        std::fs::create_dir(temp_dir.path().join("public-assets")).unwrap();

        let manager = RouteManager::from_dir(
            temp_dir.path().to_str().unwrap(),
            Some(Config {
                route: Some(RouteConfig {
                    remap: Some("/api".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );

        assert!(manager.auth_route.is_none());
        assert_eq!(manager.routes.len(), 2);
        assert!(
            manager
                .routes
                .iter()
                .any(|route| matches!(route, Route::Basic(_)))
        );
        assert!(
            manager
                .routes
                .iter()
                .any(|route| matches!(route, Route::Public(_)))
        );
    }

    #[test]
    #[should_panic(expected = "Only one auth route is allowed")]
    fn from_dir_rejects_multiple_auth_routes() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join("a")).unwrap();
        std::fs::create_dir(temp_dir.path().join("b")).unwrap();
        std::fs::write(temp_dir.path().join("a").join("{auth}.json"), "[]").unwrap();
        std::fs::write(temp_dir.path().join("b").join("{auth}.json"), "[]").unwrap();

        RouteManager::from_dir(temp_dir.path().to_str().unwrap(), None);
    }

    #[test]
    fn make_routes_registers_loaded_routes() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("get.json"), "{}").unwrap();

        let manager = RouteManager::from_dir(
            temp_dir.path().to_str().unwrap(),
            Some(Config {
                route: Some(RouteConfig {
                    remap: Some("/api".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        );
        let mut app = App::default();
        manager.make_routes(&mut app);

        assert!(app.pages.lock().unwrap().render_index().contains("GET"));
    }
}
