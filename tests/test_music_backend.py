"""Tests for MusicBackend protocol and MockMusicBackend."""

from __future__ import annotations

from yoyopy.audio.music.backend import MockMusicBackend, MusicBackend
from yoyopy.audio.music.models import Track


def test_mock_backend_satisfies_protocol() -> None:
    backend: MusicBackend = MockMusicBackend()
    assert backend.start() is True
    assert backend.is_connected is True


def test_mock_backend_transport_controls() -> None:
    backend = MockMusicBackend()
    backend.start()
    assert backend.play() is True
    assert backend.pause() is True
    assert backend.stop_playback() is True
    assert backend.next_track() is True
    assert backend.previous_track() is True


def test_mock_backend_volume() -> None:
    backend = MockMusicBackend()
    backend.start()
    assert backend.set_volume(75) is True
    assert backend.get_volume() == 75


def test_mock_backend_audio_device() -> None:
    backend = MockMusicBackend()
    backend.start()
    assert backend.set_audio_device("alsa/hw:1,0") is True


def test_mock_backend_load_tracks() -> None:
    backend = MockMusicBackend()
    backend.start()
    assert backend.load_tracks(["/music/a.mp3", "/music/b.mp3"]) is True
    assert "load_tracks" in backend.commands[-1]


def test_mock_backend_load_playlist_file() -> None:
    backend = MockMusicBackend()
    backend.start()
    assert backend.load_playlist_file("/music/chill.m3u") is True


def test_mock_backend_playback_state() -> None:
    backend = MockMusicBackend()
    backend.start()
    assert backend.get_playback_state() == "stopped"
    backend.play()
    assert backend.get_playback_state() == "playing"
    backend.pause()
    assert backend.get_playback_state() == "paused"


def test_mock_backend_track_change_callback() -> None:
    backend = MockMusicBackend()
    received: list[Track | None] = []
    backend.on_track_change(received.append)
    track = Track(uri="/music/a.mp3", name="A", artists=["X"])
    backend.emit_track_change(track)
    assert received == [track]


def test_mock_backend_playback_state_callback() -> None:
    backend = MockMusicBackend()
    received: list[str] = []
    backend.on_playback_state_change(received.append)
    backend.emit_playback_state_change("playing")
    assert received == ["playing"]


def test_mock_backend_connection_callback() -> None:
    backend = MockMusicBackend()
    received: list[tuple[bool, str]] = []
    backend.on_connection_change(lambda ok, reason: received.append((ok, reason)))
    backend.emit_connection_change(True, "connected")
    assert received == [(True, "connected")]


def test_mock_backend_stop() -> None:
    backend = MockMusicBackend()
    backend.start()
    backend.stop()
    assert backend.is_connected is False


def test_mock_backend_get_current_track() -> None:
    backend = MockMusicBackend()
    assert backend.get_current_track() is None
    track = Track(uri="/music/a.mp3", name="A", artists=["X"])
    backend.current_track = track
    assert backend.get_current_track() == track


def test_mock_backend_get_time_position() -> None:
    backend = MockMusicBackend()
    assert backend.get_time_position() == 0
    backend.time_position = 5000
    assert backend.get_time_position() == 5000
