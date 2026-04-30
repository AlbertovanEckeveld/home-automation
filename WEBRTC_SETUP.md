# AlbertoVE - WebRTC + Playback Setup Guide

## Overview

Dit systeem ondersteunt nu:
- **Live View**: WebRTC via go2rtc (real-time streaming in Home Assistant)
- **Playback**: MP4-bestanden per camera (historische beelden)
- **Storage Monitoring**: NVR schijfgebruik in %

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ NVR (Camera Feed) → MP4 Segments (Storage)                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│ API Layer:                                                   │
│ - /api/cameras/{id}/stream/playlist.m3u8 (Live HLS)        │
│ - /api/cameras/{id}/stream/playback.m3u8 (Historical)      │
│ - /api/cameras/{id}/snapshot (Current frame)               │
│ - /recordings/{id}/{filename} (Direct MP4 access)          │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│ go2rtc (WebRTC Proxy)                                       │
│ - Proxies HLS streams to WebRTC                             │
│ - Listens on port 8555 (WebRTC)                             │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│ Home Assistant Integration                                   │
│ - Live camera: HLS → WebRTC (via go2rtc)                   │
│ - Playback camera: HLS playlist                             │
│ - Storage sensor: %usage                                    │
│ - Recent recordings: MP4 list with URLs                     │
└─────────────────────────────────────────────────────────────┘
```

## Docker Setup

### 1. Use docker-compose.yml

```bash
docker-compose up -d
```

This starts:
- **NVR** on port 8080
- **go2rtc** on port 8555 (WebRTC) and 1984 (API)

### 2. Generate go2rtc Config

```bash
python3 scripts/gen_go2rtc_config.py <nvr_host> <nvr_port> > go2rtc.yaml
```

Example:
```bash
python3 scripts/gen_go2rtc_config.py 100.80.163.32 8080 > go2rtc.yaml
```

Then restart go2rtc:
```bash
docker-compose restart go2rtc
```

## Home Assistant Configuration

### 1. Add Integration

1. Go to **Settings → Devices & Services → Integrations**
2. Click **Create Integration** and search for **AlbertoVE**
3. Enter:
   - **Host**: Your NVR IP (e.g., `100.80.163.32`)
   - **Port**: `8080` (or custom if changed)

### 2. Entities Created

After setup, you'll see:

#### Per Camera:
- **Live view** (Camera) - Real-time WebRTC stream
- **Playback** (Camera) - Historical HLS playlist
- **Recording** (Binary Sensor) - Is camera recording?
- **Recent recordings** (Sensor) - List of MP4 files with direct URLs

#### System:
- **NVR storage** (Sensor) - Storage % used

### 3. WebRTC Configuration for Home Assistant

Home Assistant needs to know about your go2rtc instance. Add to `configuration.yaml`:

```yaml
stream:
  ll_hls: true  # Low-latency HLS for faster playback

webrtc:
  # If go2rtc is in Docker on same host
  stun_server: stun.l.google.com:19302
```

If your HA instance can't reach go2rtc directly, configure port forwarding or use an external WebRTC URL.

## Stream Types

### Live View (WebRTC)
- **Latency**: ~1-3 seconds (much better than HLS)
- **Codec**: H.264/VP8
- **Port**: `8555`
- **URL**: `webrtc://nvr_host:8555/camera_id`

### Playback (HLS)
- **Source**: MP4 segments stored on NVR
- **Playlist**: `/api/cameras/{id}/stream/playback.m3u8`
- **URL format**: `http://nvr_host:8080/api/cameras/{id}/stream/playback.m3u8`

### Direct MP4 Access
Available in the "Recent recordings" sensor attributes as URLs:
- `http://nvr_host:8080/recordings/{camera_id}/{filename}.mp4`

## Usage Examples

### View Live in Home Assistant

1. Open Entities tab in Home Assistant
2. Search for `Live view`
3. Click the camera entity card
4. Stream plays via WebRTC (native HA camera view)

### Access Historical Footage

1. Find **Recent recordings** sensor for your camera
2. Click to expand state attributes
3. Each recording shows:
   - Filename
   - File size (MB)
   - Modified time
   - Direct playback URL

Or directly access: `http://your_nvr:8080/recordings/cam2/recording_2025-05-01_14-30-45.mp4`

### Web Interface (Existing)

The web interface continues to work as before:
- `http://your_nvr:8080`
- Uses HLS playlists

## Troubleshooting

### Live view shows "unavailable"

Check:
1. go2rtc is running: `docker-compose logs go2rtc`
2. NVR streams are working: `curl http://nvr:8080/api/cameras | jq`
3. Home Assistant can reach go2rtc on port 8555

### Playback doesn't work

1. Check NVR has recordings: `curl http://nvr:8080/recordings/`
2. Verify MP4 files exist in storage path
3. Check "Recent recordings" sensor for file list

### Storage sensor shows wrong %

Wait 60 seconds (sensor update interval) or manually refresh the coordinator.

## Configuration Options

### Camera Recording Path

If using a different path than default `/recordings`, set in Home Assistant integration options.

### WebRTC Port

Default is 8555 in go2rtc. Change in `go2rtc.yaml`:

```yaml
webrtc:
  listen: :8555  # Change to your port
```

## Performance Notes

- **WebRTC** is more efficient than HLS for live viewing (lower latency)
- **HLS Playback** is suitable for historical content
- **Snapshots** are generated on-demand from recent MP4 segments
- **Storage Sensor** updates every 60 seconds

## Technical Details

### go2rtc Flow

```
HLS Playlist (from NVR)
    ↓
go2rtc (scales to WebRTC packet stream)
    ↓
WebRTC client (HA camera entity)
    ↓
Display in HA UI
```

### Why WebRTC?

- Lower latency (~1-3s vs HLS ~10s)
- More efficient use of bandwidth
- Native support in modern browsers/HA
- Better for real-time monitoring

## Future Enhancements

- [ ] WebRTC recording support (record the live stream)
- [ ] Motion detection trigger for playback
- [ ] Automatic cleanup of old recordings
- [ ] Multi-view dashboard layout

