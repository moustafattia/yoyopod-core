"""Typed services exposed by the scaffold recovery integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class RequestRecoveryCommand:
    """Request a retry cycle for one recoverable integration domain."""

    domain: str
