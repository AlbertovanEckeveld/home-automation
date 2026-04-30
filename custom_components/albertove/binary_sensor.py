from homeassistant.components.binary_sensor import BinarySensorEntity, BinarySensorDeviceClass
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.helpers.update_coordinator import CoordinatorEntity

from .const import DOMAIN
from .coordinator import AlbertoVECoordinator


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    coordinator: AlbertoVECoordinator = hass.data[DOMAIN][entry.entry_id]
    async_add_entities(
        AlbertoVECameraRecordingSensor(coordinator, cam_id)
        for cam_id in coordinator.data
    )


class AlbertoVECameraRecordingSensor(CoordinatorEntity, BinarySensorEntity):
    _attr_device_class = BinarySensorDeviceClass.RUNNING
    _attr_has_entity_name = True

    def __init__(self, coordinator: AlbertoVECoordinator, cam_id: str) -> None:
        super().__init__(coordinator)
        self._cam_id = cam_id
        self._attr_unique_id = f"albertove_{cam_id}_recording"
        self._attr_name = "Recording"
        self._attr_device_info = {
            "identifiers": {(DOMAIN, cam_id)},
            "name": f"AlbertoVE {cam_id}",
            "manufacturer": "Albertove Cloud",
        }

    @property
    def is_on(self) -> bool:
        cam = self.coordinator.data.get(self._cam_id, {})
        return cam.get("status") == "recording"

    @property
    def available(self) -> bool:
        return self.coordinator.last_update_success and self._cam_id in self.coordinator.data
