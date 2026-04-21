"""Compatibility shim for the relocated call manager facade."""

from yoyopod.integrations.call.manager import VoIPIterateSnapshot, VoIPManager

__all__ = ["VoIPIterateSnapshot", "VoIPManager"]
