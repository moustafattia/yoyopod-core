"""Handlers for the scaffold screen integration."""

from __future__ import annotations

from typing import Any

from yoyopod.core.events import UserActivityEvent
from yoyopod.integrations.screen.commands import (
    SetBrightnessCommand,
    SetIdleTimeoutCommand,
    SleepScreenCommand,
    WakeScreenCommand,
)


def seed_screen_state(
    app: Any,
    *,
    awake: bool,
    brightness_percent: int,
) -> None:
    """Seed the scaffold screen state rows."""

    app.states.set("screen.awake", awake)
    app.states.set("screen.brightness_percent", brightness_percent)


def wake_screen(app: Any, integration: Any, command: WakeScreenCommand) -> bool:
    """Wake the screen and record the command reason."""

    if not isinstance(command, WakeScreenCommand):
        raise TypeError("screen.wake expects WakeScreenCommand")
    integration.last_wake_reason = command.reason
    app.states.set("screen.awake", True)
    return True


def sleep_screen(app: Any, integration: Any, command: SleepScreenCommand) -> bool:
    """Put the screen to sleep and record the command reason."""

    if not isinstance(command, SleepScreenCommand):
        raise TypeError("screen.sleep expects SleepScreenCommand")
    integration.last_sleep_reason = command.reason
    app.states.set("screen.awake", False)
    return False


def set_brightness(app: Any, integration: Any, command: SetBrightnessCommand) -> int:
    """Update the in-memory screen brightness and reflected scaffold state."""

    if not isinstance(command, SetBrightnessCommand):
        raise TypeError("screen.set_brightness expects SetBrightnessCommand")
    integration.brightness_percent = _normalize_brightness_percent(command.percent)
    app.states.set("screen.brightness_percent", integration.brightness_percent)
    return integration.brightness_percent


def set_idle_timeout(integration: Any, command: SetIdleTimeoutCommand) -> float:
    """Update the in-memory screen idle timeout."""

    if not isinstance(command, SetIdleTimeoutCommand):
        raise TypeError("screen.set_idle_timeout expects SetIdleTimeoutCommand")
    integration.idle_timeout_seconds = max(0.0, float(command.timeout_seconds))
    return integration.idle_timeout_seconds


def handle_user_activity(
    app: Any,
    integration: Any,
    event: UserActivityEvent,
    *,
    now: float,
) -> None:
    """Record user activity and wake the screen if it is asleep."""

    integration.last_user_activity_at = now
    integration.last_user_activity_action = event.action_name
    if not app.states.get_value("screen.awake", False):
        app.states.set("screen.awake", True)


def resolve_initial_brightness_percent(config: object | None, fallback: int = 80) -> int:
    """Resolve the initial brightness from scaffold config or a fallback."""

    display = getattr(config, "display", None)
    value = getattr(display, "brightness", fallback)
    return _normalize_brightness_percent(value)


def resolve_idle_timeout_seconds(config: object | None, fallback: float = 300.0) -> float:
    """Resolve the effective screen timeout using the live runtime precedence."""

    display = getattr(config, "display", None)
    ui = getattr(config, "ui", None)
    display_timeout = max(
        0.0,
        float(getattr(display, "backlight_timeout_seconds", 0.0) or 0.0),
    )
    if display_timeout > 0.0:
        return display_timeout
    return max(
        0.0,
        float(getattr(ui, "screen_timeout_seconds", fallback) or fallback),
    )


def _normalize_brightness_percent(value: object) -> int:
    return max(0, min(100, int(value)))
