"""Capture plumbing for gallery screenshot generation."""

from __future__ import annotations

import time
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Callable, Protocol, cast

from yoyopod.cli.pi.gallery.fakes import (
    _FakeNetworkManager,
    _FakePeopleDirectory,
    _FakeVoipManager,
)
from yoyopod.cli.pi.gallery.fixtures import (
    _build_call_history_store,
    _build_contacts,
    _build_context,
    _build_music_service,
    _build_now_playing_screen,
    _build_power_screen,
    _build_power_snapshot,
    _build_talk_contact_screen,
    _build_voice_note_recording_screen,
    _build_voice_note_review_screen,
    _build_voice_note_sent_screen,
)

if TYPE_CHECKING:
    from yoyopod.communication import CallHistoryStore, VoIPManager
    from yoyopod.audio.music import LocalMusicService
    from yoyopod.people import PeopleDirectory
    from yoyopod.ui.display import Display


class _GalleryScreen(Protocol):
    """Minimal screen surface required for gallery capture."""

    def enter(self) -> None: ...

    def render(self) -> None: ...

    def exit(self) -> None: ...


@dataclass(frozen=True, slots=True)
class _CaptureSpec:
    """One deterministic screen capture target."""

    name: str
    build_screen: Callable[[], _GalleryScreen]
    prepare: Callable[[_GalleryScreen], None] | None = None


def _pump_display(display: "Display", duration_seconds: float) -> None:
    """Let LVGL flush and settle before capturing."""

    backend = display.get_ui_backend()
    if backend is None or not getattr(backend, "initialized", False):
        return

    deadline = time.monotonic() + max(0.01, duration_seconds)
    last_tick = time.monotonic()
    while time.monotonic() < deadline:
        now = time.monotonic()
        delta_ms = int(max(1.0, (now - last_tick) * 1000.0))
        last_tick = now
        backend.pump(delta_ms)
        time.sleep(0.016)


def _capture_screen(
    display: "Display",
    spec: _CaptureSpec,
    output_dir: Path,
    *,
    settle_seconds: float,
) -> None:
    """Render one screen state and save an LVGL readback."""

    screen = spec.build_screen()
    screen.enter()
    try:
        if spec.prepare is not None:
            spec.prepare(screen)
        screen.render()
        _pump_display(display, settle_seconds)

        adapter = display.get_adapter()
        save_readback = getattr(adapter, "save_screenshot_readback", None)
        save_shadow = getattr(adapter, "save_screenshot", None)
        if not callable(save_readback):
            raise RuntimeError("active display adapter does not support LVGL readback screenshots")
        if not callable(save_shadow):
            raise RuntimeError("active display adapter does not support shadow-buffer screenshots")

        output_path = output_dir / f"{spec.name}.png"
        if save_readback(str(output_path)):
            from loguru import logger

            logger.info("Captured {} via LVGL readback", output_path.name)
            return
        if save_shadow(str(output_path)):
            from loguru import logger

            logger.warning("Captured {} via shadow-buffer fallback", output_path.name)
            return
        raise RuntimeError(f"failed to save screenshot to {output_path}")
    finally:
        screen.exit()
        backend = display.get_ui_backend()
        if backend is not None and getattr(backend, "initialized", False):
            backend.clear()
            _pump_display(display, 0.05)


