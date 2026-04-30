use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

mod api;
mod config;
mod recorder;
mod storage;
mod supervisor;

use api::AppState;
use supervisor::watchdog::monitor_camera;
use tower_http::services::ServeDir;
use axum::middleware::Next;
use axum::body::Body;
use axum::http::Request;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cfg: config::Config = {
        let config_path = find_config_path()?;
        let raw = std::fs::read_to_string(&config_path)?;
        serde_yaml::from_str(&raw)?
    };

    let storage_root = std::env::var("SKYBASE_STORAGE_ROOT")
        .or_else(|_| std::env::var("SKYBASE_RECORDINGS_ROOT"))
        .unwrap_or_else(|_| cfg.default_storage_folder.clone());

    info!(
        "Starting NVR with {} camera(s), storage root: {}",
        cfg.cameras.len(),
        storage_root
    );

    let store = api::new_store();

    for cam in &cfg.cameras {
        monitor_camera(resolve_camera_config(cam.clone(), &storage_root), store.clone());
    }

    let web_addr = std::env::var("SKYBASE_WEB_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let web_root = std::env::var("SKYBASE_WEB_ROOT").unwrap_or_else(|_| "web".to_string());
    let recordings_root = storage_root.clone();

    tokio::spawn(async move {
        let state = AppState {
            cameras: store,
            storage_root,
        };
        if let Err(err) = run_web_server(&web_addr, &web_root, &recordings_root, state).await {
            tracing::error!("Web server failed: {}", err);
        }
    });

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}

fn resolve_camera_config(mut cam: config::CameraConfig, storage_root: &str) -> config::CameraConfig {
    let output_dir = cam
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(storage_root).join(&cam.id).display().to_string());
    cam.output_dir = Some(output_dir);
    cam
}

async fn run_web_server(addr: &str, web_root: &str, recordings_root: &str, state: AppState) -> Result<()> {
    let recordings_service = ServeDir::new(recordings_root);
    let web_service = ServeDir::new(web_root).append_index_html_on_directories(true);

    let app = axum::Router::new()
        .route("/api/cameras", axum::routing::get(api::list_cameras))
        .route("/api/cameras/", axum::routing::get(api::list_cameras))
        .route("/api/cameras/:id/snapshot", axum::routing::get(api::camera_snapshot))
        .route("/api/cameras/:id/snapshot/", axum::routing::get(api::camera_snapshot))
        .route("/api/cameras/:id/snapshot/debug", axum::routing::get(api::camera_snapshot_debug))
        .route("/api/cameras/:id/snapshot/debug/", axum::routing::get(api::camera_snapshot_debug))
        .route("/api/cameras/:id/stream/playlist.m3u8", axum::routing::get(api::stream_playlist))
        .route("/api/cameras/:id/stream/playback.m3u8", axum::routing::get(api::stream_playback_playlist))
        .route("/api/cameras/:id/stream/:segment", axum::routing::get(api::stream_segment))
        .route("/api/storage", axum::routing::get(api::storage_stats))
        .route("/api/storage/", axum::routing::get(api::storage_stats))
        .nest_service("/recordings", recordings_service)
        .fallback_service(web_service)
        .with_state(state)
        .layer(axum::middleware::from_fn(add_cors_headers));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Web UI listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn add_cors_headers(
    req: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let mut res = next.run(req).await;
    res.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    res.headers_mut().insert("Access-Control-Allow-Methods", "GET, OPTIONS, POST".parse().unwrap());
    res.headers_mut().insert("Access-Control-Allow-Headers", "Content-Type".parse().unwrap());
    res
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
