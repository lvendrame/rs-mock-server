use std::{fs::{self, DirEntry}};

use crate::{app::App, route_builder::{config::{Config, ConfigStore, Mergeable}, Route, RouteGenerator, RouteParams}};

#[derive(Debug, Default)]
pub struct RouteManager {
    pub auth_route: Route,
    pub routes: Vec<Route>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self { auth_route: Route::None, routes: vec![] }
    }

    pub fn from_dir(root_path: &str, config: Option<Config>) -> Self {
        let start_time = std::time::Instant::now();
        println!("Start - Loading routes");

        let mut manager = Self::new();
        manager.load_dir("", root_path, config);
        manager.sort();

        println!("Finish - Loading routes. Routes loaded in {:?}", start_time.elapsed());

        manager
    }

    fn load_dir(&mut self, parent_route: &str, entries_path: &str, config: Option<Config>) {
        let config_store = ConfigStore::try_from_dir(entries_path)
            .unwrap_or_else(|err| panic!("Unable to load configs from {}. Error: {:?}", entries_path, err));

        let config = config_store.get("config").merge(config);

        let entries = fs::read_dir(entries_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            self.load_entry(parent_route, &entry, &config, &config_store);
        }
    }

    fn load_entry(&mut self, parent_route: &str, entry: &DirEntry, config: &Option<Config>, config_store: &ConfigStore) {
        let route_params = RouteParams::new(parent_route, entry, config.clone().unwrap_or_default(), config_store);

        let route = Route::try_parse(&route_params);

        if route.is_none() {
            if route_params.is_dir {
                self.load_dir(
                    &route_params.full_route,
                    &route_params.file_path.to_string_lossy(),
                    Some(route_params.config.clone())
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
        self.routes.sort_by(|ra, rb| {
            ra.partial_cmp(rb).unwrap_or(std::cmp::Ordering::Equal)
        });
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
