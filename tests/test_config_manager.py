#!/usr/bin/env python3
"""Tests for portable audio/device configuration helpers."""

from yoyopy.config.config_manager import ConfigManager
from yoyopy.ui.display.whisplay_paths import find_whisplay_driver


def test_audio_device_defaults(tmp_path, monkeypatch) -> None:
    """Default config should preserve the Pi-oriented audio device settings."""
    monkeypatch.delenv("YOYOPOD_PLAYBACK_DEVICE", raising=False)
    monkeypatch.delenv("YOYOPOD_RINGER_DEVICE", raising=False)
    monkeypatch.delenv("YOYOPOD_CAPTURE_DEVICE", raising=False)
    monkeypatch.delenv("YOYOPOD_MEDIA_DEVICE", raising=False)
    monkeypatch.delenv("YOYOPOD_RING_OUTPUT_DEVICE", raising=False)

    config_manager = ConfigManager(config_dir=str(tmp_path))

    assert config_manager.get_playback_device_id() == "ALSA: wm8960-soundcard"
    assert config_manager.get_ringer_device_id() == "ALSA: wm8960-soundcard"
    assert config_manager.get_capture_device_id() == "ALSA: wm8960-soundcard"
    assert config_manager.get_media_device_id() == "ALSA: wm8960-soundcard"
    assert config_manager.get_ring_output_device() == "wm8960-soundcard"


def test_audio_env_overrides(tmp_path, monkeypatch) -> None:
    """Environment variables should override persisted audio device settings."""
    monkeypatch.setenv("YOYOPOD_PLAYBACK_DEVICE", "ALSA: default")
    monkeypatch.setenv("YOYOPOD_RINGER_DEVICE", "ALSA: sysdefault")
    monkeypatch.setenv("YOYOPOD_CAPTURE_DEVICE", "ALSA: hw:2,0")
    monkeypatch.setenv("YOYOPOD_MEDIA_DEVICE", "ALSA: plughw:3")
    monkeypatch.setenv("YOYOPOD_RING_OUTPUT_DEVICE", "default")

    config_manager = ConfigManager(config_dir=str(tmp_path))

    assert config_manager.get_playback_device_id() == "ALSA: default"
    assert config_manager.get_ringer_device_id() == "ALSA: sysdefault"
    assert config_manager.get_capture_device_id() == "ALSA: hw:2,0"
    assert config_manager.get_media_device_id() == "ALSA: plughw:3"
    assert config_manager.get_ring_output_device() == "default"


def test_whisplay_driver_path_can_come_from_env(tmp_path, monkeypatch) -> None:
    """Whisplay driver discovery should accept a configured directory path."""
    driver_dir = tmp_path / "Driver"
    driver_dir.mkdir()
    driver_file = driver_dir / "WhisPlay.py"
    driver_file.write_text("# test driver\n", encoding="utf-8")

    monkeypatch.setenv("YOYOPOD_WHISPLAY_DRIVER", str(driver_dir))

    assert find_whisplay_driver() == driver_file
