"""VoIP backend protocol and Liblinphone-backed implementation."""

from __future__ import annotations

import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Optional, Protocol

from loguru import logger

from yoyopy.voip.liblinphone_binding import (
    LiblinphoneBinding,
    LiblinphoneNativeEvent,
)
from yoyopy.voip.models import (
    BackendStopped,
    CallState,
    CallStateChanged,
    IncomingCallDetected,
    MessageDeliveryChanged,
    MessageDirection,
    MessageDownloadCompleted,
    MessageFailed,
    MessageKind,
    MessageReceived,
    MessageDeliveryState,
    RegistrationState,
    RegistrationStateChanged,
    VoIPConfig,
    VoIPEvent,
    VoIPMessageRecord,
)


class VoIPBackend(Protocol):
    """Backend contract for SIP and messaging implementations used by VoIPManager."""

    def start(self) -> bool:
        """Start the backend and begin emitting events."""

    def stop(self) -> None:
        """Stop the backend and release resources."""

    def iterate(self) -> None:
        """Advance the backend once on the coordinator thread."""

    def make_call(self, sip_address: str) -> bool:
        """Initiate an outgoing call."""

    def answer_call(self) -> bool:
        """Answer the current incoming call."""

    def reject_call(self) -> bool:
        """Reject the current incoming call."""

    def hangup(self) -> bool:
        """Terminate the current call."""

    def mute(self) -> bool:
        """Mute the current call microphone."""

    def unmute(self) -> bool:
        """Unmute the current call microphone."""

    def send_text_message(self, sip_address: str, text: str) -> str | None:
        """Send a text message and return its backend identifier when available."""

    def start_voice_note_recording(self, file_path: str) -> bool:
        """Begin recording a voice note to the provided file path."""

    def stop_voice_note_recording(self) -> int | None:
        """Stop the active recording and return its duration in milliseconds."""

    def cancel_voice_note_recording(self) -> bool:
        """Cancel and discard the active recording."""

    def send_voice_note(
        self,
        sip_address: str,
        *,
        file_path: str,
        duration_ms: int,
        mime_type: str,
    ) -> str | None:
        """Send a recorded voice note and return its backend identifier when available."""

    def on_event(self, callback: Callable[[VoIPEvent], None]) -> None:
        """Register a typed backend-event listener."""


@dataclass(slots=True)
class _AlsaMixerConfig:
    speaker_raw: int
    capture_raw: int


