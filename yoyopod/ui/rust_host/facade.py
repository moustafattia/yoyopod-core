"""Python runtime bridge for the Rust UI host."""

from __future__ import annotations

from typing import Any

from loguru import logger

from yoyopod.core.events import WorkerMessageReceivedEvent
from yoyopod.core.workers import WorkerProcessConfig
from yoyopod.ui.rust_host.snapshot import RustUiRuntimeSnapshot


class RustUiFacade:
    """Translate Python runtime state and Rust UI host intents across the worker seam."""

    def __init__(self, app: Any, *, worker_domain: str = "ui") -> None:
        self.app = app
        self.worker_domain = worker_domain

    def start_worker(
        self,
        worker_path: str,
        *,
        hardware: str = "mock",
        cwd: str | None = None,
        env: dict[str, str] | None = None,
    ) -> bool:
        supervisor = getattr(self.app, "worker_supervisor", None)
        register = getattr(supervisor, "register", None)
        start = getattr(supervisor, "start", None)
        if not callable(register) or not callable(start):
            return False

        register(
            self.worker_domain,
            WorkerProcessConfig(
                name=self.worker_domain,
                argv=[worker_path, "--hardware", hardware],
                cwd=cwd,
                env=env,
            ),
        )
        return bool(start(self.worker_domain))

    def send_snapshot(self) -> bool:
        supervisor = getattr(self.app, "worker_supervisor", None)
        send_command = getattr(supervisor, "send_command", None)
        if not callable(send_command):
            return False
        return bool(
            send_command(
                self.worker_domain,
                type="ui.runtime_snapshot",
                payload=RustUiRuntimeSnapshot.from_app(self.app).to_payload(),
            )
        )

    def send_tick(self, *, renderer: str = "auto") -> bool:
        supervisor = getattr(self.app, "worker_supervisor", None)
        send_command = getattr(supervisor, "send_command", None)
        if not callable(send_command):
            return False
        return bool(
            send_command(
                self.worker_domain,
                type="ui.tick",
                payload={"renderer": renderer},
            )
        )

    def send_backlight(self, *, brightness: float) -> bool:
        supervisor = getattr(self.app, "worker_supervisor", None)
        send_command = getattr(supervisor, "send_command", None)
        if not callable(send_command):
            return False
        return bool(
            send_command(
                self.worker_domain,
                type="ui.set_backlight",
                payload={"brightness": _clamp_normalized(brightness)},
            )
        )

    def handle_worker_message(self, event: WorkerMessageReceivedEvent) -> None:
        if event.domain != self.worker_domain:
            return
        if event.type == "ui.intent":
            self._dispatch_intent(event.payload)
        elif event.type == "ui.screen_changed":
            logger.debug("Rust UI screen changed: {}", event.payload.get("screen"))

    def _dispatch_intent(self, payload: dict[str, Any]) -> None:
        domain = str(payload.get("domain", "")).strip()
        action = str(payload.get("action", "")).strip()
        data = payload.get("payload", {})
        if not isinstance(data, dict):
            data = {}
        if not domain or not action:
            return

        service_domain, service_name = self._map_service(domain, action)
        command = self._build_command(service_domain, service_name, data)
        services = getattr(self.app, "services", None)
        call = getattr(services, "call", None)
        if not callable(call):
            return
        try:
            call(service_domain, service_name, command)
        except KeyError:
            logger.warning("No Python service registered for Rust UI intent {}.{}", domain, action)

    def _map_service(self, domain: str, action: str) -> tuple[str, str]:
        if domain == "music" and action == "play_pause":
            return "music", self._play_pause_service()
        if domain == "call" and action == "start":
            return "call", "dial"
        if domain == "call" and action == "toggle_mute":
            return "call", "unmute" if self._call_muted() else "mute"
        if domain == "voice" and action == "capture_start":
            return "call", "start_voice_note_recording"
        if domain == "voice" and action == "capture_stop":
            return "call", "stop_voice_note_recording"
        if domain == "voice" and action == "capture_toggle":
            if self._active_voice_note_recording():
                return "call", "stop_voice_note_recording"
            return "call", "start_voice_note_recording"
        return domain, action

    def _play_pause_service(self) -> str:
        context = getattr(self.app, "context", None)
        playback = getattr(getattr(context, "media", None), "playback", None)
        if bool(getattr(playback, "is_playing", False)):
            return "pause"
        if bool(getattr(playback, "is_paused", False)):
            return "resume"
        return "play"

    def _call_muted(self) -> bool:
        manager = getattr(self.app, "voip_manager", None)
        return bool(getattr(manager, "is_muted", False))

    def _build_command(self, domain: str, service: str, data: dict[str, Any]) -> object:
        if domain == "music":
            return self._build_music_command(service, data)
        if domain == "call":
            return self._build_call_command(service, data)
        return data

    def _build_music_command(self, service: str, data: dict[str, Any]) -> object:
        from yoyopod.integrations.music import (
            LoadPlaylistCommand,
            PauseCommand,
            PlayCommand,
            PlayRecentTrackCommand,
            ResumeCommand,
            ShuffleAllCommand,
        )

        if service == "load_playlist":
            return LoadPlaylistCommand(playlist_uri=_payload_value(data, "playlist_uri", "id"))
        if service == "play_recent_track":
            return PlayRecentTrackCommand(track_uri=_payload_value(data, "track_uri", "id"))
        if service == "shuffle_all":
            return ShuffleAllCommand()
        if service == "pause":
            return PauseCommand()
        if service == "resume":
            return ResumeCommand()
        if service == "play":
            return PlayCommand(track_uri=_payload_value(data, "track_uri", "id"))
        return data

    def _build_call_command(self, service: str, data: dict[str, Any]) -> object:
        from yoyopod.integrations.call import (
            AnswerCommand,
            DialCommand,
            HangupCommand,
            MuteCommand,
            RejectCommand,
            StartVoiceNoteRecordingCommand,
            StopVoiceNoteRecordingCommand,
            UnmuteCommand,
        )

        if service == "dial":
            return DialCommand(
                sip_address=_payload_value(data, "sip_address", "address", "id"),
                contact_name=_payload_value(data, "contact_name", "name", "title"),
            )
        if service == "answer":
            return AnswerCommand()
        if service == "hangup":
            return HangupCommand()
        if service == "reject":
            return RejectCommand()
        if service == "mute":
            return MuteCommand()
        if service == "unmute":
            return UnmuteCommand()
        if service == "start_voice_note_recording":
            context_recipient, context_name = self._active_voice_note_recipient()
            recipient_address = (
                _payload_value(
                    data,
                    "recipient_address",
                    "sip_address",
                    "id",
                )
                or context_recipient
            )
            if recipient_address:
                return StartVoiceNoteRecordingCommand(
                    recipient_address=recipient_address,
                    recipient_name=_payload_value(
                        data,
                        "recipient_name",
                        "name",
                        "title",
                    )
                    or context_name,
                )
            return data
        if service == "stop_voice_note_recording":
            return StopVoiceNoteRecordingCommand()
        return data

    def _active_voice_note_recipient(self) -> tuple[str, str]:
        context = getattr(self.app, "context", None)
        active_note = getattr(getattr(context, "talk", None), "active_voice_note", None)
        return (
            str(getattr(active_note, "recipient_address", "") or "").strip(),
            str(getattr(active_note, "recipient_name", "") or "").strip(),
        )

    def _active_voice_note_recording(self) -> bool:
        context = getattr(self.app, "context", None)
        active_note = getattr(getattr(context, "talk", None), "active_voice_note", None)
        send_state = str(getattr(active_note, "send_state", "") or "").strip().lower()
        return send_state == "recording"


def _payload_value(data: dict[str, Any], *keys: str) -> str:
    for key in keys:
        value = data.get(key)
        if value is not None:
            text = str(value).strip()
            if text:
                return text
    return ""


def _clamp_normalized(value: float) -> float:
    return max(0.0, min(1.0, float(value)))
