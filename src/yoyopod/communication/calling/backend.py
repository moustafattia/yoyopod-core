"""Compatibility exports for VoIP backend implementations."""

from yoyopod.communication.calling.backend_protocol import VoIPBackend, VoIPIterateMetrics
from yoyopod.communication.calling.liblinphone_backend import LiblinphoneBackend
from yoyopod.communication.calling.mock_backend import MockVoIPBackend

__all__ = [
    "LiblinphoneBackend",
    "MockVoIPBackend",
    "VoIPBackend",
    "VoIPIterateMetrics",
]
