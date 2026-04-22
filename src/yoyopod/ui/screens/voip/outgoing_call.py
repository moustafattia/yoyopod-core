"""Outgoing call screen for the Talk flow."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

from loguru import logger

from yoyopod.ui.display import Display
from yoyopod.ui.screens.base import LvglScreen
from yoyopod.ui.screens.voip.call_actions import CallActions
from yoyopod.ui.screens.voip.outgoing_call_pil_view import render_outgoing_call_pil
from yoyopod.ui.screens.voip.lvgl import LvglOutgoingCallView

if TYPE_CHECKING:
    from yoyopod.core import AppContext


class OutgoingCallScreen(LvglScreen):
    """Outgoing call surface while dialing or ringing."""

    def __init__(
        self,
        display: Display,
        context: Optional["AppContext"] = None,
        *,
        actions: CallActions | None = None,
        callee_address: str = "",
        callee_name: str = "Unknown",
    ) -> None:
        super().__init__(display, context, "OutgoingCall")
        self._actions = actions or CallActions()
        self.callee_address = callee_address
        self.callee_name = callee_name
        self.ring_animation_frame = 0

    def enter(self) -> None:
        """Create the LVGL view when the screen becomes active."""
        super().enter()
        self.ring_animation_frame = 0
        self._ensure_lvgl_view()

    def exit(self) -> None:
        """Leave the retained LVGL outgoing-call view alive across transitions."""
        super().exit()

    def _create_lvgl_view(self, ui_backend: object) -> LvglOutgoingCallView:
        """Build the retained LVGL view for this screen."""

        return LvglOutgoingCallView(self, ui_backend)

    def render(self) -> None:
        """Render the outgoing-call screen."""
        if self._sync_lvgl_view():
            return
        render_outgoing_call_pil(self)

    def _cancel_call(self) -> None:
        """Cancel the outgoing call."""
        logger.info("Canceling outgoing call")
        if self._actions.hangup_call is not None:
            self._actions.hangup_call()

    def on_back(self, data=None) -> None:
        """Cancel the outgoing call."""
        self._cancel_call()

    def on_advance(self, data=None) -> None:
        """Outgoing-call single tap is intentionally a no-op."""
        return

    def on_select(self, data=None) -> None:
        """Outgoing-call double tap is intentionally a no-op."""
        return

    def on_call_hangup(self, data=None) -> None:
        """Cancel the outgoing call from a dedicated action."""
        self._cancel_call()
