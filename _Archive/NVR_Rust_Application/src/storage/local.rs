use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tracing::warn;

use crate::config::camera::CameraConfig;
use crate::storage::{RecordingIndex, SegmentMeta};

pub fn start_indexer(cam: CameraConfig) {
	std::thread::spawn(move || loop {
		if let Err(err) = update_index(&cam) {
			warn!("Failed to update index for {}: {}", cam.id, err);
		}

		std::thread::sleep(Duration::from_secs(5));
	});
}

fn update_index(cam: &CameraConfig) -> Result<()> {
	let output_dir = PathBuf::from(&cam.output_dir);
	std::fs::create_dir_all(&output_dir)
		.with_context(|| format!("Failed to create output dir {}", output_dir.display()))?;

	let segments = collect_segments(&output_dir, cam.segment_time)?;
	let playable = filter_playable_segments(&segments, cam.segment_time, unix_now());

	let index = RecordingIndex {
		camera_id: cam.id.clone(),
		updated_unix: unix_now(),
		segments: segments.clone(),
	};

	let index_path = output_dir.join("metadata.json");
	let playlist_path = output_dir.join("playlist.m3u8");
	let playback_path = output_dir.join("playlist_playback.m3u8");

	write_atomic(&index_path, serde_json::to_vec_pretty(&index)?)?;
	write_atomic(&playlist_path, build_playlist(cam.segment_time, &playable, false).into_bytes())?;
	write_atomic(
		&playback_path,
		build_playlist(cam.segment_time, &playable, true).into_bytes(),
	)?;

	Ok(())
}

fn collect_segments(dir: &Path, segment_time: u32) -> Result<Vec<SegmentMeta>> {
	let mut segments = Vec::new();

	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		if !path.is_file() {
			continue;
		}

		let file_name = match path.file_name().and_then(|name| name.to_str()) {
			Some(name) => name.to_string(),
			None => continue,
		};

		if !file_name.ends_with(".mp4") {
			continue;
		}

		let meta = entry.metadata()?;
		let modified_unix = meta
			.modified()
			.ok()
			.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
			.map(|dur| dur.as_secs())
			.unwrap_or(0);

		segments.push(SegmentMeta {
			file_name,
			size_bytes: meta.len(),
			modified_unix,
			duration_secs: segment_time,
		});
	}

	segments.sort_by(|a, b| a.file_name.cmp(&b.file_name));
	Ok(segments)
}

fn build_playlist(segment_time: u32, segments: &[SegmentMeta], endlist: bool) -> String {
	let mut out = String::new();
	out.push_str("#EXTM3U\n");
	out.push_str("#EXT-X-VERSION:3\n");
	out.push_str("#EXT-X-PLAYLIST-TYPE:EVENT\n");
	out.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", segment_time));
	let media_sequence = segments
		.first()
		.map(|seg| segment_sequence(seg))
		.unwrap_or(0);
	out.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", media_sequence));

	for seg in segments {
		out.push_str(&format!("#EXTINF:{},\n", segment_time));
		out.push_str(&seg.file_name);
		out.push('\n');
	}

	if endlist {
		out.push_str("#EXT-X-ENDLIST\n");
	}

	out
}

fn write_atomic(path: &Path, data: Vec<u8>) -> Result<()> {
	let tmp_path = path.with_extension("tmp");
	std::fs::write(&tmp_path, data)
		.with_context(|| format!("Failed to write temp file {}", tmp_path.display()))?;
	std::fs::rename(&tmp_path, path)
		.with_context(|| format!("Failed to move temp file into {}", path.display()))?;
	Ok(())
}

fn unix_now() -> u64 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|d| d.as_secs())
		.unwrap_or(0)
}

fn filter_playable_segments(
	segments: &[SegmentMeta],
	segment_time: u32,
	now_unix: u64,
) -> Vec<SegmentMeta> {
	let min_age = u64::from(segment_time).saturating_add(1);
	let min_size = 1_000_000;

	segments
		.iter()
		.filter(|seg| seg.size_bytes >= min_size)
		.filter(|seg| now_unix.saturating_sub(seg.modified_unix) >= min_age)
		.cloned()
		.collect()
}

fn segment_sequence(seg: &SegmentMeta) -> u64 {
	seg.file_name
		.chars()
		.filter(|ch| ch.is_ascii_digit())
		.collect::<String>()
		.parse::<u64>()
		.unwrap_or(0)
}
