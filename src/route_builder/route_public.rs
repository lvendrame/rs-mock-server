use std::ffi::OsString;

use crate::{app::App, route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}};

#[derive(Debug, Clone, PartialEq)]
pub struct  RoutePublic {
    pub path: OsString,
    pub route: String,
    pub is_protected: bool,
}

impl RoutePublic {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if route_params.file_name.starts_with("public") {
            let public_route = if let Some((_, to)) = route_params.file_name.split_once('-') {
                to
            } else {
                "public"
            };


            let route = format!("{}/{}", route_params.parent_route, public_route);

            let route_public = Self {
                path: route_params.file_path,
                route,
                is_protected: false,
            };

            return Route::Public(route_public);
        }

        Route::None
    }
}

impl RouteGenerator for RoutePublic {
    fn make_routes(&self, app: &mut App) {
        app.build_public_router_v2(&self.path, &self.route);
    }
}

impl PrintRoute for RoutePublic {
    fn println(&self) {
        println!("✔️ Built public routes from folder {} to {}", self.path.to_string_lossy(), self.route);
    }
}