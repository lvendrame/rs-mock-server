use axum::{Router, routing::get};
use rs_mock_server::{App, Config, ServerConfig};

async fn index() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>rs-mock-server library example</title>
  </head>
  <body>
    <h1>rs-mock-server library example</h1>
    <ul>
      <li><a href="/host/health">Host health route</a></li>
      <li><a href="/hello">Mock hello route</a></li>
      <li><a href="/users">Mock users REST route</a></li>
      <li><a href="/mock-server">rs-mock-server home</a></li>
    </ul>
  </body>
</html>"#,
    )
}

#[tokio::main]
async fn main() {
    let config = Config {
        server: Some(ServerConfig {
            folder: Some("mocks".into()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mock_routes = App::new(config).into_router();
    let app = Router::new()
        .route("/", get(index))
        .route("/host/health", get(|| async { "host ok" }))
        .merge(mock_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    println!("Example app listening on http://localhost:8080");

    axum::serve(listener, app).await.unwrap();
}
