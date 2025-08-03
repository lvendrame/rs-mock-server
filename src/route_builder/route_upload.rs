use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{handlers::build_upload_routes, route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}};

static RE_DIR_UPLOAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?\{upload\}(\{temp\})?(-(.+))?$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_IS_TEMPORARY: usize = 2;
const ELEMENT_ROUTE: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct  RouteUpload {
    pub path: OsString,
    pub route: String,
    pub is_temporary: bool,
    pub is_protected: bool,
}

impl RouteUpload {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_DIR_UPLOAD.captures(&route_params.file_name) {
            let is_protected = route_params.is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let is_temporary = captures.get(ELEMENT_IS_TEMPORARY).is_some();
            let uploads_route = if let Some(route) = captures.get(ELEMENT_ROUTE) {
                route.as_str()
            } else {
                "upload"
            };

            let route = format!("{}/{}", route_params.parent_route, uploads_route);

            let route_upload = Self {
                path: route_params.file_path,
                route: route.to_string(),
                is_temporary,
                is_protected,
            };

            return Route::Upload(route_upload);
        }

        Route::None
    }
}

impl RouteGenerator for RouteUpload {
    fn make_routes(&self, app: &mut crate::app::App) {
        let path = self.path.to_string_lossy();
        app.push_uploads_config(path.to_string(), self.is_temporary);

        build_upload_routes(app, path.to_string(), &self.route);
    }
}

impl PrintRoute for RouteUpload {
    fn println(&self) {
        println!("✔️ Mapped uploads from folder {} to /{}", self.path.to_string_lossy(), self.route);
    }
}