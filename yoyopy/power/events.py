"""Typed power-domain events."""

from __future__ import annotations

from dataclasses import dataclass

from yoyopy.power.models import PowerSnapshot


@dataclass(frozen=True, slots=True)
class PowerSnapshotUpdated:
    """Published when a new power snapshot is collected."""

    snapshot: PowerSnapshot


@dataclass(frozen=True, slots=True)
class PowerAvailabilityChanged:
    """Published when the power backend becomes available or unavailable."""

    available: bool
    reason: str = ""

