from datetime import timedelta
import logging

import aiohttp

from homeassistant.core import HomeAssistant
from homeassistant.helpers.update_coordinator import DataUpdateCoordinator, UpdateFailed

_LOGGER = logging.getLogger(__name__)


class AlbertoVECoordinator(DataUpdateCoordinator):
    def __init__(self, hass: HomeAssistant, host: str, port: int) -> None:
        self._url = f"http://{host}:{port}/api/cameras"
        super().__init__(
            hass,
            _LOGGER,
            name="AlbertoVE",
            update_interval=timedelta(seconds=30),
        )

    async def _async_update_data(self) -> dict:
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(
                    self._url, timeout=aiohttp.ClientTimeout(total=10)
                ) as resp:
                    if resp.status != 200:
                        raise UpdateFailed(f"Albertove Cloud API returned HTTP {resp.status}")
                    cameras = await resp.json()
                    return {cam["id"]: cam for cam in cameras}
        except aiohttp.ClientError as err:
            raise UpdateFailed(f"Cannot connect to Albertove Cloud: {err}") from err
