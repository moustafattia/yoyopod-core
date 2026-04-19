"""
Screens module for YoyoPod UI.

Provides screen implementations organized by feature:
- base: Screen base class
- manager: ScreenManager for navigation
- navigation: Home, hub, and route-selection screens
- system: Device status and setup screens
- music: Now Playing and Playlist screens
- voip: Call-related screens
"""

from __future__ import annotations

from yoyopod.ui._lazy_imports import exported_dir, load_attr

_EXPORTS = {
    "Screen": ("yoyopod.ui.screens.base", "Screen"),
    "ScreenView": ("yoyopod.ui.screens.view", "ScreenView"),
    "ScreenManager": ("yoyopod.ui.screens.manager", "ScreenManager"),
    "NavigationRequest": ("yoyopod.ui.screens.router", "NavigationRequest"),
    "ScreenRouter": ("yoyopod.ui.screens.router", "ScreenRouter"),
    "HubScreen": ("yoyopod.ui.screens.navigation.hub", "HubScreen"),
    "HomeScreen": ("yoyopod.ui.screens.navigation.home", "HomeScreen"),
    "ListenScreen": ("yoyopod.ui.screens.navigation.listen", "ListenScreen"),
    "MenuScreen": ("yoyopod.ui.screens.navigation.menu", "MenuScreen"),
    "AskScreen": ("yoyopod.ui.screens.navigation.ask", "AskScreen"),
    "PowerScreen": ("yoyopod.ui.screens.system.power", "PowerScreen"),
    "NowPlayingScreen": ("yoyopod.ui.screens.music.now_playing", "NowPlayingScreen"),
    "PlaylistScreen": ("yoyopod.ui.screens.music.playlist", "PlaylistScreen"),
    "RecentTracksScreen": ("yoyopod.ui.screens.music.recent", "RecentTracksScreen"),
    "CallScreen": ("yoyopod.ui.screens.voip.quick_call", "CallScreen"),
    "CallHistoryScreen": (
        "yoyopod.ui.screens.voip.call_history",
        "CallHistoryScreen",
    ),
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
    "VoiceNoteScreen": ("yoyopod.ui.screens.voip.voice_note", "VoiceNoteScreen"),
}

__all__ = [
    # Base & Manager
    'Screen',
    'ScreenView',
    'ScreenManager',
    'NavigationRequest',
    'ScreenRouter',
    # Navigation
    'HubScreen',
    'HomeScreen',
    'ListenScreen',
    'MenuScreen',
    'AskScreen',
    'PowerScreen',
    # Music
    'NowPlayingScreen',
    'PlaylistScreen',
    'RecentTracksScreen',
    # VoIP
    'CallScreen',
    'CallHistoryScreen',
    'IncomingCallScreen',
    'OutgoingCallScreen',
    'InCallScreen',
    'ContactListScreen',
    'TalkContactScreen',
    'VoiceNoteScreen',
]


def __getattr__(name: str) -> object:
    """Resolve screen exports on demand instead of importing all screens eagerly."""

    return load_attr(_EXPORTS, __name__, name)


def __dir__() -> list[str]:
    """Expose lazy re-exports to dir() and import tooling."""

    return exported_dir(globals(), _EXPORTS)
