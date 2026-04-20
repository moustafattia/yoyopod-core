"""Compatibility exports for callers still importing calling.backend."""

from __future__ import annotations

from importlib import import_module
from typing import TYPE_CHECKING, Any

from yoyopod.communication.calling.backend_protocol import VoIPBackend, VoIPIterateMetrics

if TYPE_CHECKING:
    from yoyopod.communication.calling.mock_backend import MockVoIPBackend
    from yoyopod.communication.integrations.liblinphone import LiblinphoneBackend


_LAZY_EXPORTS = {
    "LiblinphoneBackend": "yoyopod.communication.integrations.liblinphone",
    "MockVoIPBackend": "yoyopod.communication.calling.mock_backend",
}


def __getattr__(name: str) -> Any:
    module_name = _LAZY_EXPORTS.get(name)
    if module_name is None:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
    return getattr(import_module(module_name), name)


__all__ = [
    "LiblinphoneBackend",
    "MockVoIPBackend",
    "VoIPBackend",
    "VoIPIterateMetrics",
]
