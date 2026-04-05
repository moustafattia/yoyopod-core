"""Voice-note foundation screen for the Talk flow."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

from yoyopy.ui.display import Display
from yoyopy.ui.screens.base import Screen
from yoyopy.ui.screens.theme import TALK, INK, draw_icon, render_footer, render_header, rounded_panel, wrap_text
from yoyopy.ui.screens.voip.lvgl.voice_note_view import LvglVoiceNoteView

if TYPE_CHECKING:
    from yoyopy.app_context import AppContext
    from yoyopy.ui.screens import ScreenView


class VoiceNoteScreen(Screen):
    """Small shell for future Talk voice-note recording."""

    def __init__(self, display: Display, context: Optional["AppContext"] = None) -> None:
        super().__init__(display, context, "VoiceNote")
        self._state = "ready"
        self._lvgl_view: "ScreenView | None" = None

    def enter(self) -> None:
        """Reset the voice-note shell when opened."""
        super().enter()
        self._state = "ready"
        self._ensure_lvgl_view()

    def exit(self) -> None:
        """Tear down any active LVGL view when leaving voice notes."""
        if self._lvgl_view is not None:
            self._lvgl_view.destroy()
            self._lvgl_view = None
        super().exit()

    def _ensure_lvgl_view(self) -> "ScreenView | None":
        if self._lvgl_view is not None:
            return self._lvgl_view

        if getattr(self.display, "backend_kind", "pil") != "lvgl":
            return None

        ui_backend = self.display.get_ui_backend() if hasattr(self.display, "get_ui_backend") else None
        if ui_backend is None or not getattr(ui_backend, "initialized", False):
            return None

        self._lvgl_view = LvglVoiceNoteView(self, ui_backend)
        self._lvgl_view.build()
        return self._lvgl_view

    def recipient_name(self) -> str:
        """Return the current selected recipient name."""
        if self.context is None:
            return "Friend"
        return self.context.voice_note_recipient_name or "Friend"

    def current_view_model(self) -> tuple[str, str, str, str]:
        """Return title, subtitle, footer, and icon for the current voice-note state."""
        recipient = self.recipient_name()
        if self._state == "recording":
            return (
                "Recording",
                f"Saving a note for {recipient}.",
                "Double save / Hold back" if self.is_one_button_mode() else "A save | B back",
                "voice_note",
            )
        if self._state == "saved":
            return (
                "Saved",
                f"Your next voice note will go to {recipient}.",
                "Double again / Hold back" if self.is_one_button_mode() else "A again | B back",
                "voice_note",
            )
        return (
            "Voice note",
            f"Ready for {recipient}.",
            "Double record / Hold back" if self.is_one_button_mode() else "A record | B back",
            "voice_note",
        )

    def render(self) -> None:
        """Render the current voice-note shell state."""
        lvgl_view = self._ensure_lvgl_view()
        if lvgl_view is not None:
            lvgl_view.sync()
            return

        title_text, subtitle_text, footer_text, icon_key = self.current_view_model()
        content_top = render_header(
            self.display,
            self.context,
            mode="talk",
            title="Voice Note",
            show_time=False,
            show_mode_chip=False,
        )

        panel_top = content_top + 8
        panel_bottom = self.display.HEIGHT - 28
        rounded_panel(
            self.display,
            16,
            panel_top,
            self.display.WIDTH - 16,
            panel_bottom,
            fill=(31, 34, 40),
            outline=TALK.accent_dim,
            radius=24,
        )

        draw_icon(self.display, icon_key, (self.display.WIDTH // 2) - 24, panel_top + 18, 48, TALK.accent)

        headline_width, headline_height = self.display.get_text_size(title_text, 18)
        self.display.text(
            title_text,
            (self.display.WIDTH - headline_width) // 2,
            panel_top + 78,
            color=TALK.accent,
            font_size=18,
        )

        copy_lines = wrap_text(
            self.display,
            subtitle_text,
            self.display.WIDTH - 52,
            12,
            max_lines=2,
        )
        line_y = panel_top + 108
        for line in copy_lines:
            line_width, _ = self.display.get_text_size(line, 12)
            self.display.text(line, (self.display.WIDTH - line_width) // 2, line_y, color=INK, font_size=12)
            line_y += 15

        render_footer(self.display, footer_text, mode="talk")
        self.display.update()

    def on_select(self, data=None) -> None:
        """Cycle the voice-note shell through record/save/ready states."""
        if self._state == "ready":
            self._state = "recording"
            return
        if self._state == "recording":
            self._state = "saved"
            return
        self._state = "recording"

    def on_back(self, data=None) -> None:
        """Return to the previous screen."""
        self.request_route("back")
