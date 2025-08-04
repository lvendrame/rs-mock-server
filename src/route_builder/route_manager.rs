use std::fs::{self, DirEntry};

use crate::{app::App, route_builder::{PrintRoute, Route, RouteGenerator, RouteParams}};

#[derive(Debug, Default)]
pub struct RouteManager {
    pub auth_route: Route,
    pub routes: Vec<Route>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self { auth_route: Route::None, routes: vec![] }
    }

    pub fn from_dir(root_path: &str) -> Self {
        let mut manager = Self::new();
        manager.load_dir("", root_path, false);
        manager
    }

    fn load_dir(&mut self, parent_route: &str, entries_path: &str, is_protected: bool) {
        let entries = fs::read_dir(entries_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            self.load_entry(parent_route, &entry, is_protected);
        }
    }

    fn load_entry(&mut self, parent_route: &str, entry: &DirEntry, is_protected: bool) {
        let route_params = RouteParams::new(parent_route, entry, is_protected);

        let route = Route::try_parse(&route_params);

        if route.is_none() {
            if route_params.is_dir {
                self.load_dir(
                    &route_params.full_route,
                    &route_params.file_path.to_string_lossy(),
                    route_params.is_protected
                );
            }
            return;
        }

        if let Route::Auth(_) = route {
            self.auth_route = route;
        } else {
            self.routes.push(route);
        }
    }
    fn make_route(&self, app: &mut App, route: &Route){
        if route.is_some() {
            route.println();
        }
    }

}

impl RouteGenerator for RouteManager {
    fn make_routes(&self, app: &mut App) {
        self.make_route(app, &self.auth_route);

        for route in self.routes.iter() {
            self.make_route(app, route);
        }
    }
}