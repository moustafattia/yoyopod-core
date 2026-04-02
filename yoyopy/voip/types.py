"""Shared VoIP models and typed backend events."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import TypeAlias


class RegistrationState(Enum):
    """SIP registration states."""

    NONE = "none"
    PROGRESS = "progress"
    OK = "ok"
    CLEARED = "cleared"
    FAILED = "failed"


class CallState(Enum):
    """Call states."""

    IDLE = "idle"
    INCOMING = "incoming"
    OUTGOING = "outgoing_init"
    OUTGOING_PROGRESS = "outgoing_progress"
    OUTGOING_RINGING = "outgoing_ringing"
    OUTGOING_EARLY_MEDIA = "outgoing_early_media"
    CONNECTED = "connected"
    STREAMS_RUNNING = "streams_running"
    PAUSED = "paused"
    PAUSED_BY_REMOTE = "paused_by_remote"
    UPDATED_BY_REMOTE = "updated_by_remote"
    RELEASED = "released"
    ERROR = "error"
    END = "end"


@dataclass(slots=True)
class VoIPConfig:
    """VoIP configuration."""

    sip_server: str = "sip.linphone.org"
    sip_username: str = ""
    sip_password: str = ""
    sip_password_ha1: str = ""
    sip_identity: str = ""
    transport: str = "tcp"
    stun_server: str = ""
    linphonec_path: str = "/usr/bin/linphonec"
    playback_dev_id: str = "ALSA: plughw:1"
    ringer_dev_id: str = "ALSA: plughw:1"
    capture_dev_id: str = "ALSA: plughw:1"
    media_dev_id: str = "ALSA: plughw:1"

    @staticmethod
    def from_config_manager(config_manager) -> "VoIPConfig":
        """Create a VoIPConfig from the current config manager."""

        return VoIPConfig(
            sip_server=config_manager.get_sip_server(),
            sip_username=config_manager.get_sip_username(),
            sip_password=config_manager.get_sip_password(),
            sip_password_ha1=config_manager.get_sip_password_ha1(),
            sip_identity=config_manager.get_sip_identity(),
            transport=config_manager.get_transport(),
            stun_server=config_manager.get_stun_server(),
            linphonec_path=config_manager.get_linphonec_path(),
            playback_dev_id=config_manager.get_playback_device_id(),
            ringer_dev_id=config_manager.get_ringer_device_id(),
            capture_dev_id=config_manager.get_capture_device_id(),
            media_dev_id=config_manager.get_media_device_id(),
        )


@dataclass(frozen=True, slots=True)
class RegistrationStateChanged:
    """Typed event emitted when SIP registration changes."""

    state: RegistrationState


@dataclass(frozen=True, slots=True)
class CallStateChanged:
    """Typed event emitted when the active call state changes."""

    state: CallState


@dataclass(frozen=True, slots=True)
class IncomingCallDetected:
    """Typed event emitted when an incoming call address is detected."""

    caller_address: str


@dataclass(frozen=True, slots=True)
class BackendStopped:
    """Typed event emitted when the backend process exits unexpectedly."""

    reason: str = ""


VoIPEvent: TypeAlias = (
    RegistrationStateChanged
    | CallStateChanged
    | IncomingCallDetected
    | BackendStopped
)
