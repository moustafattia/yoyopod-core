"""Future-safe Ask mode shell."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

from yoyopy.ui.display import Display
from yoyopy.ui.screens.base import Screen
from yoyopy.ui.screens.navigation.lvgl import LvglAskView
from yoyopy.ui.screens.theme import (
    ASK,
    INK,
    MUTED,
    draw_icon,
    render_footer,
    render_header,
    rounded_panel,
    wrap_text,
)

if TYPE_CHECKING:
    from yoyopy.app_context import AppContext
    from yoyopy.ui.screens import ScreenView


class AskScreen(Screen):
    """Small staged shell for the future Ask experience."""

    _PROMPTS = [
        "Tell me a fun fact",
        "Help me calm down",
        "Tell me a short story",
        "What can you do?",
    ]
    _RESPONSES = [
        "Safe answers will land here soon.",
        "This mode will answer with kid-safe help.",
        "We are building Ask carefully, not quickly.",
    ]

    def __init__(self, display: Display, context: Optional["AppContext"] = None) -> None:
        super().__init__(display, context, "Ask")
        self._state = "idle"
        self._prompt_index = 0
        self._response_index = 0
        self._lvgl_view: "ScreenView | None" = None

    def enter(self) -> None:
        """Reset Ask to its calm idle shell when the screen becomes active."""
        super().enter()
        self._state = "idle"
        self._ensure_lvgl_view()

    def exit(self) -> None:
        """Tear down any active LVGL view when leaving Ask."""
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

        self._lvgl_view = LvglAskView(self, ui_backend)
        self._lvgl_view.build()
        return self._lvgl_view

    def current_view_model(self) -> tuple[str, str, str, str]:
        """Return title, subtitle, footer, and icon for the current Ask state."""

        prompt = self._PROMPTS[self._prompt_index]
        response = self._RESPONSES[self._response_index]

        if self._state == "listening":
            return (
                "Listening",
                "Say your question now.",
                "Double done / Hold back" if self.is_one_button_mode() else "A done | B back",
                "ask",
            )
        if self._state == "thinking":
            return (
                "Thinking",
                "Preparing a safe reply.",
                "Double finish / Hold back" if self.is_one_button_mode() else "A finish | B back",
                "ask",
            )
        if self._state == "response":
            return (
                "Safe reply",
                response,
                "Tap next / Double again / Hold back" if self.is_one_button_mode() else "A again | B back | X/Y cycle",
                "ask",
            )

        return (
            "Ask AI",
            prompt,
            "Tap idea / Double start / Hold back" if self.is_one_button_mode() else "A start | B back | X/Y idea",
            "ask",
        )

    def render(self) -> None:
        """Render the current Ask shell state."""
        lvgl_view = self._ensure_lvgl_view()
        if lvgl_view is not None:
            lvgl_view.sync()
            return

        title_text, subtitle_text, footer_text, icon_key = self.current_view_model()
        content_top = render_header(
            self.display,
            self.context,
            mode="ask",
            title="Ask",
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
            outline=ASK.accent_dim,
            radius=24,
        )

        draw_icon(self.display, icon_key, (self.display.WIDTH // 2) - 24, panel_top + 18, 48, ASK.accent)

        headline_width, headline_height = self.display.get_text_size(title_text, 18)
        self.display.text(
            title_text,
            (self.display.WIDTH - headline_width) // 2,
            panel_top + 78,
            color=ASK.accent,
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
            self.display.text(
                line,
                (self.display.WIDTH - line_width) // 2,
                line_y,
                color=INK if self._state != "response" else MUTED,
                font_size=12,
            )
            line_y += 15

        render_footer(self.display, footer_text, mode="ask")
        self.display.update()

    def on_advance(self, data=None) -> None:
        """Cycle prompt ideas or response cards in one-button mode."""
        if self._state == "response":
            self._response_index = (self._response_index + 1) % len(self._RESPONSES)
            return
        if self._state == "idle":
            self._prompt_index = (self._prompt_index + 1) % len(self._PROMPTS)

    def on_select(self, data=None) -> None:
        """Move through the staged Ask shell without requiring a real AI backend yet."""
        if self._state == "idle":
            self._state = "listening"
            return
        if self._state == "listening":
            self._state = "thinking"
            return
        if self._state == "thinking":
            self._state = "response"
            return
        self._state = "listening"

    def on_back(self, data=None) -> None:
        """Return to the previous screen."""
        self.request_route("back")
