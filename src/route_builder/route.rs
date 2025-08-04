use std::cmp::Ordering;

use crate::route_builder::{
    RouteParams,
    PrintRoute,
    RouteAuth,
    RouteBasic,
    RouteGenerator,
    RoutePublic,
    RouteRest,
    RouteUpload
};

#[derive(Debug, Default, PartialEq)]
pub enum Route {
    #[default]
    None,
    Auth(RouteAuth),
    Basic(RouteBasic),
    Rest(RouteRest),
    Public(RoutePublic),
    Upload(RouteUpload),
}

impl Route {
    pub fn is_none(&self) -> bool {
        *self == Route::None
    }

    pub fn is_some(&self) -> bool {
        *self != Route::None
    }

    pub fn try_parse(route_params: &RouteParams) -> Route {
        if route_params.file_name.starts_with(".") {
            return Route::None;
        }

        if route_params.is_dir {
            let route = RoutePublic::try_parse(route_params.clone());
            if route.is_some() {
                return route;
            }

            let route = RouteUpload::try_parse(route_params.clone());
            if route.is_some() {
                return route;
            }

            return Route::None;
        }

        let route = RouteBasic::try_parse(route_params.clone());
        if route.is_some() {
            return route;
        }

        let route = RouteRest::try_parse(route_params.clone());
        if route.is_some() {
            return route;
        }

        let route = RouteAuth::try_parse(route_params.clone());
        if route.is_some() {
            return route;
        }

        Route::None
    }
}

impl RouteGenerator for Route {
    fn make_routes(&self, app: &mut crate::app::App) {
        match self {
            Route::None => (),
            Route::Auth(route_auth) => route_auth.make_routes(app),
            Route::Basic(route_basic) => route_basic.make_routes(app),
            Route::Public(route_public) => route_public.make_routes(app),
            Route::Rest(route_rest) => route_rest.make_routes(app),
            Route::Upload(route_upload) => route_upload.make_routes(app),
        }
    }
}

impl PrintRoute for Route {
    fn println(&self) {
        match self {
            Route::None => (),
            Route::Auth(route_auth) => route_auth.println(),
            Route::Basic(route_basic) => route_basic.println(),
            Route::Public(route_public) => route_public.println(),
            Route::Rest(route_rest) => route_rest.println(),
            Route::Upload(route_upload) => route_upload.println(),
        }
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {

        // First compare by enum discriminant order
        let self_order = match self {
            Route::None => 0,
            Route::Auth(_) => 1,
            Route::Basic(_) => 2,
            Route::Rest(_) => 3,
            Route::Public(_) => 4,
            Route::Upload(_) => 5,
        };
        let other_order = match other {
            Route::None => 0,
            Route::Auth(_) => 1,
            Route::Basic(_) => 2,
            Route::Rest(_) => 3,
            Route::Public(_) => 4,
            Route::Upload(_) => 5,
        };

        match self_order.cmp(&other_order) {
            Ordering::Equal => {
                // Same enum variant, compare by path then method
                match (self, other) {
                    (Route::None, Route::None) => Some(Ordering::Equal),
                    (Route::Auth(a), Route::Auth(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    (Route::Basic(a), Route::Basic(b)) => {
                        match a.path.cmp(&b.path) {
                            Ordering::Equal => a.method.to_string().partial_cmp(&b.method.to_string()),
                            other => Some(other),
                        }
                    },
                    (Route::Rest(a), Route::Rest(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    (Route::Public(a), Route::Public(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    (Route::Upload(a), Route::Upload(b)) => {
                        a.path.partial_cmp(&b.path)
                    },
                    _ => unreachable!(),
                }
            },
            other => Some(other),
        }
    }
}
