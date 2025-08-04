use std::{fs::{self, DirEntry}};

use crate::{app::App, route_builder::{Route, RouteGenerator, RouteParams}};

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
        let start_time = std::time::Instant::now();
        println!("Start - Loading routes");

        let mut manager = Self::new();
        manager.load_dir("", root_path, false);
        manager.sort();

        println!("Finish - Loading routes. Routes loaded in {:?}", start_time.elapsed());

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