def _build_capture_specs(
    display: "Display",
    *,
    advance_ask_to_response: Callable[[object], None] | None = None,
) -> list[_CaptureSpec]:
    """Build the deterministic gallery sequence."""

    from yoyopod.ui.screens.music.playlist import PlaylistScreen
    from yoyopod.ui.screens.music.recent import RecentTracksScreen
    from yoyopod.ui.screens.navigation.ask import AskScreen
    from yoyopod.ui.screens.navigation.listen import ListenScreen
    from yoyopod.ui.screens.voip.call_history import CallHistoryScreen
    from yoyopod.ui.screens.voip.contact_list import ContactListScreen
    from yoyopod.ui.screens.voip.in_call import InCallScreen
    from yoyopod.ui.screens.voip.incoming_call import IncomingCallScreen
    from yoyopod.ui.screens.voip.outgoing_call import OutgoingCallScreen
    from yoyopod.ui.screens.voip.quick_call import CallScreen

    contacts = _build_contacts()
    people_directory = cast("PeopleDirectory", _FakePeopleDirectory(contacts))
    music_service = cast("LocalMusicService", _build_music_service())
    call_history_store = cast("CallHistoryStore", _build_call_history_store())
    power_snapshot = _build_power_snapshot()
    idle_voip_manager = cast("VoIPManager", _FakeVoipManager())
    ask_prepare = cast(Callable[[_GalleryScreen], None] | None, advance_ask_to_response)

    return [
        _CaptureSpec(
            "01_listen",
            lambda: ListenScreen(display, _build_context(), music_service=None),
        ),
        _CaptureSpec(
            "02_playlists",
            lambda: PlaylistScreen(display, _build_context(), music_service=music_service),
            prepare=lambda screen: setattr(screen, "selected_index", 1),
        ),
        _CaptureSpec(
            "03_recent",
            lambda: RecentTracksScreen(display, _build_context(), music_service=music_service),
            prepare=lambda screen: setattr(screen, "selected_index", 1),
        ),
        _CaptureSpec(
            "04_now_playing",
            lambda: _build_now_playing_screen(display, playback_state="playing"),
        ),
        _CaptureSpec(
            "04b_now_playing_paused",
            lambda: _build_now_playing_screen(display, playback_state="paused"),
        ),
        _CaptureSpec(
            "04c_now_playing_offline",
            lambda: _build_now_playing_screen(display, playback_state="offline"),
        ),
        _CaptureSpec(
            "05_talk",
            lambda: CallScreen(
                display,
                _build_context(),
                voip_manager=idle_voip_manager,
                people_directory=people_directory,
                call_history_store=call_history_store,
            ),
        ),
        _CaptureSpec(
            "06_talk_contact",
            lambda: _build_talk_contact_screen(display),
        ),
        _CaptureSpec(
            "07_contacts",
            lambda: ContactListScreen(
                display,
                _build_context(),
                voip_manager=idle_voip_manager,
                people_directory=people_directory,
            ),
            prepare=lambda screen: setattr(screen, "selected_index", 1),
        ),
        _CaptureSpec(
            "08_call_history",
            lambda: CallHistoryScreen(
                display,
                _build_context(),
                voip_manager=idle_voip_manager,
                call_history_store=call_history_store,
            ),
            prepare=lambda screen: setattr(screen, "selected_index", 1),
        ),
        _CaptureSpec(
            "09_voice_note_recording",
            lambda: _build_voice_note_recording_screen(display),
        ),
        _CaptureSpec(
            "09b_voice_note_review",
            lambda: _build_voice_note_review_screen(display),
        ),
        _CaptureSpec(
            "09c_voice_note_sent",
            lambda: _build_voice_note_sent_screen(display),
        ),
        _CaptureSpec(
            "10_ask_idle",
            lambda: AskScreen(display, _build_context()),
        ),
        _CaptureSpec(
            "11_ask_response",
            lambda: AskScreen(display, _build_context()),
            prepare=ask_prepare,
        ),
        _CaptureSpec(
            "12_power",
            lambda: _build_power_screen(display, power_snapshot=power_snapshot),
        ),
        _CaptureSpec(
            "12b_gps",
            lambda: _build_power_screen(
                display,
                power_snapshot=power_snapshot,
                network_manager=_FakeNetworkManager(),
            ),
            prepare=lambda screen: setattr(screen, "page_index", 2),
        ),
        _CaptureSpec(
            "13_time",
            lambda: _build_power_screen(display, power_snapshot=power_snapshot),
            prepare=lambda screen: setattr(screen, "page_index", 1),
        ),
        _CaptureSpec(
            "14_care",
            lambda: _build_power_screen(display, power_snapshot=power_snapshot),
            prepare=lambda screen: setattr(screen, "page_index", 2),
        ),
        _CaptureSpec(
            "15_incoming_call",
            lambda: IncomingCallScreen(
                display,
                _build_context(),
                voip_manager=idle_voip_manager,
                caller_address="sip:mama@example.com",
                caller_name="Mama",
            ),
        ),
        _CaptureSpec(
            "16_outgoing_call",
            lambda: OutgoingCallScreen(
                display,
                _build_context(),
                voip_manager=cast(
                    "VoIPManager",
                    _FakeVoipManager(
                        caller_info={
                            "display_name": "Papa",
                            "address": "sip:papa@example.com",
                        }
                    ),
                ),
            ),
        ),
        _CaptureSpec(
            "17_in_call",
            lambda: InCallScreen(
                display,
                _build_context(),
                voip_manager=cast(
                    "VoIPManager",
                    _FakeVoipManager(
                        caller_info={"display_name": "Mama"},
                        duration_seconds=187,
                        muted=False,
                    ),
                ),
            ),
        ),
        _CaptureSpec(
            "17b_in_call_muted",
            lambda: InCallScreen(
                display,
                _build_context(),
                voip_manager=cast(
                    "VoIPManager",
                    _FakeVoipManager(
                        caller_info={"display_name": "Mama"},
                        duration_seconds=187,
                        muted=True,
                    ),
                ),
            ),
        ),
    ]
