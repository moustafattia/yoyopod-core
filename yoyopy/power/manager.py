"""App-facing power manager facade."""

from __future__ import annotations

from datetime import datetime
from typing import TYPE_CHECKING

from loguru import logger

from yoyopy.power.backend import PiSugarBackend, PowerBackend
from yoyopy.power.models import PowerConfig, PowerSnapshot

if TYPE_CHECKING:
    from yoyopy.config import ConfigManager


class PowerManager:
    """Coordinate power backend access and retain the latest snapshot."""

    def __init__(self, config: PowerConfig, backend: PowerBackend | None = None) -> None:
        self.config = config
        self.backend = backend or PiSugarBackend(config)
        self.last_snapshot = PowerSnapshot(
            available=False,
            checked_at=datetime.now(),
            error="power snapshot not collected yet",
        )

    @classmethod
    def from_config_manager(cls, config_manager: "ConfigManager") -> "PowerManager":
        """Build a power manager from the typed app configuration."""

        return cls(PowerConfig.from_config_manager(config_manager))

    def probe(self) -> bool:
        """Return True when the configured backend is reachable."""
        if not self.config.enabled:
            return False
        return self.backend.probe()

    def refresh(self) -> PowerSnapshot:
        """Collect and store a new snapshot from the backend."""
        try:
            self.last_snapshot = self.backend.get_snapshot()
        except Exception as exc:
            logger.error(f"Power snapshot refresh failed: {exc}")
            self.last_snapshot = PowerSnapshot(
                available=False,
                checked_at=datetime.now(),
                error=str(exc),
            )
        return self.last_snapshot

    def get_snapshot(self, refresh: bool = False) -> PowerSnapshot:
        """Return the cached snapshot, optionally refreshing first."""
        if refresh:
            return self.refresh()
        return self.last_snapshot

    def get_battery_percentage(self, refresh: bool = False) -> float | None:
        """Return the current battery percentage when available."""
        snapshot = self.get_snapshot(refresh=refresh)
        return snapshot.battery.level_percent

