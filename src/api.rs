use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::{Json};
use serde::Serialize;
use serde_json::json;
use tokio::time::timeout;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct AppState {
    pub cameras: CameraStore,
    pub storage_root: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CameraInfo {
    pub id: String,
    pub status: String,
    #[serde(skip)]
    pub output_dir: String,
}

pub type CameraStore = Arc<RwLock<HashMap<String, CameraInfo>>>;

pub fn new_store() -> CameraStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn list_cameras(State(state): State<AppState>) -> Json<Vec<CameraInfo>> {
    let map = state.cameras.read().unwrap();
    Json(map.values().cloned().collect())
}

pub async fn camera_snapshot(
    State(state): State<AppState>,
    Path(cam_id): Path<String>,
) -> Response {
    let output_dir = {
        let map = state.cameras.read().unwrap();
        match map.get(&cam_id) {
            Some(info) => info.output_dir.clone(),
            None => return not_found(),
        }
    };

    let candidates = find_snapshot_candidates(&output_dir);
    if candidates.is_empty() {
        return not_found();
    }

    for path in candidates {
        let result = timeout(
            Duration::from_secs(8),
            tokio::process::Command::new("ffmpeg")
                .args([
                    "-v",
                    "error",
                    "-i",
                    path.to_str().unwrap_or(""),
                    "-vframes",
                    "1",
                    "-f",
                    "image2",
                    "-q:v",
                    "2",
                    "-vcodec",
                    "mjpeg",
                    "pipe:1",
                ])
                .output(),
        )
        .await;

        match result {
            Ok(Ok(out)) if out.status.success() && !out.stdout.is_empty() => {
                let content_length = out.stdout.len();
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "image/jpeg")
                    .header(header::CACHE_CONTROL, "no-cache")
                    .header(header::CONTENT_LENGTH, content_length.to_string())
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Body::from(out.stdout))
                    .unwrap();
            }
            Ok(Ok(out)) => {
                warn!(
                    "Snapshot extraction failed for {} with ffmpeg exit {:?}",
                    path.display(),
                    out.status.code()
                );
            }
            Ok(Err(err)) => {
                warn!("Failed to spawn ffmpeg for snapshot: {}", err);
                // Avoid 500 in HA camera proxy when ffmpeg is temporarily unavailable.
                return not_found();
            }
            Err(_) => {
                warn!(
                    "Snapshot extraction timed out for {}; trying older segment",
                    path.display()
                );
            }
        }
    }

    // No stable segment could produce an image yet.
    not_found()
}

fn find_snapshot_candidates(dir: &str) -> Vec<PathBuf> {
    let now = SystemTime::now();
    let min_age = Duration::from_secs(2);
    let min_size_bytes = 300_000;

    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    }
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mp4"))
        .filter(|e| {
            // Skip tiny and very fresh files: these are usually incomplete segments.
            let Ok(meta) = e.metadata() else {
                return false;
            };
            if meta.len() < min_size_bytes {
                return false;
            }
            let Ok(modified) = meta.modified() else {
                return false;
            };
            let Ok(age) = now.duration_since(modified) else {
                return false;
            };
            age >= min_age
        })
        .collect();

    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    entries.reverse();
    entries.into_iter().take(5).map(|e| e.path()).collect()
}

fn not_found() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}

fn internal_error() -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap()
}

pub async fn stream_playlist(
    State(state): State<AppState>,
    Path(cam_id): Path<String>,
) -> Response {
    let output_dir = {
        let map = state.cameras.read().unwrap();
        match map.get(&cam_id) {
            Some(info) => info.output_dir.clone(),
            None => return not_found(),
        }
    };

    let playlist_path = PathBuf::from(&output_dir).join("playlist.m3u8");

    match tokio::fs::read(&playlist_path).await {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
                .header(header::CACHE_CONTROL, "public, max-age=2")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => not_found(),
    }
}

pub async fn stream_playback_playlist(
    State(state): State<AppState>,
    Path(cam_id): Path<String>,
) -> Response {
    let output_dir = {
        let map = state.cameras.read().unwrap();
        match map.get(&cam_id) {
            Some(info) => info.output_dir.clone(),
            None => return not_found(),
        }
    };

    let playlist_path = PathBuf::from(&output_dir).join("playlist_playback.m3u8");

    match tokio::fs::read(&playlist_path).await {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
                .header(header::CACHE_CONTROL, "no-cache")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .body(Body::from(content))
                .unwrap()
        }
        Err(_) => not_found(),
    }
}

pub async fn stream_segment(
    State(state): State<AppState>,
    Path((cam_id, segment_name)): Path<(String, String)>,
) -> Response {
    let output_dir = {
        let map = state.cameras.read().unwrap();
        match map.get(&cam_id) {
            Some(info) => info.output_dir.clone(),
            None => return not_found(),
        }
    };

    // Validate segment name to prevent directory traversal
    if segment_name.contains("..") || segment_name.contains('/') {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap();
    }

    let segment_path = PathBuf::from(&output_dir).join(&segment_name);

    // Verify that the segment is under the output directory
    if !segment_path.starts_with(&output_dir) {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap();
    }

    match tokio::fs::metadata(&segment_path).await {
        Ok(metadata) => {
            let file_size = metadata.len();

            // Don't serve files that are too small (likely incomplete)
            if file_size < 100_000 {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap();
            }

            match tokio::fs::read(&segment_path).await {
                Ok(content) => {
                    let content_length = content.len();
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "video/mp4")
                        .header(header::CONTENT_LENGTH, content_length.to_string())
                        .header(header::ACCEPT_RANGES, "bytes")
                        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Body::from(content))
                        .unwrap()
                }
                Err(_) => internal_error(),
            }
        }
        Err(_) => not_found(),
    }
}

