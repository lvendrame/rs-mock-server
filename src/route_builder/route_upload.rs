use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::route_builder::{PrintRoute};

static RE_DIR_UPLOAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?\{upload\}(\{temp\})?(-(.+))?$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_IS_TEMPORARY: usize = 2;
const ELEMENT_ROUTE: usize = 4;

pub struct  RouteUpload {
    pub path: OsString,
    pub route: String,
    pub is_temporary: bool,
    pub is_protected: bool,
}

impl RouteUpload {
    pub fn try_parse(parent_route: &str, file_name: String, file_path: OsString, is_protected: bool) -> Option<Self> {
        if let Some(captures) = RE_DIR_UPLOAD.captures(&file_name) {
            let is_protected = is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let is_temporary = captures.get(ELEMENT_IS_TEMPORARY).is_some();
            let uploads_route = if let Some(route) = captures.get(ELEMENT_ROUTE) {
                route.as_str()
            } else {
                "upload"
            };

            let route = if parent_route.is_empty() { "/" } else { parent_route };
            let route = format!("{}/{}", route, uploads_route);

            let route_upload = Self {
                path: file_path,
                route: route.to_string(),
                is_temporary,
                is_protected,
            };

            return Some(route_upload);
        }

        None
    }
}

impl PrintRoute for RouteUpload {
    fn println(&self) {
        println!("✔️ Mapped uploads from folder {} to /{}", self.path.to_string_lossy(), self.route);
    }
}