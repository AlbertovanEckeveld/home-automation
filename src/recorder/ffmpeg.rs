use std::process::{Child, Command, Stdio};
use anyhow::{Result, Context};
use crate::config::camera::CameraConfig;

pub fn start_recorder(cam: &CameraConfig) -> Result<Child> {
    let output_dir = cam
        .output_dir
        .as_ref()
        .expect("output_dir must be resolved before start_recorder");
    std::fs::create_dir_all(output_dir)?;

    let child = Command::new("ffmpeg")
        .args([
            "-rtsp_transport", "tcp",
            "-i", &cam.rtsp_url,
            "-c", "copy",
            "-movflags", "+faststart",
            "-f", "segment",
            "-strftime", "1",
            "-segment_time", &cam.segment_time.to_string(),
            "-reset_timestamps", "1",
            &format!("{}/%Y-%m-%d_%H-%M-%S.mp4", output_dir),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start ffmpeg")?;

    Ok(child)
}
