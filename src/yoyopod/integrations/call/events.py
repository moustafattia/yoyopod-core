"""Typed scaffold events owned by the canonical call integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class CallHistoryUpdatedEvent:
    """Published when missed-call history counters change."""

    unread_count: int
    recent_preview: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class VoiceNoteSummaryChangedEvent:
    """Published when voice-note unread counts or summaries change."""

    unread_count: int
    unread_by_address: dict[str, int]
    latest_by_contact: dict[str, dict[str, object]]
