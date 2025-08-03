use std::ffi::OsString;

use crate::route_builder::PrintRoute;

pub struct  RoutePublic {
    pub path: OsString,
    pub route: String,
    pub is_protected: bool,
}

impl RoutePublic {
    pub fn try_parse(parent_route: &str, file_name: String, file_path: OsString) -> Option<Self> {
        if file_name.starts_with("public") {
            let public_route = if let Some((_, to)) = file_name.split_once('-') {
                to
            } else {
                "public"
            };

            let route = if parent_route.is_empty() { "/" } else { parent_route };
            let route = format!("{}/{}", route, public_route);

            let route_public = Self {
                path: file_path,
                route: route.to_string(),
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