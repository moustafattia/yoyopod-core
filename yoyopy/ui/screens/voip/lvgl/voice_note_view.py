"""LVGL-backed view for the voice-note flow."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from yoyopy.ui.lvgl_binding import LvglDisplayBackend
from yoyopy.ui.screens.theme import TALK

if TYPE_CHECKING:
    from yoyopy.app_context import AppContext
    from yoyopy.ui.screens.voip.voice_note import VoiceNoteScreen


@dataclass(slots=True)
class LvglVoiceNoteView:
    """Own the LVGL object lifecycle for VoiceNoteScreen."""

    screen: "VoiceNoteScreen"
    backend: LvglDisplayBackend
    _built: bool = False

    def build(self) -> None:
        if self._built or self.backend.binding is None:
            return
        self.backend.binding.talk_actions_build()
        self._built = True

    def sync(self) -> None:
        if not self._built or self.backend.binding is None:
            return

        context = self.screen.context
        title_text, _subtitle_text, footer_text, _icon_key = self.screen.current_view_model()
        items, _badges, selected_index = self.screen.current_actions_for_view()
        action_icon_keys = self.screen.current_action_icons()
        if items:
            self.backend.binding.talk_actions_sync(
                contact_name=self.screen.recipient_name(),
                title_text=items[selected_index] if items else None,
                status_text=None,
                status_kind=0,
                footer=footer_text,
                icon_keys=action_icon_keys,
                color_kinds=self.screen.current_action_color_kinds(),
                action_count=len(items),
                selected_index=selected_index,
                layout_kind=0,
                button_size_kind=0,
                voip_state=self._voip_state(context),
                battery_percent=self._battery_percent(context),
                charging=bool(getattr(context, "battery_charging", False)) if context is not None else False,
                power_available=bool(getattr(context, "power_available", True)) if context is not None else True,
                accent=TALK.accent,
            )
            return

        status_text, _status_color = self.screen.current_primary_status()
        self.backend.binding.talk_actions_sync(
            contact_name=self.screen.recipient_name(),
            title_text=title_text,
            status_text=status_text,
            status_kind=self.screen.current_primary_status_kind(),
            footer=footer_text,
            icon_keys=[self.screen.current_primary_icon()],
            color_kinds=[self.screen.current_primary_color_kind()],
            action_count=1,
            selected_index=0,
            layout_kind=1,
            button_size_kind=2,
            voip_state=self._voip_state(context),
            battery_percent=self._battery_percent(context),
            charging=bool(getattr(context, "battery_charging", False)) if context is not None else False,
            power_available=bool(getattr(context, "power_available", True)) if context is not None else True,
            accent=TALK.accent,
        )

    def destroy(self) -> None:
        if not self._built or self.backend.binding is None:
            return
        self.backend.binding.talk_actions_destroy()
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
