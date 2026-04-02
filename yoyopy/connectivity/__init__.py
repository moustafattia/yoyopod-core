"""
Connectivity module for YoyoPod.
Manages 4G/LTE connection, WiFi fallback, network status, and VoIP calling.
"""

from yoyopy.connectivity.voip_backend import (
    LinphonecBackend,
    MockVoIPBackend,
    VoIPBackend,
)
from yoyopy.connectivity.voip_manager import VoIPManager
from yoyopy.connectivity.voip_types import (
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
