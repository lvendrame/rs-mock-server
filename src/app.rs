use std::{cell::RefCell};

use axum::{
    response::IntoResponse, routing::{get, MethodRouter},
    Router
};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, normalize_path::NormalizePathLayer, services::ServeDir, trace::TraceLayer};

use crate::{build_dyn_routes::load_mock_dir, pages::Pages, upload_configuration::UploadConfiguration};

pub struct App {
    pub port: u16,
    pub root_path: String,
    pub router: RefCell<Router>,
    pub pages: Pages,
    uploads_configurations: Vec<UploadConfiguration>,
}

impl Default for App {
    fn default() -> Self {
        let port = 3000;
        let root_path = String::from("/");
        let router = RefCell::new(Router::new());
        let pages = Pages::new();
        let uploads_configurations = vec![];
        App { port, root_path, router, pages, uploads_configurations }
    }
}

impl App {

    pub fn new(port: u16, root_path: String) -> Self {
        let routes = RefCell::new(Router::new());
        let pages = Pages::new();
        let uploads_configurations = vec![];
        App { port, root_path, router: routes, pages, uploads_configurations }
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

    pub fn route(&mut self, path: &str, method_router: MethodRouter<()>, method: Option<&str>) {
        let new_router = self.get_router().route(path, method_router);

        self.replace_router(new_router);

        if let Some(method) = method {
            self.pages.push_link(method.to_string(), path.to_string());
        }
    }

    fn build_dyn_routes(&mut self) {
        load_mock_dir(self);
    }

    fn build_index_routes(&mut self) {
        let index = self.pages.render_index();

        self
            .route("/", get(|| async {
                let body = index;
                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());

                (headers, body).into_response()
            }), None);

        let home = String::from(self.pages.home_template);

        self.route("/home-api", get(|| async {
            let body = home;
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());

            (headers, body).into_response()
        }), None);
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

    async fn start_server(&self) {
        let address = format!("0.0.0.0:{}", self.port);

        let listener = TcpListener::bind(address.clone()).await.unwrap();
        println!("🚀 RS-Mock Server listening on {}", address);

        axum::serve(listener, self.get_router()).await.unwrap();
    }

    pub async fn initialize(&mut self) {
        self.build_dyn_routes();
        self.build_index_routes();
        self.build_fallback();
        self.build_middlewares();
        self.start_server().await;
    }

    pub fn finish(&self) {
        println!("\n");

        for upload_config in self.uploads_configurations.iter() {
            upload_config.clean_upload_folder();
        }
        println!("\nGoodbye! 👋");
    }
}