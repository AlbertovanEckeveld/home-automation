pub mod local;

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SegmentMeta {
	pub file_name: String,
	pub size_bytes: u64,
	pub modified_unix: u64,
	pub duration_secs: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecordingIndex {
	pub camera_id: String,
	pub updated_unix: u64,
	pub segments: Vec<SegmentMeta>,
}
