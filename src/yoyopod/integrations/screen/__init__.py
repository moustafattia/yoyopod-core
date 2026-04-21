"""Screen integration scaffold for the Phase A spine rewrite."""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Any

from yoyopod.core.events import UserActivityEvent
from yoyopod.integrations.screen.commands import (
    SetBrightnessCommand,
    SetIdleTimeoutCommand,
    SleepScreenCommand,
    WakeScreenCommand,
)
from yoyopod.integrations.screen.handlers import (
    handle_user_activity,
    resolve_idle_timeout_seconds,
    resolve_initial_brightness_percent,
    seed_screen_state,
    set_brightness,
    set_idle_timeout,
    sleep_screen,
    wake_screen,
)


@dataclass(slots=True)
class ScreenIntegration:
    """Runtime handles owned by the scaffold screen integration."""

    brightness_percent: int
    idle_timeout_seconds: float
    last_user_activity_at: float | None = None
    last_user_activity_action: str | None = None
    last_wake_reason: str = ""
    last_sleep_reason: str = ""


def setup(
    app: Any,
    *,
    initial_awake: bool = True,
    brightness_percent: int | None = None,
    idle_timeout_seconds: float | None = None,
    monotonic: Any = None,
) -> ScreenIntegration:
    """Register scaffold screen services and seed initial screen state."""

    actual_monotonic = monotonic or time.monotonic
    actual_brightness = (
        resolve_initial_brightness_percent(app.config)
        if brightness_percent is None
        else max(0, min(100, int(brightness_percent)))
    )
    actual_timeout = (
        resolve_idle_timeout_seconds(app.config)
        if idle_timeout_seconds is None
        else max(0.0, float(idle_timeout_seconds))
    )

    integration = ScreenIntegration(
        brightness_percent=actual_brightness,
        idle_timeout_seconds=actual_timeout,
    )
    app.integrations["screen"] = integration
    seed_screen_state(
        app,
        awake=initial_awake,
        brightness_percent=actual_brightness,
    )
    integration.last_user_activity_at = actual_monotonic()

    app.bus.subscribe(
        UserActivityEvent,
        lambda event: handle_user_activity(
            app,
            integration,
            event,
            now=actual_monotonic(),
        ),
    )
    app.services.register(
        "screen",
        "wake",
        lambda data: wake_screen(app, integration, data),
    )
    app.services.register(
        "screen",
        "sleep",
        lambda data: sleep_screen(app, integration, data),
    )
    app.services.register(
        "screen",
        "set_brightness",
        lambda data: set_brightness(app, integration, data),
    )
    app.services.register(
        "screen",
        "set_idle_timeout",
        lambda data: set_idle_timeout(integration, data),
    )
    return integration


def teardown(app: Any) -> None:
    """Drop the scaffold screen integration handle."""

    app.integrations.pop("screen", None)


__all__ = [
    "ScreenIntegration",
    "SetBrightnessCommand",
    "SetIdleTimeoutCommand",
    "SleepScreenCommand",
    "WakeScreenCommand",
    "setup",
    "teardown",
]
