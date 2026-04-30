#!/usr/bin/env python3
"""
Generate go2rtc configuration dynamically based on NVR cameras.
This script reads from the NVR API and creates a go2rtc config file.
"""
import sys
import json
import time
import urllib.request
import urllib.error

def get_cameras(nvr_host: str, nvr_port: int) -> list:
    """Fetch camera list from NVR."""
    url = f"http://{nvr_host}:{nvr_port}/api/cameras"
    try:
        with urllib.request.urlopen(url, timeout=5) as response:
            return json.loads(response.read())
    except (urllib.error.URLError, json.JSONDecodeError) as e:
        print(f"Error fetching cameras: {e}", file=sys.stderr)
        return []

def generate_go2rtc_config(nvr_host: str, nvr_port: int) -> str:
    """Generate go2rtc YAML config from NVR cameras."""
    cameras = get_cameras(nvr_host, nvr_port)

    config = """# go2rtc configuration (auto-generated)
# Generated at: {}

streams:
""".format(time.strftime("%Y-%m-%d %H:%M:%S"))

    for cam in cameras:
        cam_id = cam.get("id", "unknown")
        # Point go2rtc to the HLS stream; it will convert to WebRTC
        config += f"""  {cam_id}: http://{nvr_host}:{nvr_port}/api/cameras/{cam_id}/stream/playlist.m3u8

"""

    config += """webrtc:
  listen: :8555
  candidates:
    - mode: auto

api:
  listen: :1984

log:
  level: info
"""

    return config

if __name__ == "__main__":
    nvr_host = sys.argv[1] if len(sys.argv) > 1 else "localhost"
    nvr_port = int(sys.argv[2]) if len(sys.argv) > 2 else 8080

    config = generate_go2rtc_config(nvr_host, nvr_port)
    print(config)

