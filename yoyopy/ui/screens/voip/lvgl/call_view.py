"""LVGL-backed view for the Talk contact deck."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from yoyopy.ui.lvgl_binding import LvglDisplayBackend
from yoyopy.ui.screens.theme import TALK

if TYPE_CHECKING:
    from yoyopy.app_context import AppContext
    from yoyopy.ui.screens.voip.quick_call import CallScreen


@dataclass(slots=True)
class LvglCallView:
    """Own the LVGL object lifecycle for the people-first Talk screen."""

    screen: "CallScreen"
    backend: LvglDisplayBackend
    _built: bool = False

    def build(self) -> None:
        if self._built or self.backend.binding is None:
            return
        self.backend.binding.talk_build()
        self._built = True

    def sync(self) -> None:
        if not self._built or self.backend.binding is None:
            return

        model = self.screen.current_card_model()
        context = self.screen.context
        self.backend.binding.talk_sync(
            title_text=str(model.get("title") or "Talk"),
            icon_key=str(model.get("icon_key")) if model.get("icon_key") is not None else None,
            outlined=bool(model.get("outlined", False)),
            footer="Tap Next | 2x Open | Hold Back",
            accent=TALK.accent,
            selected_index=int(model.get("selected_index", 0)),
            total_cards=max(1, int(model.get("total_cards", 0))),
            voip_state=self._voip_state(context),
            battery_percent=self._battery_percent(context),
            charging=bool(getattr(context, "battery_charging", False)) if context is not None else False,
            power_available=bool(getattr(context, "power_available", True)) if context is not None else True,
        )

    def destroy(self) -> None:
        if not self._built or self.backend.binding is None:
            return
        self.backend.binding.talk_destroy()
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
