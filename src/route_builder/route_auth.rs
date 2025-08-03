use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::route_builder::{route_params::RouteParams, PrintRoute};

static RE_FILE_AUTH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\{auth\}$").unwrap()
});

pub struct  RouteAuth {
    pub path: OsString,
    pub route: String,
}

impl RouteAuth {
    pub fn try_parse(route_params: RouteParams) -> Option<Self> {
        if RE_FILE_AUTH.is_match(&route_params.file_stem) {

            let route_auth = Self {
                path: route_params.file_path,
                route: route_params.full_route,
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