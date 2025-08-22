use std::{cell::RefCell, ffi::OsString, io::Write, sync::{Arc, Mutex}};

use axum::{
    middleware, response::IntoResponse, routing::{get, MethodRouter}, Router
};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode};
use terminal_link::Link;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, normalize_path::NormalizePathLayer, services::ServeDir, trace::TraceLayer};

use crate::{handlers::make_auth_middleware,
    memory_db::in_memory_collection::ProtectedMemCollection,
    pages::Pages,
    route_builder::{route_manager::RouteManager, RouteGenerator, RouteRegistrator},
    upload_configuration::UploadConfiguration
};

pub struct App {
    pub port: u16,
    pub root_path: String,
    pub router: RefCell<Router>,
    pub pages: Arc<Mutex<Pages>>,
    pub auth_collection: Option<ProtectedMemCollection>,
    uploads_configurations: Vec<UploadConfiguration>,
}

impl Default for App {
    fn default() -> Self {
        let port = 3000;
        let root_path = String::from("/");
        let router = RefCell::new(Router::new());
        let pages = Arc::new(Mutex::new(Pages::new()));
        let uploads_configurations = vec![];
        let auth_collection = None;
        App { port, root_path, router, pages, uploads_configurations, auth_collection }
    }
}

impl App {

    pub fn new(port: u16, root_path: String) -> Self {
        let router = RefCell::new(Router::new());
        let pages = Arc::new(Mutex::new(Pages::new()));
        let uploads_configurations = vec![];
        let auth_collection = None;
        App { port, root_path, router, pages, uploads_configurations, auth_collection }
    }

    pub fn push_uploads_config(&mut self, uploads_path: String, clean_uploads: bool) {
        self.uploads_configurations.push(
            UploadConfiguration::new(uploads_path, clean_uploads)
        );
    }

    fn get_router(&self) -> Router {
        self.router.take()
    }

    fn replace_router(&mut self, new_router: Router) {
        // _old_route object will be dropped (Axum uses builder pattern)
        let _old_route = self.router.replace(new_router);
    }

    pub fn route(&mut self, path: &str, router: MethodRouter<()>, method: Option<&str>, options: Option<&[String]>) {
        let new_router = self.get_router().route(path, router);

        self.replace_router(new_router);

        if let Some(method) = method {
            self.pages.lock().unwrap().push_link(method.to_string(), path.to_string(), options.unwrap_or(&Vec::<String>::new()));
        }
    }

    pub fn try_add_auth_middleware_layer(&mut self, router: MethodRouter, is_protected: bool) -> MethodRouter {
        if !is_protected {
            return router;
        }

        if let Some(auth_collection) = &self.auth_collection {
            return router.layer(
                middleware::from_fn(make_auth_middleware(auth_collection))
            )
        }
        router
    }

    fn build_dyn_routes(&mut self) {
        RouteManager::from_dir(&self.root_path)
            .make_routes(self);
    }

    fn build_home_route(&mut self) {
        let pages = Arc::clone(&self.pages);

        self
            .route("/", get(|| async move {
                let body = pages.lock().unwrap().render_index();
                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());
                headers.insert("Cache-Control", HeaderValue::from_str("no-cache, no-store, must-revalidate").unwrap());
                headers.insert("Pragma", HeaderValue::from_str("no-cache").unwrap());
                headers.insert("Expires", HeaderValue::from_str("0").unwrap());

                (headers, body).into_response()
            }), None, None);
    }

    fn build_middlewares(&mut self) {
        let new_router = self.get_router().layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::very_permissive())
            .layer(NormalizePathLayer::trim_trailing_slash())
        );

        self.replace_router(new_router);
    }

    fn build_fallback(&mut self) {
        let new_router = self.get_router().fallback(Self::handler_404);
        self.replace_router(new_router);
    }

    async fn handler_404() -> impl IntoResponse {
        (StatusCode::NOT_FOUND, "nothing to see here")
    }

    pub fn build_public_router(&mut self, file_name: String, path: String) {
        let  public_end_point = if let Some((_, to)) = file_name.split_once('-') {
            to
        } else {
            "public"
        };

        let static_files = ServeDir::new(path);
        let new_router = self.router.take().nest_service(
            &format!("/{}", public_end_point),
            static_files
        );
        self.replace_router(new_router);
    }

    pub fn build_public_router_v2(&mut self, path: &OsString, route: &str) {
        let static_files = ServeDir::new(path);
        let new_router = self.router.take().nest_service(
            route,
            static_files
        );
        self.replace_router(new_router);
    }

    pub fn show_greetings() {
        let banner = r"
                                  ___     ___
                                 (o o)   (o o)
 _____                          (  V  ) (  V  )                         _____
( ___ )------------------------ /--m-m- /--m-m-------------------------( ___ )
 |   |                                                                  |   |
 |   |                                                                  |   |
 |   |     ░█▀▄░█▀▀░░░░░█▄█░█▀█░█▀▀░█░█░░░░░█▀▀░█▀▀░█▀▄░█░█░█▀▀░█▀▄     |   |
 |   |     ░█▀▄░▀▀█░▄▄▄░█░█░█░█░█░░░█▀▄░▄▄▄░▀▀█░█▀▀░█▀▄░▀▄▀░█▀▀░█▀▄     |   |
 |   |     ░▀░▀░▀▀▀░░░░░▀░▀░▀▀▀░▀▀▀░▀░▀░░░░░▀▀▀░▀▀▀░▀░▀░░▀░░▀▀▀░▀░▀     |   |
 |   |                                                                  |   |
 |   |                             {{{{}}}}                             |   |
 |___|                                                                  |___|
(_____)----------------------------------------------------------------(_____)

";

        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let version = format!("{:^8}", version);
        let _ = std::io::stdout().write_all(banner.replace("{{{{}}}}", &version).as_bytes());
    }

    async fn start_server(&self) {
        let address = format!("0.0.0.0:{}", self.port);

        let listener = TcpListener::bind(address.clone()).await.unwrap();
        App::show_greetings();

        let link = format!("http://localhost:{}", self.port);
        let link = Link::new(&link, &link);
        println!("🚀 Listening on {}", link);

        axum::serve(listener, self.get_router()).await.unwrap();
    }

    pub async fn initialize(&mut self) {
        self.build_dyn_routes();
        self.build_home_route();
        self.build_fallback();
        self.build_middlewares();
        self.start_server().await;
    }

    pub fn finish(&mut self) {
        println!("\n");

        for upload_config in self.uploads_configurations.iter() {
            upload_config.clean_upload_folder();
        }

        self.router = RefCell::new(Router::new());
        self.pages = Arc::new(Mutex::new(Pages::new()));
        self.uploads_configurations = vec![];
        self.auth_collection = None;

        println!("\n👋👋👋👋👋 Goodbye! 👋👋👋👋👋👋");
    }
}

impl RouteRegistrator for App {
    fn push_route(&mut self, path: &str, router: MethodRouter, method: Option<&str>, is_protected: bool, options: Option<&[String]>) {
        let router = self.try_add_auth_middleware_layer(router, is_protected);

        self.route(path, router, method, options);
    }
}
