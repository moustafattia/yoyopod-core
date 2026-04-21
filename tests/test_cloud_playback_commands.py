from __future__ import annotations

from types import SimpleNamespace

from yoyopod.cloud.manager import CloudManager


class _FakeMqttClient:
    def __init__(self) -> None:
        self.is_connected = True
        self.acks: list[dict[str, object]] = []
        self.events: list[dict[str, object]] = []

    def publish_ack(self, **payload) -> bool:
        self.acks.append(payload)
        return True

    def publish_playback_event(self, payload: dict[str, object]) -> bool:
        self.events.append(payload)
        return True


class _FakeMusicBackend:
    def __init__(self) -> None:
        self.is_connected = True
        self.loaded: list[list[str]] = []
        self.playback_state_callbacks = []
        self.track_callbacks = []
        self.position_ms = 0
        self.stop_calls = 0
        self.pause_calls = 0
        self.play_calls = 0

    def on_track_change(self, callback) -> None:
        self.track_callbacks.append(callback)

    def on_playback_state_change(self, callback) -> None:
        self.playback_state_callbacks.append(callback)

    def load_tracks(self, uris: list[str]) -> bool:
        self.loaded.append(uris)
        return True

    def stop_playback(self) -> bool:
        self.stop_calls += 1
        return True

    def pause(self) -> bool:
        self.pause_calls += 1
        return True

    def play(self) -> bool:
        self.play_calls += 1
        return True

    def get_time_position(self) -> int:
        return self.position_ms

    def emit_playback(self, state: str) -> None:
        for callback in self.playback_state_callbacks:
            callback(state)


class _FakeConfigManager:
    def __init__(self) -> None:
        backend = SimpleNamespace(
            api_base_url="https://backend.example.test",
            auth_path="/v1/auth/device",
            refresh_path="/v1/auth/device/refresh",
            config_path_template="/v1/devices/{device_id}/config",
            contacts_bootstrap_path_template="/v1/devices/{device_id}/contacts/bootstrap",
            timeout_seconds=3.0,
            battery_report_interval_seconds=60,
            mqtt_broker_host="broker.example.test",
            mqtt_broker_port=1883,
            mqtt_username="",
            mqtt_password="",
            mqtt_use_tls=False,
            mqtt_transport="tcp",
        )
        self._cloud = SimpleNamespace(backend=backend)
        self.cloud_secrets_error = ""

    def get_cloud_settings(self):
        return self._cloud

    def get_cloud_device_id(self) -> str:
        return "YYP-DEV-0001"

    def get_cloud_device_secret(self) -> str:
        return "secret"

    def get_media_settings(self):
        return SimpleNamespace(
            music=SimpleNamespace(
                remote_cache_dir="data/media/remote_cache",
                remote_cache_max_bytes=64 * 1024 * 1024,
            )
        )

    def resolve_runtime_path(self, value: str) -> str:
        return value


class _FakePlaybackCache:
    def __init__(self) -> None:
        self.calls: list[dict[str, object]] = []

    def prepare(self, **kwargs):
        self.calls.append(kwargs)
        return SimpleNamespace(path="/tmp/cached-track.mp3", cache_hit=False)


class _FakeApp:
    def __init__(self, music_backend: _FakeMusicBackend) -> None:
        self.music_backend = music_backend
        self.context = None

    def _queue_main_thread_callback(self, callback) -> None:
        callback()


