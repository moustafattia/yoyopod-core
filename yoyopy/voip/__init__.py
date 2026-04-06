"""
VoIP module for YoyoPod.

Exports the app-facing VoIP manager, backend protocol/implementations,
and the shared SIP event and configuration types.
"""

from yoyopy.voip.backend import (
    LinphonecBackend,
    MockVoIPBackend,
    VoIPBackend,
)
from yoyopy.voip.manager import VoIPManager
from yoyopy.voip.history import CallHistoryEntry, CallHistoryStore
from yoyopy.voip.models import (
    BackendStopped,
    CallState,
    CallStateChanged,
    IncomingCallDetected,
    RegistrationState,
    RegistrationStateChanged,
    VoIPConfig,
    VoIPEvent,
)

__all__ = [
    "VoIPManager",
    "CallHistoryEntry",
    "CallHistoryStore",
    "VoIPBackend",
    "LinphonecBackend",
    "MockVoIPBackend",
    "VoIPConfig",
    "RegistrationState",
    "CallState",
    "RegistrationStateChanged",
    "CallStateChanged",
    "IncomingCallDetected",
    "BackendStopped",
    "VoIPEvent",
]
