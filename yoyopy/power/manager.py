"""App-facing power manager facade."""

from __future__ import annotations

import shlex
import subprocess
from datetime import datetime
from typing import TYPE_CHECKING, Callable

from loguru import logger

from yoyopy.power.backend import PiSugarBackend, PowerBackend
from yoyopy.power.models import PowerConfig, PowerSnapshot, RTCState

if TYPE_CHECKING:
    from yoyopy.config import ConfigManager


ShutdownHook = Callable[[], None]
ShutdownRunner = Callable[[list[str]], int]


class PowerManager:
    """Coordinate power backend access and retain the latest snapshot."""

    def __init__(
        self,
        config: PowerConfig,
        backend: PowerBackend | None = None,
        shutdown_runner: ShutdownRunner | None = None,
    ) -> None:
        self.config = config
        self.backend = backend or PiSugarBackend(config)
        self._shutdown_runner = shutdown_runner or self._default_shutdown_runner
        self._shutdown_hooks: list[tuple[str, ShutdownHook]] = []
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

    def get_rtc_state(self, refresh: bool = False) -> RTCState:
        """Return the latest RTC state, optionally refreshing first."""
        snapshot = self.get_snapshot(refresh=refresh)
        return snapshot.rtc

    def sync_time_to_rtc(self) -> RTCState:
        """Sync Raspberry Pi system time to the PiSugar RTC and return fresh RTC state."""
        self.backend.sync_time_to_rtc()
        return self.get_rtc_state(refresh=True)

    def sync_time_from_rtc(self) -> RTCState:
        """Sync PiSugar RTC time to the Raspberry Pi system clock and return fresh RTC state."""
        self.backend.sync_time_from_rtc()
        return self.get_rtc_state(refresh=True)

    def set_rtc_alarm(self, when: datetime, repeat_mask: int = 127) -> RTCState:
        """Set the PiSugar RTC wake alarm and return fresh RTC state."""
        self.backend.set_rtc_alarm(when, repeat_mask)
        return self.get_rtc_state(refresh=True)

    def disable_rtc_alarm(self) -> RTCState:
        """Disable the PiSugar RTC wake alarm and return fresh RTC state."""
        self.backend.disable_rtc_alarm()
        return self.get_rtc_state(refresh=True)

    def register_shutdown_hook(self, name: str, hook: ShutdownHook) -> None:
        """Register one callable to run before a graceful poweroff."""
        self._shutdown_hooks.append((name, hook))

    def run_shutdown_hooks(self) -> list[str]:
        """Run graceful-shutdown hooks and return any failing hook names."""
        failed_hooks: list[str] = []
        for name, hook in self._shutdown_hooks:
            try:
                hook()
            except Exception as exc:
                logger.error(f"Shutdown hook failed ({name}): {exc}")
                failed_hooks.append(name)
        return failed_hooks

    def request_system_shutdown(self) -> bool:
        """Execute the configured system shutdown command."""
        command = shlex.split(self.config.shutdown_command)
        if not command:
            logger.warning("No power shutdown command configured")
            return False

        return_code = self._shutdown_runner(command)
        if return_code != 0:
            logger.error(
                "System shutdown command failed (code {}): {}",
                return_code,
                self.config.shutdown_command,
            )
            return False
        return True

    @staticmethod
    def _default_shutdown_runner(command: list[str]) -> int:
        """Execute the real shutdown command via subprocess."""
        completed = subprocess.run(command, check=False)
        return completed.returncode

