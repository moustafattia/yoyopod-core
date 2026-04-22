"""Incoming call screen for the Talk flow."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional

from loguru import logger

from yoyopod.ui.display import Display
from yoyopod.ui.screens.base import LvglScreen
from yoyopod.ui.screens.voip.call_actions import CallActions
from yoyopod.ui.screens.voip.incoming_call_pil_view import render_incoming_call_pil
from yoyopod.ui.screens.voip.lvgl import LvglIncomingCallView

if TYPE_CHECKING:
    from yoyopod.core import AppContext


class IncomingCallScreen(LvglScreen):
    """Incoming call surface with answer and reject actions."""

    def __init__(
        self,
        display: Display,
        context: Optional["AppContext"] = None,
        *,
        actions: CallActions | None = None,
        caller_address: str = "",
        caller_name: str = "Unknown",
    ) -> None:
        super().__init__(display, context, "IncomingCall")
        self._actions = actions or CallActions()
        self.caller_address = caller_address
        self.caller_name = caller_name
        self.ring_animation_frame = 0

    def enter(self) -> None:
        """Create the LVGL view when the screen becomes active."""
        super().enter()
        self.ring_animation_frame = 0
        self._ensure_lvgl_view()

    def exit(self) -> None:
        """Leave the retained LVGL incoming-call view alive across transitions."""
        super().exit()

    def _create_lvgl_view(self, ui_backend: object) -> LvglIncomingCallView:
        """Build the retained LVGL view for this screen."""

        return LvglIncomingCallView(self, ui_backend)

    def render(self) -> None:
        """Render the incoming call screen."""
        if self._sync_lvgl_view():
            return
        render_incoming_call_pil(self)

    def _answer_call(self) -> None:
        """Answer the incoming call."""
        logger.info("Answering incoming call")
        if self._actions.answer_call is not None:
            self._actions.answer_call()

    def _reject_call(self) -> None:
        """Reject the incoming call."""
        logger.info("Rejecting incoming call")
        if self._actions.reject_call is not None:
            self._actions.reject_call()

    def on_select(self, data=None) -> None:
        """Answer the incoming call."""
        self._answer_call()

    def on_advance(self, data=None) -> None:
        """Incoming-call single tap is intentionally a no-op."""
        return

    def on_call_answer(self, data=None) -> None:
        """Answer from a dedicated call action."""
        self._answer_call()

    def on_back(self, data=None) -> None:
        """Reject the incoming call."""
        self._reject_call()

    def on_call_reject(self, data=None) -> None:
        """Reject from a dedicated call action."""
        self._reject_call()
