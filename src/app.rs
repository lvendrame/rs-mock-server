use std::{
    cell::RefCell,
    ffi::OsString,
    io::Write,
    sync::{Arc, Mutex, RwLock},
};

use axum::{
    Router, middleware,
    response::IntoResponse,
    routing::{MethodRouter, Route, get},
};
use fosk::Db;
use http::{HeaderMap, HeaderValue, StatusCode, header::CONTENT_TYPE};
use terminal_link::Link;
use tokio::net::TcpListener;
use tower::{
    Layer, ServiceBuilder,
    layer::util::{Identity, Stack},
};
use tower_http::{
    cors::CorsLayer, normalize_path::NormalizePathLayer, services::ServeDir, trace::TraceLayer,
};

use crate::{
    DEFAULT_FOLDER, DEFAULT_PORT,
    handlers::{create_collections_routes, make_auth_middleware},
    pages::Pages,
    route_builder::{
        RouteGenerator, RouteRegistrator,
        config::{Config, ServerConfig},
        route_manager::RouteManager,
    },
    upload_configuration::UploadConfiguration,
};

#[derive(Default)]
pub struct GlobalSharedInfo {
    pub jwt_secret: String,
    pub token_collection: String,
    pub auth_cookie_name: String,
}

pub const MOCK_SERVER_ROUTE: &str = "/mock-server";
pub static GLOBAL_SHARED_INFO: RwLock<GlobalSharedInfo> = RwLock::new(GlobalSharedInfo {
    jwt_secret: String::new(),
    token_collection: String::new(),
    auth_cookie_name: String::new(),
});

pub struct App {
    pub router: RefCell<Router>,
    pub pages: Arc<Mutex<Pages>>,
    uploads_configurations: Vec<UploadConfiguration>,
    pub db: Arc<Db>,
    pub server_config: Config,
}

impl Default for App {
    fn default() -> Self {
        let router = RefCell::new(Router::new());
        let pages = Arc::new(Mutex::new(Pages::new()));
        let uploads_configurations = vec![];
        let db = Db::new_arc();
        let server_config = Config {
            server: Some(ServerConfig {
                folder: Some(DEFAULT_FOLDER.into()),
                port: Some(DEFAULT_PORT),
                ..Default::default()
            }),
            ..Default::default()
        };
        App {
            router,
            pages,
            uploads_configurations,
            db,
            server_config,
        }
    }
}

impl App {
    pub fn new(server_config: Config) -> Self {
        let router = RefCell::new(Router::new());
        let pages = Arc::new(Mutex::new(Pages::new()));
        let uploads_configurations = vec![];
        let db = Db::new_arc();
        App {
            router,
            pages,
            uploads_configurations,
            db,
            server_config,
        }
    }

    pub fn get_folder(&self) -> String {
        self.server_config
            .server
            .as_ref()
            .unwrap_or(&ServerConfig::default())
            .folder
            .clone()
            .unwrap_or(DEFAULT_FOLDER.to_string())
    }

    pub fn get_port(&self) -> u16 {
        self.server_config
            .server
            .as_ref()
            .unwrap_or(&ServerConfig::default())
            .port
            .unwrap_or(DEFAULT_PORT)
    }

    pub fn push_uploads_config(&mut self, uploads_path: String, clean_uploads: bool) {
        self.uploads_configurations
            .push(UploadConfiguration::new(uploads_path, clean_uploads));
    }

    fn get_router(&self) -> Router {
        self.router.take()
    }

    #[cfg(test)]
    pub(crate) fn take_router_for_test(&self) -> Router {
        self.get_router()
    }

    fn replace_router(&mut self, new_router: Router) {
        // _old_route object will be dropped (Axum uses builder pattern)
        let _old_route = self.router.replace(new_router);
    }

    pub fn route(
        &mut self,
        path: &str,
        router: MethodRouter<()>,
        method: Option<&str>,
        options: Option<&[String]>,
    ) {
        let new_router = self.get_router().route(path, router);

        self.replace_router(new_router);

        if let Some(method) = method {
            self.pages.lock().unwrap().push_link(
                method.to_string(),
                path.to_string(),
                options.unwrap_or(&Vec::<String>::new()),
            );
        }
    }

