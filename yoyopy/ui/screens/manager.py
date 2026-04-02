"""
Screen management and navigation for YoyoPod.

Handles screen transitions and the navigation stack.
"""

from typing import Optional, Dict, Type, TYPE_CHECKING
from loguru import logger

from yoyopy.ui.display import Display
from yoyopy.ui.screens.base import Screen

# Import InputManager from new input HAL
if TYPE_CHECKING:
    from yoyopy.ui.input import InputManager, InputAction
else:
    try:
        from yoyopy.ui.input import InputManager, InputAction
    except ImportError:
        # Fallback for gradual migration
        InputManager = None
        InputAction = None


class ScreenManager:
    """
    Manages screen navigation and transitions.

    Maintains a stack of screens for back navigation and
    handles screen lifecycle (enter/exit).
    """

    def __init__(self, display: Display, input_manager: Optional['InputManager'] = None) -> None:
        """
        Initialize the screen manager.

        Args:
            display: Display controller instance
            input_manager: Input manager for handling user input (optional)
        """
        self.display = display
        self.input_manager = input_manager
        self.current_screen: Optional[Screen] = None
        self.screen_stack: list[Screen] = []
        self.screens: Dict[str, Screen] = {}

        logger.info("ScreenManager initialized")

    def register_screen(self, name: str, screen: Screen) -> None:
        """
        Register a screen with the manager.

        Args:
            name: Unique name for the screen
            screen: Screen instance
        """
        self.screens[name] = screen
        screen.set_screen_manager(self)
        logger.debug(f"Registered screen: {name}")

    def push_screen(self, screen_name: str) -> None:
        """
        Push a new screen onto the stack and display it.

        Args:
            screen_name: Name of the registered screen to display
        """
        if screen_name not in self.screens:
            logger.error(f"Screen not found: {screen_name}")
            return

        # Exit current screen and remove button handlers
        if self.current_screen:
            self._disconnect_buttons()
            self.current_screen.exit()
            self.screen_stack.append(self.current_screen)

        # Enter new screen and set up button handlers
        self.current_screen = self.screens[screen_name]
        self.current_screen.enter()
        self._connect_buttons()
        self.current_screen.render()

        logger.info(f"Pushed screen: {screen_name} (stack depth: {len(self.screen_stack)})")

    def pop_screen(self) -> bool:
        """
        Pop the current screen and return to the previous one.

        Returns:
            True if successful, False if stack is empty
        """
        if not self.screen_stack:
            logger.warning("Cannot pop: screen stack is empty")
            return False

        # Exit current screen and remove button handlers
        if self.current_screen:
            self._disconnect_buttons()
            self.current_screen.exit()

        # Return to previous screen and reconnect button handlers
        self.current_screen = self.screen_stack.pop()
        self.current_screen.enter()
        self._connect_buttons()
        self.current_screen.render()

        logger.info(f"Popped screen (stack depth: {len(self.screen_stack)})")
        return True

    def replace_screen(self, screen_name: str) -> None:
        """
        Replace the current screen without adding to stack.

        Args:
            screen_name: Name of the registered screen to display
        """
        if screen_name not in self.screens:
            logger.error(f"Screen not found: {screen_name}")
            return

        # Exit current screen and remove button handlers
        if self.current_screen:
            self._disconnect_buttons()
            self.current_screen.exit()

        # Enter new screen and connect button handlers (don't push old one to stack)
        self.current_screen = self.screens[screen_name]
        self.current_screen.enter()
        self._connect_buttons()
        self.current_screen.render()

        logger.info(f"Replaced screen with: {screen_name}")

    def clear_stack(self) -> None:
        """Clear the screen stack."""
        self.screen_stack.clear()
        logger.debug("Screen stack cleared")

    def get_current_screen(self) -> Optional[Screen]:
        """Get the currently displayed screen."""
        return self.current_screen

    def refresh_current_screen(self) -> None:
        """Re-render the current screen."""
        if self.current_screen:
            self.current_screen.render()

    def _connect_inputs(self) -> None:
        """Connect input action handlers for the current screen."""
        if not self.current_screen:
            return

        # Skip if no input manager
        if not self.input_manager:
            logger.debug("No input manager available - skipping input connection")
            return

        # Import InputAction if available
        if InputAction is None:
            logger.warning("InputAction not available - cannot connect inputs")
            return

        # Helper function to wrap action handlers with auto-refresh
        def wrap_with_refresh(handler):
            """Wrap action handler to automatically refresh display after execution."""
            def wrapper(data=None):
                previous_screen = self.current_screen
                handler(data)
                if self.current_screen is previous_screen:
                    self.refresh_current_screen()
            return wrapper

        # Register semantic action callbacks with input manager
        # These are hardware-independent actions that screens understand
        # Each callback is wrapped to auto-refresh the display after execution
        self.input_manager.on_action(InputAction.SELECT, wrap_with_refresh(self.current_screen.on_select))
        self.input_manager.on_action(InputAction.BACK, wrap_with_refresh(self.current_screen.on_back))
        self.input_manager.on_action(InputAction.UP, wrap_with_refresh(self.current_screen.on_up))
        self.input_manager.on_action(InputAction.DOWN, wrap_with_refresh(self.current_screen.on_down))
        self.input_manager.on_action(InputAction.LEFT, wrap_with_refresh(self.current_screen.on_left))
        self.input_manager.on_action(InputAction.RIGHT, wrap_with_refresh(self.current_screen.on_right))
        self.input_manager.on_action(InputAction.MENU, wrap_with_refresh(self.current_screen.on_menu))
        self.input_manager.on_action(InputAction.HOME, wrap_with_refresh(self.current_screen.on_home))

        # Playback actions
        self.input_manager.on_action(InputAction.PLAY_PAUSE, wrap_with_refresh(self.current_screen.on_play_pause))
        self.input_manager.on_action(InputAction.NEXT_TRACK, wrap_with_refresh(self.current_screen.on_next_track))
        self.input_manager.on_action(InputAction.PREV_TRACK, wrap_with_refresh(self.current_screen.on_prev_track))

        # VoIP actions
        self.input_manager.on_action(InputAction.CALL_ANSWER, wrap_with_refresh(self.current_screen.on_call_answer))
        self.input_manager.on_action(InputAction.CALL_REJECT, wrap_with_refresh(self.current_screen.on_call_reject))
        self.input_manager.on_action(InputAction.CALL_HANGUP, wrap_with_refresh(self.current_screen.on_call_hangup))

        # PTT actions
        self.input_manager.on_action(InputAction.PTT_PRESS, wrap_with_refresh(self.current_screen.on_ptt_press))
        self.input_manager.on_action(InputAction.PTT_RELEASE, wrap_with_refresh(self.current_screen.on_ptt_release))

        # Voice actions
        self.input_manager.on_action(InputAction.VOICE_COMMAND, wrap_with_refresh(self.current_screen.on_voice_command))

        logger.debug(f"Connected input actions for {self.current_screen.name}")

    def _disconnect_inputs(self) -> None:
        """Disconnect input action handlers for the current screen."""
        if not self.current_screen:
            return

        # Skip if no input manager
        if not self.input_manager:
            logger.debug("No input manager available - skipping input disconnection")
            return

        # Clear all action callbacks
        self.input_manager.clear_callbacks()

        logger.debug(f"Disconnected input actions for {self.current_screen.name}")

    # Legacy method names for backward compatibility
    def _connect_buttons(self) -> None:
        """Legacy method - redirects to _connect_inputs()."""
        self._connect_inputs()

    def _disconnect_buttons(self) -> None:
        """Legacy method - redirects to _disconnect_inputs()."""
        self._disconnect_inputs()
