"""LVGL-backed view for the Setup screen."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from yoyopy.ui.lvgl_binding import LvglDisplayBackend
from yoyopy.ui.screens.theme import SETUP

if TYPE_CHECKING:
    from yoyopy.app_context import AppContext
    from yoyopy.ui.screens.system.power import PowerScreen


@dataclass(slots=True)
class LvglPowerView:
    """Own the LVGL object lifecycle for PowerScreen."""

    screen: "PowerScreen"
    backend: LvglDisplayBackend
    _built: bool = False

    def build(self) -> None:
        if self._built or self.backend.binding is None:
            return
        self.backend.binding.power_build()
        self._built = True

    def sync(self) -> None:
        if not self._built or self.backend.binding is None:
            return

        snapshot = self.screen._get_snapshot()
        status = self.screen._get_status()
        pages = self.screen.build_pages(snapshot=snapshot, status=status)
        if not pages:
            return

        self.screen.page_index %= len(pages)
        active_page = pages[self.screen.page_index]

        # Page picker (Power/Time/Care/Voice) without showing page contents.
        if not getattr(self.screen, "in_detail", True):
            items: list[str] = []
            for index, page in enumerate(pages[:4]):
                prefix = "> " if index == self.screen.page_index else ""
                items.append(f"{prefix}{page.title}")

            context = self.screen.context
            self.backend.binding.power_sync(
                title_text="Setup",
                page_text=None,
                icon_key=self.screen._page_icon_key(active_page.title),
                footer=self.screen._instruction_text(active_page),
                items=items,
                current_page_index=self.screen.page_index,
                total_pages=len(pages),
                voip_state=self._voip_state(context),
                battery_percent=self._battery_percent(context),
                charging=bool(getattr(context, "battery_charging", False)) if context is not None else False,
                power_available=bool(getattr(context, "power_available", True)) if context is not None else True,
                accent=SETUP.accent,
            )
            return

        rows = list(active_page.rows)
        selected_row = max(0, min(self.screen.selected_row, max(0, len(rows) - 1))) if rows else 0

        # LVGL power scene only supports 4 visible rows. For interactive pages,
        # scroll so the selected row remains visible.
        start_row = 0
        if active_page.interactive and rows:
            max_visible = min(4, len(rows))
            start_row = max(0, min(selected_row - (max_visible // 2), len(rows) - max_visible))

        visible_rows = rows[start_row : start_row + 4]
        items: list[str] = []
        for offset, (label, value) in enumerate(visible_rows):
            absolute_index = start_row + offset
            prefix = "> " if active_page.interactive and absolute_index == selected_row else ""
            items.append(f"{prefix}{label}: {value}")
        context = self.screen.context

        self.backend.binding.power_sync(
            title_text=active_page.title,
            page_text=None,
            icon_key=self.screen._page_icon_key(active_page.title),
            footer=self.screen._instruction_text(active_page),
            items=items,
            current_page_index=self.screen.page_index,
            total_pages=len(pages),
            voip_state=self._voip_state(context),
            battery_percent=self._battery_percent(context),
            charging=bool(getattr(context, "battery_charging", False)) if context is not None else False,
            power_available=bool(getattr(context, "power_available", True)) if context is not None else True,
            accent=SETUP.accent,
        )

    def destroy(self) -> None:
        if not self._built or self.backend.binding is None:
            return
        self.backend.binding.power_destroy()
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