    pub fn try_add_auth_middleware_layer(
        &mut self,
        router: MethodRouter,
        is_protected: bool,
    ) -> MethodRouter {
        if !is_protected {
            return router;
        }

        let shared_info = GLOBAL_SHARED_INFO.read().unwrap();
        if let Some(token_collection) = &self.db.get(&shared_info.token_collection) {
            return router.layer(middleware::from_fn(make_auth_middleware(
                token_collection,
                &shared_info.jwt_secret,
                &shared_info.auth_cookie_name,
            )));
        }
        router
    }

    fn build_dyn_routes(&mut self) {
        let dir = self.get_folder();
        RouteManager::from_dir(&dir, Some(self.server_config.clone())).make_routes(self);
    }

    fn build_home_route(&mut self) {
        let pages = Arc::clone(&self.pages);

        self.route(
            "/",
            get(|| async move {
                let body = pages.lock().unwrap().render_index();
                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());
                headers.insert(
                    "Cache-Control",
                    HeaderValue::from_str("no-cache, no-store, must-revalidate").unwrap(),
                );
                headers.insert("Pragma", HeaderValue::from_str("no-cache").unwrap());
                headers.insert("Expires", HeaderValue::from_str("0").unwrap());

                (headers, body).into_response()
            }),
            None,
            None,
        );
    }

    fn build_cors_layer<L>(
        &self,
        service_builder: ServiceBuilder<L>,
    ) -> ServiceBuilder<Stack<tower::util::Either<CorsLayer, Identity>, L>>
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        Stack<CorsLayer, L>: Layer<Route> + Clone + Send + Sync + 'static,
    {
        let server_config = self.server_config.server.clone().unwrap_or_default();
        let enable = server_config.enable_cors.unwrap_or(true);
        let allowed_origin = server_config.allowed_origin;

        service_builder.option_layer(enable.then(|| {
            if let Some(allowed_origin) = allowed_origin {
                CorsLayer::very_permissive()
                    .allow_origin(allowed_origin.parse::<HeaderValue>().unwrap())
            } else {
                CorsLayer::very_permissive()
            }
        }))
    }

    fn build_middlewares(&mut self) {
        let service_builder = ServiceBuilder::new().layer(TraceLayer::new_for_http());

        let service_builder = self.build_cors_layer(service_builder);

        let service_builder = service_builder.layer(NormalizePathLayer::trim_trailing_slash());

        let new_router = self.get_router().layer(service_builder);

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
        let public_end_point = if let Some((_, to)) = file_name.split_once('-') {
            to
        } else {
            "public"
        };

        let static_files = ServeDir::new(path);
        let new_router = self
            .router
            .take()
            .nest_service(&format!("/{}", public_end_point), static_files);
        self.replace_router(new_router);
    }

    pub fn build_public_router_v2(&mut self, path: &OsString, route: &str) {
        let static_files = ServeDir::new(path);
        let new_router = self.router.take().nest_service(route, static_files);
        self.replace_router(new_router);
    }

    pub fn build_collections_route(&mut self) {
        create_collections_routes(self);
    }

    pub fn build_collections_references(&mut self) {
        let collections = self.db.list_collections();

        if collections.len() > 1 {
            for i in 0..collections.len() - 1 {
                for j in i + 1..collections.len() {
                    self.db.infer_reference(&collections[i], &collections[j]);
                    self.db.infer_reference(&collections[j], &collections[i]);
                }
            }
        }
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
        let address = format!("0.0.0.0:{}", self.get_port());

        let listener = TcpListener::bind(address.clone()).await.unwrap();
        App::show_greetings();

        let link = format!("http://localhost:{}", self.get_port());
        let link = Link::new(&link, &link);
        println!("🚀 Listening on {}", link);

        axum::serve(listener, self.get_router()).await.unwrap();
    }

    pub async fn initialize(&mut self) {
        self.build_dyn_routes();
        self.build_home_route();
        self.build_collections_route();
        self.build_fallback();
        self.build_middlewares();
        self.build_collections_references();
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
        self.db.clear();

        println!("\n👋👋👋👋👋 Goodbye! 👋👋👋👋👋👋");
    }
}

