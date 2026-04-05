"""Talk hub screen for YoyoPod VoIP functionality."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal, Optional

from loguru import logger

from yoyopy.ui.display import Display
from yoyopy.ui.screens.base import Screen
from yoyopy.ui.screens.theme import (
    INK,
    MUTED,
    SURFACE,
    TALK,
    draw_empty_state,
    draw_list_item,
    render_footer,
    render_header,
    rounded_panel,
)
from yoyopy.ui.screens.voip.lvgl import LvglCallView

if TYPE_CHECKING:
    from yoyopy.app_context import AppContext
    from yoyopy.config import ConfigManager, Contact
    from yoyopy.ui.screens import ScreenView
    from yoyopy.voip import CallHistoryStore, VoIPManager


@dataclass(slots=True)
class QuickCallTarget:
    """Represents one selectable quick action on the Talk screen."""

    kind: Literal["contact", "browse_contacts", "history", "voice_notes"]
    title: str
    subtitle: str
    sip_address: str = ""


class CallScreen(Screen):
    """Talk screen showing readiness, favorites, recents, and voice notes."""

    _MAX_QUICK_CONTACTS = 4

    def __init__(
        self,
        display: Display,
        context: Optional["AppContext"] = None,
        voip_manager: Optional["VoIPManager"] = None,
        config_manager: Optional["ConfigManager"] = None,
        call_history_store: Optional["CallHistoryStore"] = None,
    ) -> None:
        super().__init__(display, context, "Call")
        self.voip_manager = voip_manager
        self.config_manager = config_manager
        self.call_history_store = call_history_store
        self.quick_targets: list[QuickCallTarget] = []
        self.selected_index = 0
        self.scroll_offset = 0
        self.max_visible_items = 3 if display.is_portrait() else 4
        self._lvgl_view: "ScreenView | None" = None

    def enter(self) -> None:
        """Refresh quick-call shortcuts whenever the screen becomes active."""
        super().enter()
        self._load_quick_targets()
        self._ensure_lvgl_view()

    def exit(self) -> None:
        """Tear down any active LVGL view when leaving Talk."""
        if self._lvgl_view is not None:
            self._lvgl_view.destroy()
            self._lvgl_view = None
        super().exit()

    def _ensure_lvgl_view(self) -> "ScreenView | None":
        """Create an LVGL view when the Whisplay renderer is active."""
        if self._lvgl_view is not None:
            return self._lvgl_view

        if getattr(self.display, "backend_kind", "pil") != "lvgl":
            return None

        ui_backend = self.display.get_ui_backend() if hasattr(self.display, "get_ui_backend") else None
        if ui_backend is None or not getattr(ui_backend, "initialized", False):
            return None

        self._lvgl_view = LvglCallView(self, ui_backend)
        self._lvgl_view.build()
        return self._lvgl_view

    def _load_quick_targets(self) -> None:
        """Load quick-call contacts plus product actions like recents and voice notes."""
        contacts: list["Contact"] = []
        if self.config_manager is not None:
            contacts = sorted(
                self.config_manager.get_contacts(),
                key=lambda contact: (not contact.favorite, contact.name.lower()),
            )

        favorites = [contact for contact in contacts if contact.favorite]
        quick_contacts = favorites or contacts
        visible_contacts = quick_contacts[: self._MAX_QUICK_CONTACTS]

        targets = [
            QuickCallTarget(
                kind="contact",
                title=contact.name,
                subtitle=contact.sip_address,
                sip_address=contact.sip_address,
            )
            for contact in visible_contacts
        ]

        if self.call_history_store is not None:
            targets.append(
                QuickCallTarget(
                    kind="history",
                    title="Recents",
                    subtitle=self._history_subtitle(),
                )
            )

        if contacts:
            targets.append(
                QuickCallTarget(
                    kind="voice_notes",
                    title="Voice Note",
                    subtitle="Pick who gets your note",
                )
            )

        has_more_contacts = bool(contacts) and (
            len(contacts) > len(visible_contacts)
            or (bool(favorites) and len(favorites) < len(contacts))
        )
        if has_more_contacts or contacts:
            targets.append(
                QuickCallTarget(
                    kind="browse_contacts",
                    title="All Contacts",
                    subtitle="See everyone",
                )
            )

        self.quick_targets = targets
        if not self.quick_targets:
            self.selected_index = 0
            self.scroll_offset = 0
            return

        self.selected_index = min(self.selected_index, len(self.quick_targets) - 1)
        self._ensure_selection_visible()

    def _history_subtitle(self) -> str:
        """Return the compact subtitle for the Recents quick target."""
        if self.call_history_store is None:
            return "Recent calls"

        missed_count = self.call_history_store.missed_count()
        if missed_count > 0:
            label = "call" if missed_count == 1 else "calls"
            return f"{missed_count} missed {label}"

        recent_entries = self.call_history_store.list_recent(limit=1)
        if recent_entries:
            return f"Last: {recent_entries[0].title}"
        return "Recent calls"

    def render(self) -> None:
        """Render the Talk hub with status and quick-call cards."""
        lvgl_view = self._ensure_lvgl_view()
        if lvgl_view is not None:
            lvgl_view.sync()
            return

        status = self._get_status_snapshot()
        status_text, _status_color, _detail_text = self._status_lines(status)
        call_state_text, caller_text = self._call_context_lines(status)

        content_top = render_header(
            self.display,
            self.context,
            mode="talk",
            title="Talk",
            page_text=None,
            show_time=False,
            show_mode_chip=False,
        )

        panel_top = content_top + 6
        panel_bottom = self.display.HEIGHT - 28
        rounded_panel(
            self.display,
            12,
            panel_top,
            self.display.WIDTH - 12,
            panel_bottom,
            fill=SURFACE,
            outline=None,
            radius=24,
        )

        header_text = call_state_text or "Calls"
        self.display.text(header_text, 22, panel_top + 10, color=TALK.accent, font_size=13)

        if call_state_text and caller_text:
            self.display.text(caller_text, 22, panel_top + 28, color=INK, font_size=10)
        else:
            self.display.text(status_text, 22, panel_top + 28, color=MUTED, font_size=10)

        if not self.quick_targets:
            draw_empty_state(
                self.display,
                mode="talk",
                title="No contacts",
                subtitle="Add people to make Talk feel instant.",
                icon="talk",
                top=content_top + 12,
            )
            render_footer(self.display, "Hold back", mode="talk")
            self.display.update()
            return

        visible_items = max(1, min(self.max_visible_items, len(self.quick_targets)))
        self._ensure_selection_visible(visible_items)

        item_height = 50
        list_top = panel_top + 48
        for row in range(visible_items):
            target_index = self.scroll_offset + row
            if target_index >= len(self.quick_targets):
                break

            target = self.quick_targets[target_index]
            y1 = list_top + (row * item_height)
            y2 = y1 + 42
            draw_list_item(
                self.display,
                x1=20,
                y1=y1,
                x2=self.display.WIDTH - 20,
                y2=y2,
                title=target.title,
                subtitle="",
                mode="talk",
                selected=target_index == self.selected_index,
                badge=None,
            )

        render_footer(self.display, self._instruction_text(), mode="talk")
        self.display.update()

    def get_page_text(self) -> str | None:
        """Talk now intentionally omits page counts to keep the chrome calmer."""
        return None

    def get_visible_window(self) -> tuple[list[str], list[str], int]:
        """Return the visible quick-target titles, badges, and selected row index."""
        if not self.quick_targets:
            return [], [], 0

        visible_items = max(1, min(self.max_visible_items, len(self.quick_targets)))
        self._ensure_selection_visible(visible_items)

        visible_titles: list[str] = []
        visible_badges: list[str] = []
        selected_visible_index = 0

        for row in range(visible_items):
            target_index = self.scroll_offset + row
            if target_index >= len(self.quick_targets):
                break

            target = self.quick_targets[target_index]
            visible_titles.append(target.title)
            visible_badges.append("")
            if target_index == self.selected_index:
                selected_visible_index = row

        return visible_titles, visible_badges, selected_visible_index

    def _get_status_snapshot(self) -> dict:
        """Return a stable VoIP status dict for rendering and actions."""
        if self.voip_manager is None:
            return {}
        return self.voip_manager.get_status()

    def _status_lines(self, status: dict) -> tuple[str, tuple[int, int, int], str]:
        """Return the primary Talk availability copy."""
        if not self.voip_manager:
            return ("Talk offline", self.display.COLOR_RED, "Manager not initialized")

        running = status.get("running", False)
        registered = status.get("registered", False)
        registration_state = status.get("registration_state", "none")

        if not status.get("sip_identity"):
            return ("Not set up", self.display.COLOR_GRAY, "")
        if not running:
            return ("Recovering", self.display.COLOR_YELLOW, "")
        if registered:
            return ("Calls ready", self.display.COLOR_GREEN, "")
        if registration_state == "progress":
            return ("Connecting", self.display.COLOR_YELLOW, "")
        if registration_state == "failed":
            return ("Offline", self.display.COLOR_RED, "")
        return ("Offline", self.display.COLOR_GRAY, "")

    def _call_context_lines(self, status: dict) -> tuple[str, str]:
        """Return the current call-state summary if the backend is mid-call."""
        if not self.voip_manager:
            return ("", "")

        call_state = status.get("call_state", "idle")
        if call_state in {"idle", "released"}:
            return ("", "")

        state_labels = {
            "incoming": "Incoming",
            "outgoing": "Calling",
            "outgoing_progress": "Calling",
            "outgoing_ringing": "Ringing",
            "outgoing_early_media": "Connecting",
            "connected": "In call",
            "streams_running": "In call",
        }
        caller_info = self.voip_manager.get_caller_info()
        caller_text = caller_info.get("display_name") or caller_info.get("address") or ""
        return (state_labels.get(call_state, call_state.replace("_", " ").title()), caller_text)

    def _selected_target(self) -> Optional[QuickCallTarget]:
        """Return the currently selected quick target, if any."""
        if not self.quick_targets:
            return None
        return self.quick_targets[self.selected_index]

    def _is_ready_to_call(self) -> bool:
        """Return whether the VoIP backend is ready to place a call."""
        status = self._get_status_snapshot()
        return bool(status.get("running")) and bool(status.get("registered"))

    def _instruction_text(self) -> str:
        """Return footer hints for the current selection and state."""
        if not self.quick_targets:
            return "Hold back" if self.is_one_button_mode() else "B back"

        selected_target = self._selected_target()
        if selected_target is None:
            return "Hold back" if self.is_one_button_mode() else "B back"

        if self.is_one_button_mode():
            if selected_target.kind == "contact" and self._is_ready_to_call():
                return "Tap next / Double call"
            return "Tap next / Double open"

        if selected_target.kind == "contact" and self._is_ready_to_call():
            return "A call | B back | X/Y move"
        return "A open | B back | X/Y move"

    def _ensure_selection_visible(self, visible_items: Optional[int] = None) -> None:
        """Adjust scroll offset to keep the selected target on screen."""
        if not self.quick_targets:
            self.scroll_offset = 0
            return

        visible = visible_items or self.max_visible_items
        if self.selected_index < self.scroll_offset:
            self.scroll_offset = self.selected_index
        elif self.selected_index >= self.scroll_offset + visible:
            self.scroll_offset = self.selected_index - visible + 1

    def _browse_contacts(self) -> None:
        """Open the full contact list."""
        logger.info("Opening full contact list from Talk")
        self.request_route("browse_contacts")

    def _open_history(self) -> None:
        """Open the recent-calls history screen."""
        logger.info("Opening Talk recents")
        self.request_route("browse_history")

    def _open_voice_notes(self) -> None:
        """Open the voice-note recipient picker."""
        logger.info("Opening Talk voice-note recipients")
        self.request_route("voice_notes")

    def _call_selected_target(self) -> None:
        """Place a call to the currently selected contact or open the chosen action."""
        selected_target = self._selected_target()
        if selected_target is None:
            logger.debug("No Talk target selected")
            return

        if selected_target.kind == "browse_contacts":
            self._browse_contacts()
            return

        if selected_target.kind == "history":
            self._open_history()
            return

        if selected_target.kind == "voice_notes":
            self._open_voice_notes()
            return

        if not self._is_ready_to_call():
            self._browse_contacts()
            return

        if self.voip_manager is None:
            logger.error("Cannot place call: VoIP manager unavailable")
            return

        logger.info(f"Calling quick contact: {selected_target.title} ({selected_target.sip_address})")
        if self.voip_manager.make_call(selected_target.sip_address, contact_name=selected_target.title):
            self.request_route("call_started")
            return

        logger.error(f"Failed to initiate quick call to {selected_target.title}")

    def on_select(self, data=None) -> None:
        """Call the selected person or open the selected Talk action."""
        self._call_selected_target()

    def on_back(self, data=None) -> None:
        """Return to the previous screen."""
        self.request_route("back")

    def on_up(self, data=None) -> None:
        """Move selection upward."""
        if self.selected_index > 0:
            self.selected_index -= 1
            self._ensure_selection_visible()

    def on_down(self, data=None) -> None:
        """Move selection downward."""
        if self.selected_index < len(self.quick_targets) - 1:
            self.selected_index += 1
            self._ensure_selection_visible()

    def on_advance(self, data=None) -> None:
        """Move through quick-call targets with wraparound."""
        if not self.quick_targets:
            return
        self.selected_index = (self.selected_index + 1) % len(self.quick_targets)
        self._ensure_selection_visible()