def test_play_track_command_acks_and_publishes_buffering_then_playing() -> None:
    music_backend = _FakeMusicBackend()
    manager = CloudManager(
        app=_FakeApp(music_backend),
        config_manager=_FakeConfigManager(),
        client=SimpleNamespace(),
    )
    manager._mqtt = _FakeMqttClient()
    manager._remote_playback_cache = _FakePlaybackCache()
    manager._bind_playback_callbacks()
    manager._start_worker = lambda *, name, work: work()  # type: ignore[method-assign]

    manager._apply_mqtt_command(
        {
            "messageType": "playback.command",
            "commandId": "cmd-1",
            "command": "play_track",
            "payload": {
                "trackId": "track-1",
                "mediaUrl": "https://media.example.test/file.mp3",
                "title": "Song",
                "artist": "Artist",
                "durationMs": 120000,
            },
        }
    )

    assert music_backend.loaded == [["/tmp/cached-track.mp3"]]
    assert manager._mqtt.acks[0]["command_id"] == "cmd-1"
    assert manager._mqtt.acks[0]["ok"] is True
    assert manager._mqtt.events[0]["eventType"] == "buffering"
    assert manager._remote_playback_session["cached_path"] == "/tmp/cached-track.mp3"

    music_backend.position_ms = 500
    music_backend.emit_playback("playing")

    assert manager._mqtt.events[-1]["eventType"] == "playing"
    assert manager._mqtt.events[-1]["payload"]["positionMs"] == 500


def test_stop_command_acks_and_emits_completed_when_near_end() -> None:
    music_backend = _FakeMusicBackend()
    manager = CloudManager(
        app=_FakeApp(music_backend),
        config_manager=_FakeConfigManager(),
        client=SimpleNamespace(),
    )
    manager._mqtt = _FakeMqttClient()
    manager._remote_playback_cache = _FakePlaybackCache()
    manager._bind_playback_callbacks()

    manager._remote_playback_session = {
        "command_id": "cmd-1",
        "track_id": "track-1",
        "duration_ms": 10000,
        "media_url": "https://media.example.test/file.mp3",
    }

    manager._apply_mqtt_command(
        {
            "commandId": "cmd-2",
            "command": "stop",
            "payload": {"trackId": "track-1"},
        }
    )

    assert music_backend.stop_calls == 1
    assert manager._mqtt.acks[-1]["command_id"] == "cmd-2"

    music_backend.position_ms = 9800
    music_backend.emit_playback("stopped")

    assert manager._mqtt.events[-1]["commandId"] == "cmd-1"
    assert manager._mqtt.events[-1]["eventType"] == "completed"


def test_invalid_play_command_nacks() -> None:
    music_backend = _FakeMusicBackend()
    manager = CloudManager(
        app=_FakeApp(music_backend),
        config_manager=_FakeConfigManager(),
        client=SimpleNamespace(),
    )
    manager._mqtt = _FakeMqttClient()
    manager._remote_playback_cache = _FakePlaybackCache()
    manager._bind_playback_callbacks()

    manager._apply_mqtt_command(
        {
            "commandId": "cmd-3",
            "command": "play_track",
            "payload": {"trackId": "track-1"},
        }
    )

    assert manager._mqtt.acks == [
        {
            "command_id": "cmd-3",
            "ok": False,
            "reason": "invalid_command",
        }
    ]


def test_stopped_callback_during_pending_remote_fetch_does_not_clear_session() -> None:
    music_backend = _FakeMusicBackend()
    manager = CloudManager(
        app=_FakeApp(music_backend),
        config_manager=_FakeConfigManager(),
        client=SimpleNamespace(),
    )
    manager._mqtt = _FakeMqttClient()
    manager._bind_playback_callbacks()

    queued_work: list[object] = []
    manager._start_worker = lambda *, name, work: queued_work.append(work)  # type: ignore[method-assign]

    manager._apply_mqtt_command(
        {
            "messageType": "playback.command",
            "commandId": "cmd-4",
            "command": "play_track",
            "payload": {
                "trackId": "track-4",
                "mediaUrl": "https://media.example.test/file.mp3",
                "title": "Song",
            },
        }
    )

    assert manager._remote_playback_session is not None
    assert manager._remote_playback_session["activation_pending"] is True

    music_backend.emit_playback("stopped")

    assert manager._remote_playback_session is not None
    assert music_backend.loaded == []
    assert len(queued_work) == 1
