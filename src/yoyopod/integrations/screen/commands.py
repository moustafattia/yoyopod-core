"""Typed commands for the scaffold screen integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class WakeScreenCommand:
    """Wake the screen immediately."""

    reason: str = ""


@dataclass(frozen=True, slots=True)
class SleepScreenCommand:
    """Put the screen to sleep immediately."""

    reason: str = ""


@dataclass(frozen=True, slots=True)
class SetBrightnessCommand:
    """Set the active brightness percentage."""

    percent: int


@dataclass(frozen=True, slots=True)
class SetIdleTimeoutCommand:
    """Set the inactivity timeout used by the screen integration."""

    timeout_seconds: float
