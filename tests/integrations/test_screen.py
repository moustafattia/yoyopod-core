"""Tests for the scaffold screen integration."""

from __future__ import annotations

from types import SimpleNamespace

from yoyopod.core import UserActivityEvent, build_test_app, drain_all
from yoyopod.integrations.screen import (
    SetBrightnessCommand,
    SetIdleTimeoutCommand,
    SleepScreenCommand,
    WakeScreenCommand,
    setup,
    teardown,
)


def test_screen_setup_seeds_state_from_config_defaults() -> None:
    app = build_test_app()
    app.config = SimpleNamespace(
        display=SimpleNamespace(brightness=65, backlight_timeout_seconds=45),
        ui=SimpleNamespace(screen_timeout_seconds=300),
    )

    integration = setup(app)

    assert integration is app.integrations["screen"]
    assert integration.brightness_percent == 65
    assert integration.idle_timeout_seconds == 45.0
    assert app.states.get_value("screen.awake") is True
    assert app.states.get_value("screen.brightness_percent") == 65


def test_screen_services_update_state_and_runtime_values() -> None:
    app = build_test_app()
    integration = setup(app, brightness_percent=20, idle_timeout_seconds=10.0)

    slept = app.services.call("screen", "sleep", SleepScreenCommand(reason="idle"))
    brightness = app.services.call("screen", "set_brightness", SetBrightnessCommand(percent=120))
    timeout = app.services.call(
        "screen",
        "set_idle_timeout",
        SetIdleTimeoutCommand(timeout_seconds=-5.0),
    )
    woke = app.services.call("screen", "wake", WakeScreenCommand(reason="button"))

    assert slept is False
    assert woke is True
    assert brightness == 100
    assert timeout == 0.0
    assert integration.last_sleep_reason == "idle"
    assert integration.last_wake_reason == "button"
    assert integration.brightness_percent == 100
    assert integration.idle_timeout_seconds == 0.0
    assert app.states.get_value("screen.awake") is True
    assert app.states.get_value("screen.brightness_percent") == 100


def test_screen_user_activity_wakes_sleeping_screen() -> None:
    app = build_test_app()
    integration = setup(app, initial_awake=False, brightness_percent=55)

    app.bus.publish(UserActivityEvent(action_name="dial"))
    drain_all(app)

    assert app.states.get_value("screen.awake") is True
    assert integration.last_user_activity_action == "dial"
    assert integration.last_user_activity_at is not None


def test_screen_services_reject_wrong_payload_types() -> None:
    app = build_test_app()
    setup(app)

    try:
        app.services.call("screen", "set_brightness", {"percent": 50})  # type: ignore[arg-type]
    except TypeError as exc:
        assert str(exc) == "screen.set_brightness expects SetBrightnessCommand"
    else:
        raise AssertionError("screen.set_brightness accepted an untyped payload")

    teardown(app)
    assert "screen" not in app.integrations
