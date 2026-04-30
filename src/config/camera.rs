use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct CameraConfig {
    pub id: String,
    pub rtsp_url: String,
    pub segment_time: u32,
    #[serde(default)]
    pub output_dir: Option<String>,
}
