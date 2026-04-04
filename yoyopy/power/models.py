"""Typed models for PiSugar-backed power management."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from yoyopy.config import ConfigManager


@dataclass(slots=True)
class PowerConfig:
    """Runtime power-backend configuration."""

    enabled: bool = True
    backend: str = "pisugar"
    transport: str = "auto"
    socket_path: str = "/tmp/pisugar-server.sock"
    tcp_host: str = "127.0.0.1"
    tcp_port: int = 8423
    timeout_seconds: float = 2.0
    poll_interval_seconds: float = 30.0
    low_battery_warning_percent: float = 20.0
    low_battery_warning_cooldown_seconds: float = 300.0
    auto_shutdown_enabled: bool = True
    critical_shutdown_percent: float = 10.0
    shutdown_delay_seconds: float = 15.0
    shutdown_command: str = "sudo -n shutdown -h now"
    shutdown_state_file: str = "data/last_shutdown_state.json"

    @staticmethod
    def from_config_manager(config_manager: "ConfigManager") -> "PowerConfig":
        """Create a power config from the current application settings."""

        settings = config_manager.get_app_settings().power
        return PowerConfig(
            enabled=settings.enabled,
            backend=settings.backend,
            transport=settings.transport,
            socket_path=settings.socket_path,
            tcp_host=settings.tcp_host,
            tcp_port=settings.tcp_port,
            timeout_seconds=settings.timeout_seconds,
            poll_interval_seconds=settings.poll_interval_seconds,
            low_battery_warning_percent=settings.low_battery_warning_percent,
            low_battery_warning_cooldown_seconds=settings.low_battery_warning_cooldown_seconds,
            auto_shutdown_enabled=settings.auto_shutdown_enabled,
            critical_shutdown_percent=settings.critical_shutdown_percent,
            shutdown_delay_seconds=settings.shutdown_delay_seconds,
            shutdown_command=settings.shutdown_command,
            shutdown_state_file=settings.shutdown_state_file,
        )


@dataclass(frozen=True, slots=True)
class PowerDeviceInfo:
    """Static-ish information about the attached PiSugar device."""

    model: str | None = None
    firmware_version: str | None = None


@dataclass(frozen=True, slots=True)
class BatteryState:
    """Current battery and charger state."""

    level_percent: float | None = None
    voltage_volts: float | None = None
    charging: bool | None = None
    power_plugged: bool | None = None
    allow_charging: bool | None = None
    output_enabled: bool | None = None
    temperature_celsius: float | None = None


@dataclass(frozen=True, slots=True)
class RTCState:
    """Current RTC status exposed by PiSugar."""

    time: datetime | None = None
    alarm_enabled: bool | None = None
    alarm_time: datetime | None = None
    alarm_repeat_mask: int | None = None
    adjust_ppm: float | None = None


@dataclass(frozen=True, slots=True)
class ShutdownState:
    """Safe-shutdown configuration currently active on the PiSugar."""

    safe_shutdown_level_percent: float | None = None
    safe_shutdown_delay_seconds: int | None = None


@dataclass(frozen=True, slots=True)
class PowerSnapshot:
    """Best-effort point-in-time power snapshot."""

    available: bool
    checked_at: datetime
    source: str = "pisugar"
    device: PowerDeviceInfo = field(default_factory=PowerDeviceInfo)
    battery: BatteryState = field(default_factory=BatteryState)
    rtc: RTCState = field(default_factory=RTCState)
    shutdown: ShutdownState = field(default_factory=ShutdownState)
    error: str = ""
