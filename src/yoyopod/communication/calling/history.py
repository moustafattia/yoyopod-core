"""Compatibility exports for the relocated Talk call-history models."""

from yoyopod.integrations.call.history import (
    CallDirection,
    CallHistoryEntry,
    CallHistoryStore,
    CallOutcome,
)

__all__ = [
    "CallDirection",
    "CallHistoryEntry",
    "CallHistoryStore",
    "CallOutcome",
]
