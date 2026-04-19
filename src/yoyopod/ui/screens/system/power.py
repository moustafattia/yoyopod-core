"""Setup screen for power, runtime, and device care."""

from __future__ import annotations

from yoyopod.ui.screens.system.power_rows import PowerPage
from yoyopod.ui.screens.system.power_screen import (
    PowerScreen,
    PowerScreenLvglPayload,
)
from yoyopod.ui.screens.system.power_viewmodel import (
    PowerScreenActions,
    PowerScreenState,
    _VOICE_PAGE_SIGNATURE_FIELDS,
    _build_gps_rows_from_manager,
    _build_network_rows_from_manager,
    build_power_screen_actions,
    build_power_screen_state_provider,
)

__all__ = [
    "PowerPage",
    "PowerScreen",
    "PowerScreenActions",
    "PowerScreenState",
    "PowerScreenLvglPayload",
    "_VOICE_PAGE_SIGNATURE_FIELDS",
    "_build_gps_rows_from_manager",
    "_build_network_rows_from_manager",
    "build_power_screen_actions",
    "build_power_screen_state_provider",
]
