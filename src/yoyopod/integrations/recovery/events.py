"""Events emitted by the scaffold recovery integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class RecoveryAttemptedEvent:
    """Published after one recovery attempt finishes."""

    domain: str
    success: bool
    reason: str = ""
