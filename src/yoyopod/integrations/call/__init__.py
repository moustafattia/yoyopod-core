"""Canonical public seam for call-domain runtime ownership."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from yoyopod.communication.calling.backend_protocol import VoIPIterateMetrics
    from yoyopod.integrations.call.history import CallHistoryEntry, CallHistoryStore
    from yoyopod.integrations.call.manager import VoIPManager
    from yoyopod.integrations.call.voice_notes import VoiceNoteDraft, VoiceNoteService


_PUBLIC_EXPORTS = {
    "CallHistoryEntry": ("yoyopod.integrations.call.history", "CallHistoryEntry"),
    "CallHistoryStore": ("yoyopod.integrations.call.history", "CallHistoryStore"),
    "VoIPIterateMetrics": (
        "yoyopod.communication.calling.backend_protocol",
        "VoIPIterateMetrics",
    ),
    "VoIPManager": ("yoyopod.integrations.call.manager", "VoIPManager"),
    "VoiceNoteDraft": ("yoyopod.integrations.call.voice_notes", "VoiceNoteDraft"),
    "VoiceNoteService": ("yoyopod.integrations.call.voice_notes", "VoiceNoteService"),
}


def __getattr__(name: str) -> Any:
    """Load public call exports lazily to keep communication imports acyclic."""

    try:
        module_name, attribute = _PUBLIC_EXPORTS[name]
    except KeyError as exc:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}") from exc

    module = __import__(module_name, fromlist=[attribute])
    return getattr(module, attribute)


__all__ = [
    "CallHistoryEntry",
    "CallHistoryStore",
    "VoIPIterateMetrics",
    "VoIPManager",
    "VoiceNoteDraft",
    "VoiceNoteService",
]
