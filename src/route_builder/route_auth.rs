use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::route_builder::PrintRoute;

static RE_FILE_AUTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\{auth\}$").unwrap()
});

pub struct  RouteAuth {
    pub path: OsString,
    pub route: String,
}

impl RouteAuth {
    pub fn try_parse(parent_route: &str, file_name: String, file_path: OsString) -> Option<Self> {
        let file_stem = file_name.split('.').next().unwrap_or("");

        if RE_FILE_AUTH.is_match(file_stem) {
            let route = if parent_route.is_empty() { "/" } else { parent_route };

            let route_auth = Self {
                path: file_path,
                route: route.to_string(),
            };

            return Some(route_auth);
        }

        None
    }
}

impl PrintRoute for RouteAuth {
    fn println(&self) {
        println!("✔️ Built AUTH routes for {}", self.route);
    }
}