"""
Sensor entity for recent recordings / playback access.
Lists available MP4 files per camera for convenient playback.
"""
from __future__ import annotations

import logging
import os
from datetime import datetime, timedelta

from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.components.sensor import SensorEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.helpers.update_coordinator import CoordinatorEntity
from homeassistant.const import UnitOfTime

from .const import DOMAIN
from .coordinator import AlbertoVECoordinator

_LOGGER = logging.getLogger(__name__)


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    """Set up playback recording sensors for each camera."""
    coordinator: AlbertoVECoordinator = hass.data[DOMAIN][entry.entry_id]
    host = entry.data["host"]
    port = entry.data["port"]

    # Get the storage root from config or use a default
    storage_root = entry.options.get("storage_root") or "/recordings"

    entities = []
    for cam_id in coordinator.data:
        entities.append(
            AlbertoVERecentRecordingsSensor(
                coordinator, cam_id, host, port, storage_root
            )
        )

    async_add_entities(entities)


class AlbertoVERecentRecordingsSensor(CoordinatorEntity, SensorEntity):
    """Sensor showing recent recordings for a camera."""

    _attr_has_entity_name = True
    _attr_icon = "mdi:video-box"

    def __init__(
        self,
        coordinator: AlbertoVECoordinator,
        cam_id: str,
        host: str,
        port: int,
        storage_root: str,
    ) -> None:
        super().__init__(coordinator)
        self._cam_id = cam_id
        self._host = host
        self._port = port
        self._storage_root = storage_root
        self._attr_unique_id = f"albertove_{cam_id}_recent_recordings"
        self._attr_name = "Recent recordings"
        self._attr_device_info = {
            "identifiers": {(DOMAIN, cam_id)},
            "name": f"AlbertoVE {cam_id}",
            "manufacturer": "Albertove Cloud",
        }

    @property
    def native_value(self) -> str:
        """Return the number of recent recordings."""
        files = self._get_recent_recordings()
        return str(len(files))

    @property
    def extra_state_attributes(self) -> dict:
        """Return recent recordings as state attributes."""
        files = self._get_recent_recordings()

        # Format as list of dicts with filename, size, modified time, and playback URL
        recordings = []
        for filepath in files[:10]:  # Show last 10
            filename = os.path.basename(filepath)
            try:
                stat = os.stat(filepath)
                size_mb = stat.st_size / (1024 * 1024)
                mtime = datetime.fromtimestamp(stat.st_mtime)
                playback_url = (
                    f"http://{self._host}:{self._port}/recordings/"
                    f"{self._cam_id}/{filename}"
                )
                recordings.append({
                    "filename": filename,
                    "size_mb": round(size_mb, 2),
                    "modified": mtime.isoformat(),
                    "url": playback_url,
                })
            except (OSError, ValueError):
                continue

        return {
            "recent_recordings": recordings,
            "storage_path": f"{self._storage_root}/{self._cam_id}",
        }

    def _get_recent_recordings(self, hours: int = 24) -> list:
        """Get MP4 files from the last N hours."""
        cam_dir = os.path.join(self._storage_root, self._cam_id)

        if not os.path.isdir(cam_dir):
            return []

        cutoff_time = datetime.now() - timedelta(hours=hours)
        files = []

        try:
            for filename in os.listdir(cam_dir):
                if not filename.endswith(".mp4"):
                    continue

                filepath = os.path.join(cam_dir, filename)
                try:
                    mtime = datetime.fromtimestamp(os.path.getmtime(filepath))
                    if mtime >= cutoff_time:
                        files.append(filepath)
                except (OSError, ValueError):
                    continue

            # Sort by modification time, newest first
            files.sort(key=lambda f: os.path.getmtime(f), reverse=True)
        except OSError as e:
            _LOGGER.warning("Could not list recordings for %s: %s", self._cam_id, e)

        return files

    @property
    def available(self) -> bool:
        return self.coordinator.last_update_success