pub async fn camera_snapshot_debug(
    State(state): State<AppState>,
    Path(cam_id): Path<String>,
) -> Json<serde_json::Value> {
    let output_dir = {
        let map = state.cameras.read().unwrap();
        match map.get(&cam_id) {
            Some(info) => info.output_dir.clone(),
            None => return Json(json!({"error": "camera_not_found"})),
        }
    };

    let candidates = find_snapshot_candidates(&output_dir);
    if candidates.is_empty() {
        return Json(json!({"error": "no_segments"}));
    }

    let mut infos = Vec::new();
    let now = SystemTime::now();

    for path in candidates {
        let mut entry = json!({
            "path": path.display().to_string(),
        });

        if let Ok(meta) = std::fs::metadata(&path) {
            let size = meta.len();
            let modified = meta.modified().ok().and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            let age = now.duration_since(meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)).ok().map(|d| d.as_secs()).unwrap_or(0);
            entry.as_object_mut().unwrap().insert("size".to_string(), json!(size));
            entry.as_object_mut().unwrap().insert("modified_unix".to_string(), json!(modified));
            entry.as_object_mut().unwrap().insert("age_secs".to_string(), json!(age));

            if size < 300_000 {
                entry.as_object_mut().unwrap().insert("status".to_string(), json!("too_small"));
                infos.push(entry);
                continue;
            }

            if age < 2 {
                entry.as_object_mut().unwrap().insert("status".to_string(), json!("too_fresh"));
                infos.push(entry);
                continue;
            }

            // Try spawning ffmpeg to generate a frame (short timeout)
            match timeout(Duration::from_secs(5), tokio::process::Command::new("ffmpeg")
                .args([
                    "-v", "error",
                    "-i", path.to_str().unwrap_or(""),
                    "-vframes", "1",
                    "-f", "image2",
                    "-q:v", "2",
                    "-vcodec", "mjpeg",
                    "pipe:1",
                ])
                .output()).await {
                Ok(Ok(out)) => {
                    if out.status.success() && !out.stdout.is_empty() {
                        entry.as_object_mut().unwrap().insert("status".to_string(), json!("ffmpeg_ok"));
                    } else {
                        entry.as_object_mut().unwrap().insert("status".to_string(), json!("ffmpeg_failed"));
                        entry.as_object_mut().unwrap().insert("ffmpeg_exit".to_string(), json!(out.status.code()));
                    }
                }
                Ok(Err(e)) => {
                    entry.as_object_mut().unwrap().insert("status".to_string(), json!("ffmpeg_spawn_error"));
                    entry.as_object_mut().unwrap().insert("error".to_string(), json!(format!("{}", e)));
                }
                Err(_) => {
                    entry.as_object_mut().unwrap().insert("status".to_string(), json!("ffmpeg_timeout"));
                }
            }
        } else {
            entry.as_object_mut().unwrap().insert("status".to_string(), json!("stat_failed"));
        }

        infos.push(entry);
    }

    Json(json!({"candidates": infos}))
}

#[derive(Debug, Serialize)]
pub struct StorageStats {
    // Percent free space available for non-root users.
    pub free_percent: String,
}

pub async fn storage_stats(
    State(state): State<AppState>,
) -> Result<Json<StorageStats>, Response> {
    let stats = match disk_free_stats(&state.storage_root) {
        Ok(stats) => stats,
        Err(_) => StorageStats {
            free_percent: "0.00".to_string(),
        },
    };
    Ok(Json(stats))
}

#[cfg(unix)]
fn disk_free_stats(path: &str) -> Result<StorageStats, std::io::Error> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let existing = existing_path_for_stat(path);
    let c_path = CString::new(existing.to_string_lossy().as_ref())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }

    let stat = unsafe { stat.assume_init() };
    let total_bytes = stat.f_blocks as u128 * stat.f_frsize as u128;
    let available_bytes = stat.f_bavail as u128 * stat.f_frsize as u128;
    let free_percent = if total_bytes == 0 {
        0.0
    } else {
        (available_bytes as f64 / total_bytes as f64) * 100.0
    };

    Ok(StorageStats {
        free_percent: format!("{:.2}", free_percent),
    })
}

#[cfg(unix)]
fn existing_path_for_stat(path: &str) -> std::path::PathBuf {
    let mut p = std::path::Path::new(path);
    while !p.exists() {
        if let Some(parent) = p.parent() {
            p = parent;
        } else {
            return std::path::PathBuf::from("/");
        }
    }
    p.to_path_buf()
}

#[cfg(not(unix))]
fn disk_free_stats(_path: &str) -> Result<StorageStats, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "disk stats not supported on this platform",
    ))
}

