use crate::{
    handlers::make_auth_middleware,
    app::App,
};

use axum::{middleware, routing::MethodRouter};


pub fn try_add_auth_middleware_layer(app: &mut App, router: MethodRouter, is_protected: bool) -> MethodRouter {
    if !is_protected {
        return router;
    }

    if let Some(auth_collection) = &app.auth_collection {
        return router.layer(
            middleware::from_fn(make_auth_middleware(auth_collection))
        )
    }
    router
}
