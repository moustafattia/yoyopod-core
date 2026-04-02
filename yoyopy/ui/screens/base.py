"""
Screen management for YoyoPod UI.

Provides base Screen class and concrete screen implementations
for different application states.
"""

from abc import ABC, abstractmethod
from typing import Optional, Any, TYPE_CHECKING
from loguru import logger

from yoyopy.ui.display import Display

if TYPE_CHECKING:
    from yoyopy.ui.screens.manager import ScreenManager
    from yoyopy.app_context import AppContext


class Screen(ABC):
    """
    Base class for all UI screens.

    Screens are responsible for rendering their content to the display
    and handling any screen-specific logic.
    """

    def __init__(
        self,
        display: Display,
        context: Optional['AppContext'] = None,
        name: str = "Screen"
    ) -> None:
        """
        Initialize the screen.

        Args:
            display: Display controller instance
            context: Application context (optional)
            name: Screen name for logging
        """
        self.display = display
        self.context = context
        self.name = name
        self.screen_manager: Optional['ScreenManager'] = None
        logger.debug(f"Screen '{name}' initialized")

    def set_screen_manager(self, manager: 'ScreenManager') -> None:
        """Set the screen manager for navigation."""
        self.screen_manager = manager

    def set_context(self, context: 'AppContext') -> None:
        """Set the application context."""
        self.context = context

    @abstractmethod
    def render(self) -> None:
        """
        Render the screen content.

        This method should draw all screen elements to the display buffer.
        Call display.update() to show the rendered content.
        """
        pass

    def enter(self) -> None:
        """
        Called when screen becomes active.

        Override to perform screen initialization and set up button handlers.
        """
        logger.info(f"Entering screen: {self.name}")

    def exit(self) -> None:
        """
        Called when screen becomes inactive.

        Override to perform cleanup and remove button handlers.
        """
        logger.info(f"Exiting screen: {self.name}")

    # ===== SEMANTIC ACTION HANDLERS =====
    # These are hardware-independent input actions that screens can respond to.
    # Screens override these methods based on their functionality.

    def on_select(self, data: Optional[Any] = None) -> None:
        """Handle SELECT action (confirm/accept current item)."""
        # Backward compatibility: delegate to on_button_a if not overridden
        if hasattr(self, 'on_button_a') and self.on_select.__func__ is Screen.on_select:
            self.on_button_a()

    def on_back(self, data: Optional[Any] = None) -> None:
        """Handle BACK action (cancel/return to previous screen)."""
        # Backward compatibility: delegate to on_button_b if not overridden
        if hasattr(self, 'on_button_b') and self.on_back.__func__ is Screen.on_back:
            self.on_button_b()

    def on_up(self, data: Optional[Any] = None) -> None:
        """Handle UP action (navigate up in lists/menus)."""
        # Backward compatibility: delegate to on_button_x if not overridden
        if hasattr(self, 'on_button_x') and self.on_up.__func__ is Screen.on_up:
            self.on_button_x()

    def on_down(self, data: Optional[Any] = None) -> None:
        """Handle DOWN action (navigate down in lists/menus)."""
        # Backward compatibility: delegate to on_button_y if not overridden
        if hasattr(self, 'on_button_y') and self.on_down.__func__ is Screen.on_down:
            self.on_button_y()

    def on_left(self, data: Optional[Any] = None) -> None:
        """Handle LEFT action (navigate left/previous)."""
        pass

    def on_right(self, data: Optional[Any] = None) -> None:
        """Handle RIGHT action (navigate right/next)."""
        pass

    def on_menu(self, data: Optional[Any] = None) -> None:
        """Handle MENU action (open menu)."""
        pass

    def on_home(self, data: Optional[Any] = None) -> None:
        """Handle HOME action (return to home screen)."""
        pass

    # Playback actions
    def on_play_pause(self, data: Optional[Any] = None) -> None:
        """Handle PLAY_PAUSE action (toggle playback)."""
        pass

    def on_next_track(self, data: Optional[Any] = None) -> None:
        """Handle NEXT_TRACK action (skip to next track)."""
        pass

    def on_prev_track(self, data: Optional[Any] = None) -> None:
        """Handle PREV_TRACK action (go to previous track)."""
        pass

    # VoIP actions
    def on_call_answer(self, data: Optional[Any] = None) -> None:
        """Handle CALL_ANSWER action (answer incoming call)."""
        pass

    def on_call_reject(self, data: Optional[Any] = None) -> None:
        """Handle CALL_REJECT action (reject incoming call)."""
        pass

    def on_call_hangup(self, data: Optional[Any] = None) -> None:
        """Handle CALL_HANGUP action (end active call)."""
        pass

    # PTT (Push-to-Talk) actions
    def on_ptt_press(self, data: Optional[Any] = None) -> None:
        """Handle PTT_PRESS action (PTT button pressed)."""
        pass

    def on_ptt_release(self, data: Optional[Any] = None) -> None:
        """Handle PTT_RELEASE action (PTT button released)."""
        pass

    # Voice actions
    def on_voice_command(self, data: Optional[Any] = None) -> None:
        """
        Handle VOICE_COMMAND action (voice command received).

        Args:
            data: Dict containing voice command information:
                  {'command': str, 'confidence': float, ...}
        """
        pass

    # ===== LEGACY BUTTON HANDLERS (for backward compatibility) =====
    # These will be removed in a future version.
    # Use semantic action handlers (on_select, on_back, etc.) instead.

    def on_button_a(self) -> None:
        """
        DEPRECATED: Handle Button A press.

        Use on_select() instead for hardware-independent input.
        """
        pass

    def on_button_b(self) -> None:
        """
        DEPRECATED: Handle Button B press.

        Use on_back() instead for hardware-independent input.
        """
        pass

    def on_button_x(self) -> None:
        """
        DEPRECATED: Handle Button X press.

        Use on_up() instead for hardware-independent input.
        """
        pass

    def on_button_y(self) -> None:
        """
        DEPRECATED: Handle Button Y press.

        Use on_down() instead for hardware-independent input.
        """
        pass
