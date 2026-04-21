"""Calling-domain implementations and app-facing helpers."""

from __future__ import annotations

from importlib import import_module
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from yoyopod.communication.calling.backend_protocol import VoIPIterateMetrics
    from yoyopod.integrations.call.history import CallHistoryEntry, CallHistoryStore
    from yoyopod.integrations.call.manager import VoIPManager
    from yoyopod.integrations.call.voice_notes import VoiceNoteDraft


_LAZY_EXPORTS = {
    "CallHistoryEntry": "yoyopod.integrations.call.history",
    "CallHistoryStore": "yoyopod.integrations.call.history",
    "VoIPIterateMetrics": "yoyopod.communication.calling.backend_protocol",
    "VoIPManager": "yoyopod.integrations.call.manager",
    "VoiceNoteDraft": "yoyopod.integrations.call.voice_notes",
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
