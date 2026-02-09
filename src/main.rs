use anyhow::Result;
use tracing::info;

mod config;
mod recorder;
mod storage;
mod supervisor;

use supervisor::watchdog::monitor_camera;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cfg: config::Config = {
        let config_path = find_config_path()?;
        let raw = std::fs::read_to_string(&config_path)?;
        serde_yaml::from_str(&raw)?
    };

    info!("Starting NVR with {} camera(s)", cfg.cameras.len());

    for cam in &cfg.cameras {
        monitor_camera(cam.clone());
    }

    let web_addr = std::env::var("SKYBASE_WEB_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let web_root = std::env::var("SKYBASE_WEB_ROOT").unwrap_or_else(|_| "web".to_string());
    let recordings_root = std::env::var("SKYBASE_RECORDINGS_ROOT")
        .unwrap_or_else(|_| "/mnt/recordings".to_string());

    tokio::spawn(async move {
        if let Err(err) = run_web_server(&web_addr, &web_root, &recordings_root).await {
            tracing::error!("Web server failed: {}", err);
        }
    });

    // Daemon alive houden
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}

async fn run_web_server(addr: &str, web_root: &str, recordings_root: &str) -> Result<()> {
    let recordings_service = ServeDir::new(recordings_root);
    let web_service = ServeDir::new(web_root).append_index_html_on_directories(true);

    let app = axum::Router::new()
        .nest_service("/recordings", recordings_service)
        .fallback_service(web_service);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Web UI listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

fn find_config_path() -> Result<std::path::PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("SKYBASE_CONFIG") {
        candidates.push(std::path::PathBuf::from(path));
    }

    candidates.push(std::path::PathBuf::from("config.yaml"));
    candidates.push(std::path::PathBuf::from("src/config.yaml"));

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!(
        "Config file not found. Set SKYBASE_CONFIG or place config.yaml in the project root."
    ))
}
