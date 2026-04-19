"""
UI module for YoyoPod.

Provides display management, input handling, and screen navigation.
"""

from __future__ import annotations

from yoyopod.ui._lazy_imports import exported_dir, load_attr

_EXPORTS = {
    "Display": ("yoyopod.ui.display.manager", "Display"),
    "DisplayHAL": ("yoyopod.ui.display.hal", "DisplayHAL"),
    "InputManager": ("yoyopod.ui.input", "InputManager"),
    "InputAction": ("yoyopod.ui.input", "InputAction"),
    "InteractionProfile": ("yoyopod.ui.input", "InteractionProfile"),
    "Screen": ("yoyopod.ui.screens.base", "Screen"),
    "ScreenManager": ("yoyopod.ui.screens.manager", "ScreenManager"),
    "HomeScreen": ("yoyopod.ui.screens.navigation.home", "HomeScreen"),
    "MenuScreen": ("yoyopod.ui.screens.navigation.menu", "MenuScreen"),
    "NowPlayingScreen": ("yoyopod.ui.screens.music.now_playing", "NowPlayingScreen"),
    "PlaylistScreen": ("yoyopod.ui.screens.music.playlist", "PlaylistScreen"),
    "CallScreen": ("yoyopod.ui.screens.voip.quick_call", "CallScreen"),
    "IncomingCallScreen": (
        "yoyopod.ui.screens.voip.incoming_call",
        "IncomingCallScreen",
    ),
    "OutgoingCallScreen": (
        "yoyopod.ui.screens.voip.outgoing_call",
        "OutgoingCallScreen",
    ),
    "InCallScreen": ("yoyopod.ui.screens.voip.in_call", "InCallScreen"),
    "ContactListScreen": (
        "yoyopod.ui.screens.voip.contact_list",
        "ContactListScreen",
    ),
    "TalkContactScreen": (
        "yoyopod.ui.screens.voip.talk_contact",
        "TalkContactScreen",
    ),
}

__all__ = [
    # Display
    'Display',
    'DisplayHAL',
    # Input
    'InputManager',
    'InputAction',
    'InteractionProfile',
    # Screens
    'Screen',
    'ScreenManager',
    'HomeScreen',
    'MenuScreen',
    'NowPlayingScreen',
    'PlaylistScreen',
    'CallScreen',
    'IncomingCallScreen',
    'OutgoingCallScreen',
    'InCallScreen',
    'ContactListScreen',
    'TalkContactScreen',
]


def __getattr__(name: str) -> object:
    """Resolve UI re-exports without importing every screen package at import time."""

    return load_attr(_EXPORTS, __name__, name)


def __dir__() -> list[str]:
    """Expose lazy re-exports to dir() and import tooling."""

    return exported_dir(globals(), _EXPORTS)
