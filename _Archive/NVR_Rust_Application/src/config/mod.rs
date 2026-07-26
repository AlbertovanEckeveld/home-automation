pub mod camera;

use serde::Deserialize;
use crate::config::camera::CameraConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cameras: Vec<CameraConfig>,
}
