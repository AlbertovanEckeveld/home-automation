use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use crate::config::camera::CameraConfig;
use crate::recorder::ffmpeg::start_recorder;
use crate::storage::local::start_indexer;

pub fn monitor_camera(cam: CameraConfig) {
    start_indexer(cam.clone());

    std::thread::spawn(move || loop {
        info!("Starting recorder for {}", cam.id);
        match start_recorder(&cam) {
            Ok(mut child) => {
                let unreachable = Arc::new(AtomicBool::new(false));
                let last_lines: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::with_capacity(20)));

                if let Some(stderr) = child.stderr.take() {
                    let unreachable = Arc::clone(&unreachable);
                    let last_lines = Arc::clone(&last_lines);
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().flatten() {
                            if let Ok(mut buf) = last_lines.lock() {
                                if buf.len() >= 20 {
                                    buf.pop_front();
                                }
                                buf.push_back(line.clone());
                            }

                            if is_camera_unreachable(&line) {
                                unreachable.store(true, Ordering::Relaxed);
                                error!("Camera unreachable: {}", line);
                            }
                        }
                    });
                }

                let start = Instant::now();
                let grace = Duration::from_secs(u64::from(cam.segment_time).saturating_mul(2).max(15));
                let output_dir = PathBuf::from(&cam.output_dir);

                loop {
                    if let Ok(Some(status)) = child.try_wait() {
                        if status.success() {
                            warn!("FFmpeg exited with status: {}", status);
                        } else {
                            warn!("FFmpeg exited with status: {}", status);
                            if let Ok(buf) = last_lines.lock() {
                                if !buf.is_empty() {
                                    let lines = buf.iter().map(|line| line.as_str()).collect::<Vec<_>>();
                                    warn!("FFmpeg stderr (last lines): {}", lines.join(" | "));
                                }
                            }
                        }
                        break;
                    }

                    let mut should_restart = unreachable.load(Ordering::Relaxed);

                    if start.elapsed() >= grace {
                        match has_any_recording(&output_dir) {
                            Ok(true) => {}
                            Ok(false) => {
                                error!(
                                    "No recordings created after {:?}. Camera may be unreachable. Output dir: {}",
                                    grace,
                                    output_dir.display()
                                );
                                should_restart = true;
                            }
                            Err(err) => {
                                warn!("Failed to check output dir {}: {}", output_dir.display(), err);
                            }
                        }
                    }

                    if should_restart {
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }

                    std::thread::sleep(Duration::from_secs(1));
                }
            }
            Err(err) => {
                error!("Failed to start recorder for {}: {}", cam.id, err);
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    });
}

fn has_any_recording(dir: &Path) -> std::io::Result<bool> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn is_camera_unreachable(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let indicators = [
        "connection refused",
        "connection timed out",
        "connection to",
        "401 unauthorized",
        "unauthorized",
        "403 forbidden",
        "404 not found",
        "not found",
        "server returned 404",
        "failed to resolve hostname",
        "could not find stream information",
        "invalid data found",
        "end of file",
        "input/output error",
        "no route to host",
        "network is unreachable",
        "timed out",
        "rtsp error",
    ];

    indicators.iter().any(|needle| lower.contains(needle))
}
