"""VoIPBackend adapter backed by the Rust VoIP Host worker."""

from __future__ import annotations

import dataclasses
import os
from collections.abc import Callable
from typing import Any

from loguru import logger

from yoyopod.backends.voip.protocol import VoIPIterateMetrics
from yoyopod.core.workers import WorkerProcessConfig
from yoyopod.integrations.call.models import (
    BackendStopped,
    CallState,
    CallStateChanged,
    IncomingCallDetected,
    RegistrationState,
    RegistrationStateChanged,
    VoIPConfig,
    VoIPEvent,
)


class RustHostBackend:
    """Calls-only VoIPBackend adapter for the Rust VoIP Host."""

    def __init__(
        self,
        config: VoIPConfig,
        *,
        worker_supervisor: Any,
        worker_path: str,
        domain: str = "voip",
        env: dict[str, str] | None = None,
        cwd: str | None = None,
    ) -> None:
        self.config = config
        self.worker_supervisor = worker_supervisor
        self.worker_path = worker_path
        self.domain = domain
        self.env = env
        self.cwd = cwd
        self.running = False
        self.event_callbacks: list[Callable[[VoIPEvent], None]] = []
        self._registered_with_supervisor = False

    def on_event(self, callback: Callable[[VoIPEvent], None]) -> None:
        self.event_callbacks.append(callback)

    def start(self) -> bool:
        if self.running:
            return True
        if not self.config.is_backend_start_configured():
            logger.error("Rust VoIP Host requires sip_server and sip_identity")
            return False

        register = getattr(self.worker_supervisor, "register", None)
        start = getattr(self.worker_supervisor, "start", None)
        if not callable(register) or not callable(start):
            logger.error("Rust VoIP Host supervisor is unavailable")
            return False

        try:
            if not self._registered_with_supervisor:
                register(
                    self.domain,
                    WorkerProcessConfig(
                        name=self.domain,
                        argv=[self.worker_path],
                        cwd=self.cwd,
                        env=self._process_env(),
                    ),
                )
                self._registered_with_supervisor = True
            if not bool(start(self.domain)):
                return False
            if not self._send("voip.configure", self._config_payload()):
                return False
            if not self._send("voip.register", {}):
                return False
        except Exception as exc:
            logger.error("Rust VoIP Host start failed: {}", exc)
            self.running = False
            return False

        self.running = True
        return True

    def stop(self) -> None:
        self._send("voip.unregister", {})
        stop = getattr(self.worker_supervisor, "stop", None)
        if callable(stop):
            stop(self.domain, grace_seconds=1.0)
        self.running = False

    def iterate(self) -> int:
        return 0

    def get_iterate_metrics(self) -> VoIPIterateMetrics | None:
        return None

    def make_call(self, sip_address: str) -> bool:
        return self._send("voip.dial", {"uri": sip_address})

    def answer_call(self) -> bool:
        return self._send("voip.answer", {})

    def reject_call(self) -> bool:
        return self._send("voip.reject", {})

    def hangup(self) -> bool:
        return self._send("voip.hangup", {})

    def mute(self) -> bool:
        return self._send("voip.set_mute", {"muted": True})

    def unmute(self) -> bool:
        return self._send("voip.set_mute", {"muted": False})

    def send_text_message(self, sip_address: str, text: str) -> str | None:
        del sip_address, text
        logger.warning("Rust VoIP Host does not support text messages in calls-only mode")
        return None

    def start_voice_note_recording(self, file_path: str) -> bool:
        del file_path
        logger.warning("Rust VoIP Host does not support voice notes in calls-only mode")
        return False

    def stop_voice_note_recording(self) -> int | None:
        return None

    def cancel_voice_note_recording(self) -> bool:
        return False

    def send_voice_note(
        self,
        sip_address: str,
        *,
        file_path: str,
        duration_ms: int,
        mime_type: str,
    ) -> str | None:
        del sip_address, file_path, duration_ms, mime_type
        return None

    def handle_worker_message(self, event: Any) -> None:
        if getattr(event, "domain", self.domain) != self.domain:
            return
        if getattr(event, "kind", "event") != "event":
            return

        payload = getattr(event, "payload", {}) or {}
        event_type = getattr(event, "type", "")
        if event_type == "voip.registration_changed":
            self._dispatch(
                RegistrationStateChanged(
                    state=_registration_state(str(payload.get("state", "none")))
                )
            )
            return
        if event_type == "voip.incoming_call":
            self._dispatch(
                IncomingCallDetected(caller_address=str(payload.get("from_uri", "")))
            )
            return
        if event_type == "voip.call_state_changed":
            self._dispatch(CallStateChanged(state=_call_state(str(payload.get("state", "idle")))))
            return
        if event_type == "voip.backend_stopped":
            self.running = False
            self._dispatch(BackendStopped(reason=str(payload.get("reason", ""))))

    def _send(self, message_type: str, payload: dict[str, Any]) -> bool:
        send_command = getattr(self.worker_supervisor, "send_command", None)
        if not callable(send_command):
            return False
        try:
            return bool(send_command(self.domain, type=message_type, payload=payload))
        except Exception as exc:
            logger.error("Rust VoIP Host command {} failed: {}", message_type, exc)
            return False

    def _dispatch(self, event: VoIPEvent) -> None:
        for callback in list(self.event_callbacks):
            try:
                callback(event)
            except Exception:
                logger.exception("Rust VoIP Host callback raised for {}", type(event).__name__)

    def _config_payload(self) -> dict[str, Any]:
        payload = dataclasses.asdict(self.config)
        payload["conference_factory_uri"] = self.config.effective_conference_factory_uri()
        payload["file_transfer_server_url"] = self.config.effective_file_transfer_server_url()
        payload["lime_server_url"] = self.config.effective_lime_server_url()
        return payload

    def _process_env(self) -> dict[str, str] | None:
        if self.env is None:
            return None
        merged = dict(os.environ)
        merged.update(self.env)
        return merged


def _registration_state(value: str) -> RegistrationState:
    try:
        return RegistrationState(value)
    except ValueError:
        return RegistrationState.NONE


def _call_state(value: str) -> CallState:
    try:
        return CallState(value)
    except ValueError:
        return CallState.IDLE
