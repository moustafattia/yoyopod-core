"""LVGL-backed view for the outgoing-call screen."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from yoyopy.ui.lvgl_binding import LvglDisplayBackend
from yoyopy.ui.screens.theme import TALK

if TYPE_CHECKING:
    from yoyopy.app_context import AppContext
    from yoyopy.ui.screens.voip.outgoing_call import OutgoingCallScreen


@dataclass(slots=True)
class LvglOutgoingCallView:
    """Own the LVGL object lifecycle for OutgoingCallScreen."""

    screen: "OutgoingCallScreen"
    backend: LvglDisplayBackend
    _built: bool = False

    def build(self) -> None:
        if self._built or self.backend.binding is None:
            return
        self.backend.binding.outgoing_call_build()
        self._built = True

    def sync(self) -> None:
        if not self._built or self.backend.binding is None:
            return

        callee_name = self.screen.callee_name or "Unknown"
        callee_address = self.screen.callee_address or "Unknown"
        if self.screen.voip_manager:
            caller_info = self.screen.voip_manager.get_caller_info()
            callee_name = caller_info.get("display_name", callee_name) or "Unknown"
            callee_address = caller_info.get("address", callee_address) or "Unknown"

        footer = "Hold = Cancel" if self.screen.is_one_button_mode() else "B cancel"
        context = self.screen.context

        self.backend.binding.outgoing_call_sync(
            callee_name=callee_name,
            callee_address=callee_address,
            footer=footer,
            voip_state=self._voip_state(context),
            battery_percent=self._battery_percent(context),
            charging=bool(getattr(context, "battery_charging", False)) if context is not None else False,
            power_available=bool(getattr(context, "power_available", True)) if context is not None else True,
            accent=TALK.accent,
        )

    def destroy(self) -> None:
        if not self._built or self.backend.binding is None:
            return
        self.backend.binding.outgoing_call_destroy()
        self._built = False

    @staticmethod
    def _battery_percent(context: "AppContext | None") -> int:
        if context is None:
            return 100
        return max(0, min(100, int(getattr(context, "battery_percent", 100))))

    @staticmethod
    def _voip_state(context: "AppContext | None") -> int:
        if context is None or not getattr(context, "voip_configured", False):
            return 0
        return 1 if getattr(context, "voip_ready", False) else 2
