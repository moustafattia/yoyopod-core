"""Telemetry manager — publishes device state to the backend via MQTT."""

from __future__ import annotations

import time
from typing import TYPE_CHECKING

from loguru import logger

from yoyopod.cloud.mqtt_client import DeviceMqttClient

if TYPE_CHECKING:
    from yoyopod.config.models import BackendTelemetryConfig


class TelemetryManager:
    """Own MQTT-based device telemetry publishing (battery, heartbeat, connectivity).

    This is intentionally decoupled from the runtime loop — it runs its MQTT
    client on a background daemon thread and exposes simple fire-and-forget
    publish helpers that the coordinator layer calls directly.
    """

    def __init__(self, *, config: "BackendTelemetryConfig", device_id: str) -> None:
        self._config = config
        self._device_id = device_id
        self._mqtt: DeviceMqttClient | None = None
        self._next_battery_report_at: float = 0.0

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def start(self) -> None:
        """Create and connect the MQTT client."""
        cfg = self._config
        if not cfg.mqtt_broker_host:
            logger.info("Telemetry: no MQTT broker configured — skipping")
            return

        self._mqtt = DeviceMqttClient(
            broker_host=cfg.mqtt_broker_host,
            device_id=self._device_id,
            port=cfg.mqtt_broker_port,
            username=cfg.mqtt_username or None,
            password=cfg.mqtt_password or None,
            use_tls=cfg.mqtt_use_tls,
        )
        self._mqtt.start()
        logger.info(
            "Telemetry: MQTT client started (broker={}:{})",
            cfg.mqtt_broker_host,
            cfg.mqtt_broker_port,
        )

    def stop(self) -> None:
        """Disconnect the MQTT client."""
        if self._mqtt is not None:
            self._mqtt.stop()
            self._mqtt = None

    # ------------------------------------------------------------------
    # Publishing
    # ------------------------------------------------------------------

    def publish_battery(self, *, level: int, charging: bool, now: float | None = None) -> None:
        """Publish a battery event, rate-limited by config interval."""
        if self._mqtt is None:
            return

        monotonic_now = time.monotonic() if now is None else now
        if monotonic_now < self._next_battery_report_at:
            return

        if self._mqtt.publish_battery(level=level, charging=charging):
            self._next_battery_report_at = (
                monotonic_now + self._config.battery_report_interval_seconds
            )

    def publish_heartbeat(self, *, firmware_version: str | None = None) -> None:
        """Publish a heartbeat event (e.g. on screen wake or periodic tick)."""
        if self._mqtt is not None:
            self._mqtt.publish_heartbeat(firmware_version=firmware_version)

    @property
    def is_connected(self) -> bool:
        return self._mqtt is not None and self._mqtt.is_connected