impl RouteRegistrator for App {
    fn push_route(
        &mut self,
        path: &str,
        router: MethodRouter,
        method: Option<&str>,
        is_protected: bool,
        options: Option<&[String]>,
    ) {
        let router = self.try_add_auth_middleware_layer(router, is_protected);

        self.route(path, router, method, options);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
        routing::get,
    };
    use tower::ServiceExt;

    fn config(folder: Option<&str>, port: Option<u16>) -> Config {
        Config {
            server: Some(ServerConfig {
                folder: folder.map(str::to_string),
                port,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn default_and_custom_config_values_are_resolved() {
        let default_app = App::default();
        assert_eq!(default_app.get_folder(), DEFAULT_FOLDER);
        assert_eq!(default_app.get_port(), DEFAULT_PORT);

        let custom_app = App::new(config(Some("fixtures"), Some(9876)));
        assert_eq!(custom_app.get_folder(), "fixtures");
        assert_eq!(custom_app.get_port(), 9876);
    }

    #[tokio::test]
    async fn route_registers_page_link_and_serves_handler() {
        let mut app = App::default();
        app.route(
            "/ping",
            get(|| async { "pong" }),
            Some("GET"),
            Some(&["json".to_string()]),
        );

        let html = app.pages.lock().unwrap().render_index();
        assert!(html.contains("/ping"));
        assert!(html.contains("GET"));

        let response = app
            .take_router_for_test()
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX).await.unwrap(),
            "pong"
        );
    }

    #[tokio::test]
    async fn unprotected_auth_layer_returns_original_router() {
        let mut app = App::default();
        app.push_route("/open", get(|| async { "ok" }), Some("GET"), false, None);

        let response = app
            .take_router_for_test()
            .oneshot(Request::builder().uri("/open").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn protected_auth_layer_without_token_collection_leaves_route_open() {
        let mut app = App::default();
        {
            let mut shared_info = GLOBAL_SHARED_INFO.write().unwrap();
            shared_info.token_collection = "tokens".to_string();
            shared_info.jwt_secret = "secret".to_string();
            shared_info.auth_cookie_name = "auth".to_string();
        }
        app.push_route(
            "/protected",
            get(|| async { "ok" }),
            Some("GET"),
            true,
            None,
        );

        let response = app
            .take_router_for_test()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn home_fallback_public_and_middlewares_are_built() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("asset.txt"), "asset").unwrap();

        let mut app = App::new(Config {
            server: Some(ServerConfig {
                enable_cors: Some(true),
                allowed_origin: Some("http://example.com".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        });
        app.pages
            .lock()
            .unwrap()
            .push_link("GET".to_string(), "/x".to_string(), &[]);
        app.build_home_route();
        app.build_public_router(
            "public-assets".to_string(),
            temp_dir.path().to_string_lossy().to_string(),
        );
        app.build_fallback();
        app.build_middlewares();

        let router = app.take_router_for_test();
        let home = router
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(home.status(), StatusCode::OK);
        assert_eq!(home.headers().get(CONTENT_TYPE).unwrap(), "text/html");

        let public = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/assets/asset.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(public.status(), StatusCode::OK);

        let fallback = router
            .oneshot(
                Request::builder()
                    .uri("/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(fallback.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn cors_can_be_disabled_and_public_v2_can_mount_route() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("file.txt"), "file").unwrap();

        let mut app = App::new(Config {
            server: Some(ServerConfig {
                enable_cors: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        });
        app.build_public_router_v2(&temp_dir.path().as_os_str().to_os_string(), "/static");
        app.build_middlewares();

        let response = app
            .take_router_for_test()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/static/file.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn collections_references_and_finish_reset_state() {
        let mut app = App::default();
        app.db.create("users");
        app.db.create("orders");
        app.build_collections_references();
        app.build_collections_route();
        App::show_greetings();
        app.push_uploads_config("missing-folder".to_string(), false);
        app.finish();

        assert!(app.db.list_collections().is_empty());
        assert!(
            app.pages
                .lock()
                .unwrap()
                .render_index()
                .contains("mock_routes = []")
        );
    }
}
