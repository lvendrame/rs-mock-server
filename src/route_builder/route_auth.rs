use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    app::App,
    handlers::build_auth_routes,
    route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}
};

static RE_FILE_AUTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\{auth\}$").unwrap()
});

#[derive(Debug, Clone, PartialEq)]
pub struct  RouteAuth {
    pub path: OsString,
    pub route: String,
}

impl RouteAuth {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if RE_FILE_AUTH.is_match(&route_params.file_stem) {

            let route_auth = Self {
                path: route_params.file_path,
                route: route_params.full_route,
            };

            return Route::Auth(route_auth);
        }

        Route::None
    }
}

impl RouteGenerator for RouteAuth {
    fn make_routes(&self, app: &mut App) {
        build_auth_routes(app, &self.route, &self.path);
    }
}

impl PrintRoute for RouteAuth {
    fn println(&self) {
        println!("✔️ Built AUTH routes for {}", self.route);
    }
}