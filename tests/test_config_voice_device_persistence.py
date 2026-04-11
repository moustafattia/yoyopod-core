"""Config persistence tests for voice device selectors."""

from __future__ import annotations

from pathlib import Path

from yoyopy.config.manager import ConfigManager


def test_config_manager_persists_voice_device_ids(tmp_path: Path) -> None:
    cfg_dir = tmp_path / "config"
    manager = ConfigManager(config_dir=str(cfg_dir))

    assert manager.set_voice_speaker_device_id("plughw:CARD=SE,DEV=0") is True
    assert manager.set_voice_capture_device_id("plughw:CARD=SE,DEV=0") is True

    reloaded = ConfigManager(config_dir=str(cfg_dir))
    assert reloaded.get_app_settings().voice.speaker_device_id == "plughw:CARD=SE,DEV=0"
    assert reloaded.get_app_settings().voice.capture_device_id == "plughw:CARD=SE,DEV=0"


def test_config_manager_allows_auto_device_ids(tmp_path: Path) -> None:
    cfg_dir = tmp_path / "config"
    manager = ConfigManager(config_dir=str(cfg_dir))

    assert manager.set_voice_speaker_device_id(None) is True
    assert manager.set_voice_capture_device_id("") is True

    reloaded = ConfigManager(config_dir=str(cfg_dir))
    assert reloaded.get_app_settings().voice.speaker_device_id == ""
    assert reloaded.get_app_settings().voice.capture_device_id == ""

