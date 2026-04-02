"""
YoyoPod package.

Integrated Raspberry Pi application for button-driven music playback and SIP calling.
"""

from yoyopy.event_bus import EventBus
from yoyopy.events import (
    CallEndedEvent,
    CallStateChangedEvent,
    IncomingCallEvent,
    MusicAvailabilityChangedEvent,
    PlaybackStateChangedEvent,
    RegistrationChangedEvent,
    ScreenChangedEvent,
    TrackChangedEvent,
    VoIPAvailabilityChangedEvent,
)
from yoyopy.fsm import CallFSM, CallInterruptionPolicy, CallSessionState, MusicFSM, MusicState

__version__ = "0.1.0"
__author__ = "YoyoPod Team"

__all__ = [
    "EventBus",
    "IncomingCallEvent",
    "CallStateChangedEvent",
    "CallEndedEvent",
    "RegistrationChangedEvent",
    "ScreenChangedEvent",
    "VoIPAvailabilityChangedEvent",
    "TrackChangedEvent",
    "PlaybackStateChangedEvent",
    "MusicAvailabilityChangedEvent",
    "MusicFSM",
    "MusicState",
    "CallFSM",
    "CallSessionState",
    "CallInterruptionPolicy",
]