class LiblinphoneBackend:
    """Production VoIP backend driven by the native Liblinphone shim."""

    def __init__(
        self,
        config: VoIPConfig,
        *,
        binding: LiblinphoneBinding | None = None,
    ) -> None:
        self.config = config
        self.binding = binding or LiblinphoneBinding.try_load()
        self.running = False
        self.event_callbacks: list[Callable[[VoIPEvent], None]] = []

    def on_event(self, callback: Callable[[VoIPEvent], None]) -> None:
        self.event_callbacks.append(callback)

    def start(self) -> bool:
        if self.running:
            return True

        if self.binding is None:
            logger.error("Liblinphone backend requested but native shim is unavailable")
            return False

        try:
            factory_config_path = self._resolve_factory_config_path()
            conference_factory_uri = self.config.effective_conference_factory_uri()
            file_transfer_server_url = self.config.effective_file_transfer_server_url()
            lime_server_url = self.config.effective_lime_server_url()
            self.binding.init()
            self._configure_alsa_mixer()
            if not self.config.conference_factory_uri and conference_factory_uri:
                logger.info(
                    "Using inferred conference factory {} for hosted account {}",
                    conference_factory_uri,
                    self.config.sip_server,
                )
            if not self.config.file_transfer_server_url and file_transfer_server_url:
                logger.info(
                    "Using inferred file-transfer server {} for hosted account {}",
                    file_transfer_server_url,
                    self.config.sip_server,
                )
            if not self.config.lime_server_url and lime_server_url:
                logger.info(
                    "Using inferred LIME server {} for hosted account {}",
                    lime_server_url,
                    self.config.sip_server,
                )
            self.binding.start(
                sip_server=self.config.sip_server,
                sip_username=self.config.sip_username,
                sip_password=self.config.sip_password,
                sip_password_ha1=self.config.sip_password_ha1,
                sip_identity=self.config.sip_identity,
                factory_config_path=factory_config_path,
                transport=self.config.transport,
                stun_server=self.config.stun_server,
                conference_factory_uri=conference_factory_uri,
                file_transfer_server_url=file_transfer_server_url,
                lime_server_url=lime_server_url,
                auto_download_incoming_voice_recordings=(
                    self.config.auto_download_incoming_voice_recordings
                ),
                playback_device_id=self.config.playback_dev_id,
                ringer_device_id=self.config.ringer_dev_id,
                capture_device_id=self.config.capture_dev_id,
                media_device_id=self.config.media_dev_id,
                echo_cancellation=True,
                mic_gain=self.config.mic_gain,
                speaker_volume=self.config.speaker_volume,
                voice_note_store_dir=self.config.voice_note_store_dir,
            )
            self.running = True
            logger.info("Liblinphone backend started successfully")
            return True
        except Exception as exc:
            logger.error("Failed to start Liblinphone backend: {}", exc)
            try:
                if self.binding is not None:
                    self.binding.shutdown()
            except Exception:
                logger.debug("Liblinphone shutdown after failed start also failed")
            self.running = False
            return False

    def _resolve_factory_config_path(self) -> str:
        path = Path(self.config.factory_config_path)
        if not path.is_absolute():
            repo_root = Path(__file__).resolve().parents[2]
            candidate = repo_root / path
            path = candidate if candidate.exists() else (Path.cwd() / path)
        if not path.exists():
            logger.warning("Liblinphone factory config not found at {}", path)
            return ""
        return str(path)

    def stop(self) -> None:
        if self.binding is None:
            self.running = False
            return

        try:
            self.binding.stop()
        finally:
            self.binding.shutdown()
            self.running = False

    def iterate(self) -> None:
        if not self.running or self.binding is None:
            return

        try:
            self.binding.iterate()
            while True:
                event = self.binding.poll_event()
                if event is None:
                    break
                self._emit_native_event(event)
        except Exception as exc:
            logger.error("Liblinphone iterate failed: {}", exc)
            self.running = False
            self._emit(BackendStopped(reason=str(exc)))

    def make_call(self, sip_address: str) -> bool:
        return self._call_binding(lambda: self.binding.make_call(sip_address))

    def answer_call(self) -> bool:
        return self._call_binding(self.binding.answer_call)

    def reject_call(self) -> bool:
        return self._call_binding(self.binding.reject_call)

    def hangup(self) -> bool:
        return self._call_binding(self.binding.hangup)

    def mute(self) -> bool:
        return self._call_binding(lambda: self.binding.set_muted(True))

    def unmute(self) -> bool:
        return self._call_binding(lambda: self.binding.set_muted(False))

    def send_text_message(self, sip_address: str, text: str) -> str | None:
        if not self.running or self.binding is None:
            return None
        try:
            return self.binding.send_text_message(sip_address, text)
        except Exception as exc:
            logger.error("Failed to send text message to {}: {}", sip_address, exc)
            return None

    def start_voice_note_recording(self, file_path: str) -> bool:
        return self._call_binding(lambda: self.binding.start_voice_recording(file_path))

    def stop_voice_note_recording(self) -> int | None:
        if not self.running or self.binding is None:
            return None
        try:
            return self.binding.stop_voice_recording()
        except Exception as exc:
            logger.error("Failed to stop voice-note recording: {}", exc)
            return None

    def cancel_voice_note_recording(self) -> bool:
        return self._call_binding(self.binding.cancel_voice_recording)

    def send_voice_note(
        self,
        sip_address: str,
        *,
        file_path: str,
        duration_ms: int,
        mime_type: str,
    ) -> str | None:
        if not self.running or self.binding is None:
            return None
        try:
            return self.binding.send_voice_note(
                sip_address,
                file_path=file_path,
                duration_ms=duration_ms,
                mime_type=mime_type,
            )
        except Exception as exc:
            logger.error("Failed to send voice note to {}: {}", sip_address, exc)
            return None

    def _call_binding(self, operation: Callable[[], None]) -> bool:
        if not self.running or self.binding is None:
            logger.error("Cannot execute VoIP operation: Liblinphone backend not running")
            return False
        try:
            operation()
            return True
        except Exception as exc:
            logger.error("Liblinphone operation failed: {}", exc)
            return False

    def _emit(self, event: VoIPEvent) -> None:
        for callback in self.event_callbacks:
            try:
                callback(event)
            except Exception as exc:
                logger.error("Error in VoIP backend callback: {}", exc)

    def _emit_native_event(self, event: LiblinphoneNativeEvent) -> None:
        if event.type == 1:
            self._emit(RegistrationStateChanged(state=self._registration_state(event.registration_state)))
            return

        if event.type == 2:
            self._emit(CallStateChanged(state=self._call_state(event.call_state)))
            return

        if event.type == 3:
            self._emit(IncomingCallDetected(caller_address=event.peer_sip_address))
            return

        if event.type == 4:
            self.running = False
            self._emit(BackendStopped(reason=event.reason))
            return

        if event.type == 5:
            logger.info(
                "Liblinphone incoming message: id={} kind={} peer={} file={}",
                event.message_id,
                self._message_kind(event.message_kind).value,
                event.peer_sip_address,
                event.local_file_path,
            )
            self._emit(MessageReceived(message=self._message_record(event)))
            return

        if event.type == 6:
            logger.info(
                "Liblinphone message delivery: id={} state={} peer={} reason={}",
                event.message_id,
                self._delivery_state(event.message_delivery_state).value,
                event.peer_sip_address,
                event.reason,
            )
            self._emit(
                MessageDeliveryChanged(
                    message_id=event.message_id,
                    delivery_state=self._delivery_state(event.message_delivery_state),
                    local_file_path=event.local_file_path,
                    error=event.reason,
                )
            )
            return

        if event.type == 7:
            logger.info(
                "Liblinphone message download complete: id={} file={} mime={}",
                event.message_id,
                event.local_file_path,
                event.mime_type,
            )
            self._emit(
                MessageDownloadCompleted(
                    message_id=event.message_id,
                    local_file_path=event.local_file_path,
                    mime_type=event.mime_type,
                )
            )
            return

        if event.type == 8:
            logger.warning(
                "Liblinphone message failed: id={} reason={}",
                event.message_id,
                event.reason,
            )
            self._emit(MessageFailed(message_id=event.message_id, reason=event.reason))

    def _message_record(self, event: LiblinphoneNativeEvent) -> VoIPMessageRecord:
        timestamp = self._iso_now()
        return VoIPMessageRecord(
            id=event.message_id,
            peer_sip_address=event.peer_sip_address,
            sender_sip_address=event.sender_sip_address,
            recipient_sip_address=event.recipient_sip_address,
            kind=self._message_kind(event.message_kind),
            direction=self._message_direction(event.message_direction),
            delivery_state=self._delivery_state(event.message_delivery_state),
            created_at=timestamp,
            updated_at=timestamp,
            text=event.text,
            local_file_path=event.local_file_path,
            mime_type=event.mime_type,
            duration_ms=max(0, int(event.duration_ms)),
            unread=bool(event.unread),
        )

    @staticmethod
    def _registration_state(value: int) -> RegistrationState:
        mapping = {
            1: RegistrationState.PROGRESS,
            2: RegistrationState.OK,
            3: RegistrationState.CLEARED,
            4: RegistrationState.FAILED,
        }
        return mapping.get(value, RegistrationState.NONE)

    @staticmethod
    def _call_state(value: int) -> CallState:
        mapping = {
            1: CallState.INCOMING,
            2: CallState.OUTGOING,
            3: CallState.OUTGOING_PROGRESS,
            4: CallState.OUTGOING_RINGING,
            5: CallState.OUTGOING_EARLY_MEDIA,
            6: CallState.CONNECTED,
            7: CallState.STREAMS_RUNNING,
            8: CallState.PAUSED,
            9: CallState.PAUSED_BY_REMOTE,
            10: CallState.UPDATED_BY_REMOTE,
            11: CallState.RELEASED,
            12: CallState.ERROR,
            13: CallState.END,
        }
        return mapping.get(value, CallState.IDLE)

    @staticmethod
    def _message_kind(value: int) -> MessageKind:
        return MessageKind.VOICE_NOTE if value == 2 else MessageKind.TEXT

    @staticmethod
    def _message_direction(value: int) -> MessageDirection:
        return MessageDirection.OUTGOING if value == 2 else MessageDirection.INCOMING

    @staticmethod
    def _delivery_state(value: int) -> MessageDeliveryState:
        mapping = {
            1: MessageDeliveryState.QUEUED,
            2: MessageDeliveryState.SENDING,
            3: MessageDeliveryState.SENT,
            4: MessageDeliveryState.DELIVERED,
            5: MessageDeliveryState.FAILED,
        }
        return mapping.get(value, MessageDeliveryState.FAILED)

    @staticmethod
    def _iso_now() -> str:
        from datetime import datetime, timezone

        return datetime.now(timezone.utc).isoformat()

    def _configure_alsa_mixer(self) -> None:
        mixer = self._alsa_mixer_config()
        card = "1"
        commands = [
            f"amixer -c {card} sset 'Speaker' {mixer.speaker_raw}",
            f"amixer -c {card} sset 'Playback' 255",
            f"amixer -c {card} sset 'Headphone' {mixer.speaker_raw}",
            f"amixer -c {card} sset 'Capture' {mixer.capture_raw}",
            f"amixer -c {card} sset 'ADC PCM' 195",
            f"amixer -c {card} sset 'Left Input Boost Mixer LINPUT1' 1",
            f"amixer -c {card} sset 'Right Input Boost Mixer RINPUT1' 1",
        ]

        for command in commands:
            try:
                subprocess.run(command, shell=True, capture_output=True, timeout=5, check=False)
            except Exception as exc:
                logger.warning("ALSA mixer command failed: {}: {}", command, exc)

    def _alsa_mixer_config(self) -> _AlsaMixerConfig:
        speaker_pct = min(100, max(0, self.config.speaker_volume))
        capture_pct = min(100, max(0, self.config.mic_gain))
        return _AlsaMixerConfig(
            speaker_raw=int(85 + speaker_pct * 0.30),
            capture_raw=int(14 + capture_pct * 0.16),
        )


