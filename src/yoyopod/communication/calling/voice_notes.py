"""Compatibility exports for the relocated Talk voice-note service."""

from yoyopod.integrations.call.voice_notes import (
    VOICE_NOTE_CONTAINER_GAIN_DB,
    VOICE_NOTE_SEND_TIMEOUT_SECONDS,
    VoiceNoteDraft,
    VoiceNoteService,
)

__all__ = [
    "VOICE_NOTE_CONTAINER_GAIN_DB",
    "VOICE_NOTE_SEND_TIMEOUT_SECONDS",
    "VoiceNoteDraft",
    "VoiceNoteService",
]
