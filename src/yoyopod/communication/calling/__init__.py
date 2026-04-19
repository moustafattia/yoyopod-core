"""Calling-domain implementations and app-facing helpers."""

from __future__ import annotations

from importlib import import_module
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from yoyopod.communication.calling.backend_protocol import VoIPIterateMetrics
    from yoyopod.communication.calling.history import CallHistoryEntry, CallHistoryStore
    from yoyopod.communication.calling.manager import VoIPManager
    from yoyopod.communication.calling.voice_notes import VoiceNoteDraft


_LAZY_EXPORTS = {
    "CallHistoryEntry": "yoyopod.communication.calling.history",
    "CallHistoryStore": "yoyopod.communication.calling.history",
    "VoIPIterateMetrics": "yoyopod.communication.calling.backend_protocol",
    "VoIPManager": "yoyopod.communication.calling.manager",
    "VoiceNoteDraft": "yoyopod.communication.calling.voice_notes",
}


def __getattr__(name: str) -> Any:
    module_name = _LAZY_EXPORTS.get(name)
    if module_name is None:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
    return getattr(import_module(module_name), name)


__all__ = [
    "CallHistoryEntry",
    "CallHistoryStore",
    "VoIPIterateMetrics",
    "VoIPManager",
    "VoiceNoteDraft",
]
