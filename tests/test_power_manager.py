"""Tests for the app-facing power manager facade."""

from __future__ import annotations

from datetime import datetime

from yoyopy.config import ConfigManager
from yoyopy.power.manager import PowerManager
from yoyopy.power.models import BatteryState, PowerConfig, PowerSnapshot


class FakeBackend:
    """Minimal power backend double for manager tests."""

    def __init__(self, snapshot: PowerSnapshot, probe_result: bool = True) -> None:
        self.snapshot = snapshot
        self.probe_result = probe_result
        self.refresh_calls = 0

    def probe(self) -> bool:
        return self.probe_result

    def get_snapshot(self) -> PowerSnapshot:
        self.refresh_calls += 1
        return self.snapshot


class ExplodingBackend:
    """Backend double that raises on refresh."""

    def probe(self) -> bool:
        return False

    def get_snapshot(self) -> PowerSnapshot:
        raise RuntimeError("backend boom")


def test_power_manager_refresh_caches_latest_snapshot() -> None:
    """Refreshing through the manager should retain the latest backend snapshot."""

    snapshot = PowerSnapshot(
        available=True,
        checked_at=datetime(2026, 4, 4, 12, 0, 0),
        battery=BatteryState(level_percent=83.0),
    )
    backend = FakeBackend(snapshot)
    manager = PowerManager(PowerConfig(), backend=backend)

    result = manager.refresh()

    assert result is snapshot
    assert manager.get_snapshot() is snapshot
    assert manager.get_battery_percentage() == 83.0
    assert backend.refresh_calls == 1


def test_power_manager_returns_unavailable_snapshot_when_backend_refresh_fails() -> None:
    """Refresh failures should be captured as an unavailable snapshot instead of bubbling up."""

    manager = PowerManager(PowerConfig(), backend=ExplodingBackend())

    snapshot = manager.refresh()

    assert snapshot.available is False
    assert snapshot.error == "backend boom"


def test_power_manager_reads_power_config_from_config_manager(tmp_path) -> None:
    """Typed application settings should feed the power manager configuration."""

    config_dir = tmp_path
    (config_dir / "yoyopod_config.yaml").write_text(
        """
power:
  enabled: true
  transport: tcp
  tcp_host: "192.168.1.10"
  tcp_port: 9000
  timeout_seconds: 5.0
""".strip(),
        encoding="utf-8",
    )

    config_manager = ConfigManager(config_dir=str(config_dir))
    manager = PowerManager.from_config_manager(config_manager)

    assert manager.config.enabled is True
    assert manager.config.transport == "tcp"
    assert manager.config.tcp_host == "192.168.1.10"
    assert manager.config.tcp_port == 9000
    assert manager.config.timeout_seconds == 5.0

