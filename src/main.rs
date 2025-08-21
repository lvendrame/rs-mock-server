use clap::Parser;
use notify::{RecursiveMode, Watcher};
use tokio::sync::Mutex;
use std::{path::Path, sync::Arc};
use std::time::{Duration, Instant};
use tokio::{signal, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::app::App;

pub mod in_memory_collection;
pub mod route_builder;
pub mod handlers;
pub mod id_manager;
pub mod app;
pub mod link;
pub mod pages;
pub mod upload_configuration;

/// rs-mock-server is a simple mock server for testing APIs.
/// It serves static files as API responses based on their filenames and directory structure.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value_t = 4520)]
    port: u16,

    /// Directory to load mock files from
    #[arg(short, long, default_value = "mocks")]
    folder: String,
}

enum SessionResult {
    Restart,
    Shutdown,
}

fn is_upload_folder(folder: &str) -> bool {
    folder.contains("{upload}")
}

async fn run_app_session(args: &Args) -> SessionResult {
    let token = CancellationToken::new();
    let app = App::new(args.port, args.folder.clone());
    let app_arc = Arc::new(Mutex::new(app));

    let main_logic = {
        let app_ref = Arc::clone(&app_arc);
        async move {
            let mut app = app_ref.lock().await;
            app.initialize().await
        }
    };

    let app_finisher_task = tokio::spawn({
        let token_clone = token.clone();
        let app_ref = Arc::clone(&app_arc);
        async move {
            token_clone.cancelled().await;
            let mut app = app_ref.lock().await;
            app.finish();
        }
    });

    tracing::info!("RS-MOCK-SERVER started. Watching for file changes in '{}'...", &args.folder);

    let (tx, mut rx) = mpsc::channel(1);
    let last_send_time = Arc::new(Mutex::new(Instant::now() - Duration::from_millis(1000)));
    let debounce_duration = Duration::from_millis(300);

    let mut watcher = notify::recommended_watcher(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                for path in &event.paths {
                    if is_upload_folder(path.to_str().unwrap()) {
                        // For upload folders, only allow modify events for folders, skip all file events
                        if !path.is_dir() {
                            return;
                        }
                    }
                }
                println!("event {:?}", event.paths.iter().map(|f|f.to_str().unwrap_or("")).collect::<Vec<&str>>().join("|"));

                // Simple debouncing - only send if enough time has passed since last send
                let now = std::time::Instant::now();
                let mut last_time = last_send_time.blocking_lock();

                if now.duration_since(*last_time) >= debounce_duration {
                    *last_time = now;
                    let _ = tx.blocking_send(());
                }
            }
        }
    ).unwrap();

    watcher.watch(Path::new(&args.folder), RecursiveMode::Recursive).unwrap();

    let result = tokio::select! {
        _ = main_logic => {
            tracing::warn!("Main logic completed unexpectedly. Shutting down.");
            SessionResult::Shutdown
        },
        _ = rx.recv() => {
            tracing::info!("File change detected. Restarting application...");
            SessionResult::Restart
        },
        _ = signal::ctrl_c() => {
            tracing::info!("Ctrl+C received. Shutting down.");
            SessionResult::Shutdown
        }
    };

    token.cancel();
    let _ = app_finisher_task.await;
    tracing::info!("Application instance shut down gracefully.");

    result
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();

    let args = Args::parse();

    while let SessionResult::Restart = run_app_session(&args).await {
        // Small delay before restarting
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
