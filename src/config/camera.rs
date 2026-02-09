use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct CameraConfig {
    pub id: String,
    pub rtsp_url: String,
    pub segment_time: u32,
    pub output_dir: String,
}
