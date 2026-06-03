use clap::Parser;
use notify::{RecursiveMode, Watcher};
use rs_mock_server::{
    App, Config, DEFAULT_FOLDER, DEFAULT_PORT, ServerConfig, generator::run_generator,
};
use std::time::{Duration, Instant};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use tokio::{signal, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// rs-mock-server is a simple mock server for testing APIs.
/// It serves static files as API responses based on their filenames and directory structure.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Directory to load mock files from
    #[arg(short, long, default_value = DEFAULT_FOLDER)]
    folder: String,

    /// Disable CORS, by default CORS is enabled
    #[arg(short, long)]
    disable_cors: bool,

    /// Allowed origin, by default all origins are allowed
    #[arg(short, long)]
    allowed_origin: Option<String>,

    /// Open the interactive mock file and configuration generator
    #[arg(short, long)]
    generate: bool,

    /// Serve over HTTPS using a generated localhost certificate
    #[arg(long)]
    ssl: bool,

    /// PEM certificate path for HTTPS
    #[arg(long = "ssl-cert")]
    ssl_cert: Option<String>,

    /// PEM private key path for HTTPS
    #[arg(long = "ssl-key")]
    ssl_key: Option<String>,
}

enum SessionResult {
    Restart,
    Shutdown,
}

fn is_upload_folder(folder: &str) -> bool {
    folder.contains("{upload}")
}

async fn run_app_session(config: Config) -> SessionResult {
    let token = CancellationToken::new();
    let app = App::new(config);
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

    tracing::info!(
        "RS-MOCK-SERVER started. Watching for file changes in '{}'...",
        app_arc.lock().await.get_folder()
    );

    let (tx, mut rx) = mpsc::channel(1);
    let last_send_time = Arc::new(Mutex::new(Instant::now() - Duration::from_millis(1000)));
    let debounce_duration = Duration::from_millis(300);

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    notify::EventKind::Create(_) => println!("Event: Create"),
                    notify::EventKind::Modify(_) => println!("Event: Modify"),
                    notify::EventKind::Remove(_) => println!("Event: Remove"),
                    _ => return,
                }

                for path in &event.paths {
                    if is_upload_folder(path.to_str().unwrap()) {
                        // For upload folders, only allow modify events for folders, skip all file events
                        if !path.is_dir() {
                            return;
                        }
                    }
                }
                println!(
                    "event {:?}",
                    event
                        .paths
                        .iter()
                        .map(|f| f.to_str().unwrap_or(""))
                        .collect::<Vec<&str>>()
                        .join("|")
                );

                // Simple debouncing - only send if enough time has passed since last send
                let now = std::time::Instant::now();
                let mut last_time = last_send_time.blocking_lock();

                if now.duration_since(*last_time) >= debounce_duration {
                    *last_time = now;
                    let _ = tx.blocking_send(());
                }
            }
        })
        .unwrap();

    watcher
        .watch(
            Path::new(&app_arc.lock().await.get_folder()),
            RecursiveMode::Recursive,
        )
        .unwrap();

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

    if args.generate {
        if let Err(err) = run_generator(&args.folder) {
            eprintln!("Generator failed: {}", err);
        }
        return;
    }

    let config = if let Ok(file) = std::fs::read_to_string("./rs-mock-server.toml") {
        match Config::try_from(file.as_str()) {
            Ok(config) => apply_cli_ssl_config(config, &args),
            Err(err) => {
                println!("Error: {}", err);
                return;
            }
        }
    } else {
        Config {
            server: Some(ServerConfig {
                port: Some(args.port),
                folder: Some(args.folder),
                allowed_origin: args.allowed_origin,
                enable_cors: Some(!args.disable_cors),
                ssl: Some(args.ssl).filter(|enabled| *enabled),
                ssl_cert: args.ssl_cert,
                ssl_key: args.ssl_key,
            }),
            ..Default::default()
        }
    };

    while let SessionResult::Restart = run_app_session(config.clone()).await {
        // Small delay before restarting
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn apply_cli_ssl_config(mut config: Config, args: &Args) -> Config {
    if !args.ssl && args.ssl_cert.is_none() && args.ssl_key.is_none() {
        return config;
    }

    let mut server = config.server.unwrap_or_default();
    server.ssl = Some(args.ssl).filter(|enabled| *enabled).or(server.ssl);
    server.ssl_cert = args.ssl_cert.clone().or(server.ssl_cert);
    server.ssl_key = args.ssl_key.clone().or(server.ssl_key);
    config.server = Some(server);

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_folder_detection_matches_upload_marker() {
        assert!(is_upload_folder("mocks/{upload}"));
        assert!(is_upload_folder("mocks/${upload}{temp}"));
        assert!(!is_upload_folder("mocks/uploads"));
    }

    #[test]
    fn cli_ssl_options_overlay_file_config() {
        let args = Args::parse_from([
            "rs-mock-server",
            "--ssl-cert",
            "localhost.pem",
            "--ssl-key",
            "localhost-key.pem",
        ]);
        let config = Config {
            server: Some(ServerConfig {
                port: Some(9876),
                ..Default::default()
            }),
            ..Default::default()
        };

        let config = apply_cli_ssl_config(config, &args);
        let server = config.server.unwrap();

        assert_eq!(server.port, Some(9876));
        assert_eq!(server.ssl_cert, Some("localhost.pem".into()));
        assert_eq!(server.ssl_key, Some("localhost-key.pem".into()));
    }
}
