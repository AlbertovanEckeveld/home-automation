from __future__ import annotations

import logging
import os
from datetime import datetime, timedelta

import aiohttp

from homeassistant.core import HomeAssistant
from homeassistant.helpers.update_coordinator import DataUpdateCoordinator, UpdateFailed, CoordinatorEntity
from homeassistant.components.sensor import SensorEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.const import PERCENTAGE

from .const import DOMAIN
from .coordinator import AlbertoVECoordinator

_LOGGER = logging.getLogger(__name__)


class StorageCoordinator(DataUpdateCoordinator):
    def __init__(self, hass: HomeAssistant, host: str, port: int) -> None:
        self._url = f"http://{host}:{port}/api/storage"
        super().__init__(
            hass,
            _LOGGER,
            name="AlbertoVE storage",
            update_interval=timedelta(seconds=60),
        )

    async def _async_update_data(self) -> dict:
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(self._url, timeout=aiohttp.ClientTimeout(total=10)) as resp:
                    if resp.status != 200:
                        raise UpdateFailed(f"Albertove Cloud storage API returned HTTP {resp.status}")
                    return await resp.json()
        except aiohttp.ClientError as err:
            raise UpdateFailed(f"Cannot connect to Albertove Cloud storage endpoint: {err}") from err


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    host = entry.data["host"]
    port = entry.data["port"]
    storage_root = entry.options.get("storage_root") or "/recordings"

    coordinator = StorageCoordinator(hass, host, port)
    await coordinator.async_config_entry_first_refresh()

    main_coordinator: AlbertoVECoordinator = hass.data[DOMAIN][entry.entry_id]

    entities = [AlbertoVEStorageSensor(coordinator, host, port)]
    for cam_id in main_coordinator.data:
        entities.append(
            AlbertoVERecentRecordingsSensor(
                main_coordinator, cam_id, host, port, storage_root
            )
        )
    async_add_entities(entities)


class AlbertoVEStorageSensor(CoordinatorEntity, SensorEntity):
    _attr_has_entity_name = True

    def __init__(self, coordinator: StorageCoordinator, host: str, port: int) -> None:
        super().__init__(coordinator)
        self._host = host
        self._port = port
        self._attr_unique_id = f"albertove_storage_percent_{host}_{port}"
        self._attr_name = "NVR storage"
        self._attr_device_info = {
            "identifiers": {(DOMAIN, f"nvr_{host}_{port}")},
            "name": "AlbertoVE NVR",
            "manufacturer": "Albertove Cloud",
        }
        self._attr_native_unit_of_measurement = PERCENTAGE

    @property
    def native_value(self):
        """Return the storage percentage value.

        The API may return different shapes. Handle common cases:
        - {"percent": 42}
        - {"used": 4200000, "total": 10000000}
        - {"usage": {"percent": 42}} or similar
        """
        data = self.coordinator.data or {}

        # Direct percent fields (assume they represent percent used)
        for key in ("percent", "usage_percent", "used_percent", "percent_used"):
            if key in data:
                try:
                    return round(float(data[key]), 2)
                except (TypeError, ValueError):
                    break

        # Direct free_percent -> compute percent used
        for free_key in ("free_percent", "free"):
            if free_key in data:
                try:
                    return round(100.0 - float(data[free_key]), 2)
                except (TypeError, ValueError):
                    break

        # Nested usage
        usage = data.get("usage") or data.get("storage")
        if isinstance(usage, dict):
            for key in ("percent", "usage_percent"):
                if key in usage:
                    try:
                        return round(float(usage[key]), 2)
                    except (TypeError, ValueError):
                        break
            # nested free_percent
            for free_key in ("free_percent", "free"):
                if free_key in usage:
                    try:
                        return round(100.0 - float(usage[free_key]), 2)
                    except (TypeError, ValueError):
                        break

        # Compute from used/total
        used = data.get("used")
        total = data.get("total")
        if used is None and isinstance(usage, dict):
            used = usage.get("used")
            total = usage.get("total")

        try:
            if used is not None and total:
                return round(float(used) / float(total) * 100.0, 2)
        except (TypeError, ValueError, ZeroDivisionError):
            return None

        # If nothing matched, expose raw value if it's numeric
        if isinstance(data, (int, float)):
            # assume it's already a percentage
            return round(float(data), 2)

        return None

    @property
    def available(self) -> bool:
        return self.coordinator.last_update_success

    @property
    def extra_state_attributes(self) -> dict:
        return {"raw": self.coordinator.data}


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

        recordings = []
        for filepath in files[:10]:
            filename = os.path.basename(filepath)
            try:
                stat = os.stat(filepath)
                size_mb = stat.st_size / (1024 * 1024)
                mtime = datetime.fromtimestamp(stat.st_mtime).isoformat()
                playback_url = (
                    f"http://{self._host}:{self._port}/recordings/"
                    f"{self._cam_id}/{filename}"
                )
                recordings.append({
                    "filename": filename,
                    "size_mb": round(size_mb, 2),
                    "modified": mtime,
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

            files.sort(key=lambda f: os.path.getmtime(f), reverse=True)
        except OSError as e:
            _LOGGER.warning("Could not list recordings for %s: %s", self._cam_id, e)

        return files

    @property
    def available(self) -> bool:
        return self.coordinator.last_update_success