class MockVoIPBackend:
    """Simple in-memory backend used for unit tests."""

    def __init__(self, start_result: bool = True) -> None:
        self.start_result = start_result
        self.running = False
        self.commands: list[str] = []
        self.event_callbacks: list[Callable[[VoIPEvent], None]] = []
        self.make_call_result = True
        self.answer_result = True
        self.reject_result = True
        self.hangup_result = True
        self.mute_result = True
        self.unmute_result = True
        self.recording_active = False
        self.recording_path = ""
        self.recording_duration_ms = 1500
        self.next_text_message_id = "mock-text-1"
        self.next_voice_note_id = "mock-note-1"

    def on_event(self, callback: Callable[[VoIPEvent], None]) -> None:
        self.event_callbacks.append(callback)

    def emit(self, event: VoIPEvent) -> None:
        for callback in self.event_callbacks:
            callback(event)

    def start(self) -> bool:
        self.running = self.start_result
        return self.start_result

    def stop(self) -> None:
        self.running = False
        self.recording_active = False

    def iterate(self) -> None:
        return

    def make_call(self, sip_address: str) -> bool:
        self.commands.append(f"call {sip_address}")
        return self.make_call_result

    def answer_call(self) -> bool:
        self.commands.append("answer")
        return self.answer_result

    def reject_call(self) -> bool:
        self.commands.append("decline")
        return self.reject_result

    def hangup(self) -> bool:
        self.commands.append("terminate")
        return self.hangup_result

    def mute(self) -> bool:
        self.commands.append("mute")
        return self.mute_result

    def unmute(self) -> bool:
        self.commands.append("unmute")
        return self.unmute_result

    def send_text_message(self, sip_address: str, text: str) -> str | None:
        self.commands.append(f"text {sip_address} {text}")
        return self.next_text_message_id

    def start_voice_note_recording(self, file_path: str) -> bool:
        self.recording_active = True
        self.recording_path = file_path
        self.commands.append(f"record-start {file_path}")
        return True

    def stop_voice_note_recording(self) -> int | None:
        if not self.recording_active:
            return None
        self.recording_active = False
        self.commands.append("record-stop")
        return self.recording_duration_ms

    def cancel_voice_note_recording(self) -> bool:
        self.recording_active = False
        self.commands.append("record-cancel")
        return True

    def send_voice_note(
        self,
        sip_address: str,
        *,
        file_path: str,
        duration_ms: int,
        mime_type: str,
    ) -> str | None:
        self.commands.append(f"voice-note {sip_address} {Path(file_path).name} {duration_ms} {mime_type}")
        return self.next_voice_note_id
