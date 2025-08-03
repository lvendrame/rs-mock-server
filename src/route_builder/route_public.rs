use std::ffi::OsString;

use crate::route_builder::{route_params::RouteParams, PrintRoute};

pub struct  RoutePublic {
    pub path: OsString,
    pub route: String,
    pub is_protected: bool,
}

impl RoutePublic {
    pub fn try_parse(route_params: RouteParams) -> Option<Self> {
        if route_params.file_name.starts_with("public") {
            let public_route = if let Some((_, to)) = route_params.file_name.split_once('-') {
                to
            } else {
                "public"
            };


            let route = format!("{}/{}", route_params.full_route, public_route);

            let route_public = Self {
                path: route_params.file_path,
                route,
                is_protected: false,
            };

            return Some(route_public);
        }

        None
    }
}

impl PrintRoute for RoutePublic {
    fn println(&self) {
        println!("✔️ Built public routes for {}", self.route);
    }
}