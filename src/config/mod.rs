pub mod camera;

pub use camera::CameraConfig;

use serde::Deserialize;

fn default_storage_folder() -> String {
    "/home/alberto-adm/records".to_string()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_storage_folder")]
    pub default_storage_folder: String,
    pub cameras: Vec<CameraConfig>,
}
