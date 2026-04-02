#!/usr/bin/env python3
"""Smoke tests for current display and screen rendering paths."""

import pytest

from yoyopy.app_context import AppContext
from yoyopy.ui.display import Display
from yoyopy.ui.screens import HomeScreen, MenuScreen, NowPlayingScreen


@pytest.fixture
def display():
    """Create a simulation display and clean it up after the test."""
    test_display = Display(simulate=True)
    try:
        yield test_display
    finally:
        test_display.cleanup()


@pytest.fixture
def context() -> AppContext:
    """Create an app context with a demo playlist loaded."""
    app_context = AppContext()
    playlist = app_context.create_demo_playlist()
    app_context.set_playlist(playlist)
    app_context.update_system_status(battery=85, signal=3, connected=True)
    app_context.play()
    app_context.playback.position = 45.0
    return app_context


def test_core_screens_render_without_hardware(display: Display, context: AppContext) -> None:
    """Core screens should render cleanly in simulation mode."""
    screens = [
        HomeScreen(display, context),
        MenuScreen(display, context, items=["Music", "Podcasts", "Stories", "Settings"]),
        NowPlayingScreen(display, context),
    ]

    for screen in screens:
        screen.enter()
        screen.render()
        screen.exit()


def test_rendering_still_works_after_state_changes(display: Display, context: AppContext) -> None:
    """Menu and now playing screens should still render after local state changes."""
    menu_screen = MenuScreen(display, context, items=["Music", "Podcasts", "Stories", "Settings"])
    now_playing_screen = NowPlayingScreen(display, context)

    menu_screen.select_next()
    menu_screen.select_next()
    menu_screen.render()

    context.next_track()
    context.pause()
    now_playing_screen.render()
