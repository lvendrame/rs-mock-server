use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;

pub mod route_builder;
pub mod handlers;
pub mod id_manager;
pub mod app;
pub mod link;
pub mod pages;
pub mod build_dyn_routes;
pub mod upload_configuration;
pub mod utils;

/// rs-mock-server is a simple mock server for testing APIs.
/// It serves static files as API responses based on their filenames and directory structure.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value_t = 4520,)]
    port: u16,

    /// Directory to load mock files from
    #[arg(short, long, default_value = "mocks")]
    folder: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_testing=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let mut app = app::App::new(args.port, args.folder);

    let main_logic = app.initialize();

    tokio::select! {
        // Wait for the main logic to complete (which it won't in this case)
        _ = main_logic => {
            app.finish();
        },
        // Wait for the Ctrl+C signal
        _ = signal::ctrl_c() => {
            app.finish();
        }
    }
}
