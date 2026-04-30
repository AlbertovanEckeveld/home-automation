import logging

import aiohttp

from homeassistant.components.camera import Camera, CameraEntityFeature
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.helpers.update_coordinator import CoordinatorEntity

from .const import DOMAIN
from .coordinator import AlbertoVECoordinator

_LOGGER = logging.getLogger(__name__)


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    coordinator: AlbertoVECoordinator = hass.data[DOMAIN][entry.entry_id]
    host = entry.data["host"]
    port = entry.data["port"]
    entities = []
    for cam_id in coordinator.data:
        entities.append(AlbertoVELiveCamera(coordinator, cam_id, host, port))
        entities.append(AlbertoVEPlaybackCamera(coordinator, cam_id, host, port))
    async_add_entities(entities)


class _BaseAlbertoVECamera(CoordinatorEntity, Camera):
    _attr_has_entity_name = True
    _attr_use_stream_for_stills = True
    _attr_is_streaming = True

    def __init__(
        self,
        coordinator: AlbertoVECoordinator,
        cam_id: str,
        host: str,
        port: int,
    ) -> None:
        CoordinatorEntity.__init__(self, coordinator)
        Camera.__init__(self)
        self._cam_id = cam_id
        self._host = host
        self._port = port
        self._attr_device_info = {
            "identifiers": {(DOMAIN, cam_id)},
            "name": f"AlbertoVE {cam_id}",
            "manufacturer": "Albertove Cloud",
        }

    async def async_camera_image(
        self, width: int | None = None, height: int | None = None
    ) -> bytes | None:
        url = f"http://{self._host}:{self._port}/api/cameras/{self._cam_id}/snapshot"
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(
                    url, timeout=aiohttp.ClientTimeout(total=15)
                ) as resp:
                    if resp.status == 200:
                        return await resp.read()
        except aiohttp.ClientError as err:
            _LOGGER.debug("Could not fetch snapshot for %s: %s", self._cam_id, err)
        return None

    @property
    def available(self) -> bool:
        if not self.coordinator.last_update_success:
            _LOGGER.debug(
                "AlbertoVE camera %s unavailable: last_update_success=%s",
                self._cam_id,
                self.coordinator.last_update_success,
            )
            return False
        data = self.coordinator.data or {}
        in_data = self._cam_id in data
        if not in_data:
            _LOGGER.debug(
                "AlbertoVE camera %s not in coordinator data. Available cam_ids: %s",
                self._cam_id,
                list(data.keys()) if data else [],
            )
        return in_data


class AlbertoVELiveCamera(_BaseAlbertoVECamera):
    """Live camera stream via WebRTC (go2rtc) for better Home Assistant integration."""
    def __init__(
        self,
        coordinator: AlbertoVECoordinator,
        cam_id: str,
        host: str,
        port: int,
    ) -> None:
        super().__init__(coordinator, cam_id, host, port)
        self._attr_unique_id = f"albertove_{cam_id}_live"
        self._attr_name = "Live view"
        self._attr_supported_features = CameraEntityFeature.STREAM
        # Note: go2rtc typically listens on port 8555 for WebRTC
        self._webrtc_port = 8555

    async def stream_source(self) -> str | None:
        """Return HLS stream source for go2rtc to proxy to WebRTC.
        
        go2rtc will convert this HLS stream to WebRTC for Home Assistant.
        """
        return (
            f"http://{self._host}:{self._port}/api/cameras/{self._cam_id}/stream/playlist.m3u8"
        )

    @property
    def extra_state_attributes(self) -> dict:
        """Expose WebRTC stream URL for advanced uses."""
        return {
            "webrtc_url": f"webrtc://{self._host}:{self._webrtc_port}/{self._cam_id}",
            "hls_url": f"http://{self._host}:{self._port}/api/cameras/{self._cam_id}/stream/playlist.m3u8",
            "snapshot_url": f"http://{self._host}:{self._port}/api/cameras/{self._cam_id}/snapshot",
        }


class AlbertoVEPlaybackCamera(_BaseAlbertoVECamera):
    def __init__(
        self,
        coordinator: AlbertoVECoordinator,
        cam_id: str,
        host: str,
        port: int,
    ) -> None:
        super().__init__(coordinator, cam_id, host, port)
        self._attr_unique_id = f"albertove_{cam_id}_playback"
        self._attr_name = "Playback"
        self._attr_supported_features = CameraEntityFeature.STREAM

    async def stream_source(self) -> str | None:
        return (
            f"http://{self._host}:{self._port}/api/cameras/{self._cam_id}/stream/playback.m3u8"
        )


