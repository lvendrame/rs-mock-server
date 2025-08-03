use std::ffi::OsString;

use http::Method;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{handlers::build_method_router, route_builder::{method_from_str, route_params::RouteParams, PrintRoute, Route, RouteGenerator, RouteRegistrator}};

static RE_FILE_METHODS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?(get|post|put|patch|delete|options)(\{(.+)\})?$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_METHOD: usize = 2;
const ELEMENT_DESCRIPTOR: usize = 4;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum SubRoute {
    #[default]
    None,
    Id,
    Range(u32, u32),
    Static(String)
}

impl SubRoute {
    pub fn from(pattern: Option<regex::Match<'_>>) -> Self {
        if pattern.is_none() {
            return Self::None;
        }

        let pattern = pattern.unwrap().as_str();
        if pattern == "id" {
            return Self::Id;
        }

        if pattern.contains('-') {
            if let Some((start_str, end_str)) = pattern.split_once('-') {
                if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
                    return Self::Range(start, end);
                }
            }
        }

        Self::Static(pattern.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteBasic {
    pub path: OsString,
    pub method: Method,
    pub route: String,
    pub sub_route: SubRoute,
    pub is_protected: bool,
}

impl RouteBasic {
    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_FILE_METHODS.captures(&route_params.file_stem) {
            let is_protected = route_params.is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let method = captures.get(ELEMENT_METHOD).unwrap().as_str();
            let pattern = captures.get(ELEMENT_DESCRIPTOR);

            let route_basic = Self {
                path: route_params.file_path,
                method: method_from_str(method),
                route: route_params.full_route,
                sub_route: SubRoute::from(pattern),
                is_protected,
            };

            return Route::Basic(route_basic);
        }

        Route::None
    }
}

impl RouteGenerator for RouteBasic {
    fn make_routes(&self, app: &mut crate::app::App) {
        let method = self.method.as_str();

        match &self.sub_route {
            SubRoute::None => {
                let router = build_method_router(&self.path, method);
                app.push_route(&self.route, router, Some(method), self.is_protected);
            },
            SubRoute::Id => {
                let route_path = format!("{}/{}", self.route, "{id}");
                let router = build_method_router(&self.path, method);
                app.push_route(&route_path, router, Some(method), self.is_protected);
            },
            SubRoute::Range(start, end) => {
                for i in *start..=*end {
                    let route_path = format!("{}/{}", self.route, i);
                    let router = build_method_router(&self.path, method);
                    app.push_route(&route_path, router, Some(method), self.is_protected);
                }
            },
            SubRoute::Static(end_point) => {
                let route_path = format!("{}/{}", self.route, end_point);
                let router = build_method_router(&self.path, method);
                app.push_route(&route_path, router, Some(method), self.is_protected);
            },
        }

    }
}

impl PrintRoute for RouteBasic {
    fn println(&self) {
        let path = &self.path.to_string_lossy();
        let method = self.method.as_str();
        let route = &self.route;
        match self.sub_route {
            SubRoute::Range(start, end) => println!("✔️ Mapped {} to {} {}/[{}-{}]", path, method, route, start, end),
            _ => println!("✔️ Mapped {} to {} {}", path, method, route),
        }
    }
}
