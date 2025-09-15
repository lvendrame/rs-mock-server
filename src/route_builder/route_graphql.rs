use std::{ffi::OsString};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    app::App,
    handlers::build_graphql_routes,
    route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}
};

static RE_FOLDER_GRAPHQL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?graphql$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct  RouteGraphQL {
    pub path: OsString,
    pub route: String,
    pub delay: Option<u16>,
    pub is_protected: bool,
}

impl RouteGraphQL {
    pub fn new(
        path: OsString,
        route: String,
        is_protected: bool,
        delay: Option<u16>,
    ) -> Self {
        Self { path, route, is_protected, delay }
    }

    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_FOLDER_GRAPHQL.captures(&route_params.file_stem) {
            let config = route_params.config.clone();
            let route_config = config.route.clone().unwrap_or_default();

            let delay = route_config.delay;
            let is_protected = route_config.protect.unwrap_or(false);
            let is_protected = is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();

            let route = route_config.remap.unwrap_or(route_params.full_route);


            let route_graphql = Self {
                path: route_params.file_path,
                route,
                delay,
                is_protected,
            };

            return Route::GraphQL(route_graphql);
        }

        Route::None
    }
}

impl RouteGenerator for RouteGraphQL {
    fn make_routes(&self, app: &mut App) {
        build_graphql_routes(app, self);
    }
}

impl PrintRoute for RouteGraphQL {
    fn println(&self) {
        println!("✔️ Built GraphQL routes");
    }
}
