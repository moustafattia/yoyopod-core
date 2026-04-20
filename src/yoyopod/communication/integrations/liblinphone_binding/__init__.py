"""Compatibility alias for the relocated Liblinphone binding package."""

from yoyopod.communication.integrations.liblinphone_binding.binding import (
    LiblinphoneBinding,
    LiblinphoneBindingError,
    LiblinphoneNativeEvent,
)

__all__ = [
    "LiblinphoneBinding",
    "LiblinphoneBindingError",
    "LiblinphoneNativeEvent",
]
