#!/usr/bin/env python3
"""
State machine demo for YoyoPod.

This demo drives the `StateMachine` with semantic input actions and keeps
screen changes coordinated through the current screen stack.
"""

import sys
import time

from loguru import logger

from yoyopy.app_context import AppContext
from yoyopy.state_machine import AppState, StateMachine
from yoyopy.ui.display import Display
from yoyopy.ui.input import InputAction, get_input_manager
from yoyopy.ui.screens import HomeScreen, MenuScreen, NowPlayingScreen, ScreenManager

ACTION_MAP = {
    "a": InputAction.SELECT,
    "b": InputAction.BACK,
    "x": InputAction.UP,
    "y": InputAction.DOWN,
}

logger.remove()
logger.add(
    sys.stderr,
    format="<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | <level>{message}</level>",
    level="INFO",
)


def main() -> int:
    """Run the state machine demo."""
    logger.info("Starting YoyoPod state machine demo")
    logger.info("=" * 50)

    context = AppContext()
    demo_playlist = context.create_demo_playlist()
    context.set_playlist(demo_playlist)
    context.update_system_status(battery=85, signal=3, connected=True)
    logger.info(f"Demo playlist loaded: {len(demo_playlist.tracks)} tracks")

    display = Display(simulate="--simulate" in sys.argv)
    input_manager = get_input_manager(display.get_adapter(), simulate=display.simulate)
    if input_manager is None:
        logger.error("No input manager available for the detected display adapter")
        return 1

    state_machine = StateMachine(context)
    screen_manager = ScreenManager(display)

    home_screen = HomeScreen(display, context)
    menu_screen = MenuScreen(display, context)
    now_playing_screen = NowPlayingScreen(display, context)

    screen_manager.register_screen("home", home_screen)
    screen_manager.register_screen("menu", menu_screen)
    screen_manager.register_screen("now_playing", now_playing_screen)

    def on_enter_idle() -> None:
        screen_manager.replace_screen("home")

    def on_enter_menu() -> None:
        screen_manager.replace_screen("menu")

    def on_enter_playing() -> None:
        screen_manager.replace_screen("now_playing")

    def on_enter_paused() -> None:
        screen_manager.refresh_current_screen()

    state_machine.on_enter(AppState.IDLE, on_enter_idle)
    state_machine.on_enter(AppState.MENU, on_enter_menu)
    state_machine.on_enter(AppState.PLAYING, on_enter_playing)
    state_machine.on_enter(AppState.PAUSED, on_enter_paused)

    def handle_select(_data=None) -> None:
        current_state = state_machine.current_state

        if current_state == AppState.IDLE:
            state_machine.open_menu()
        elif current_state == AppState.MENU:
            selected = menu_screen.get_selected()
            if selected in ["Music", "Podcasts", "Audiobooks"]:
                state_machine.start_playback()
            else:
                logger.info(f"Selected menu item without state transition: {selected}")
        elif current_state in [AppState.PLAYING, AppState.PAUSED]:
            state_machine.toggle_playback()
            screen_manager.refresh_current_screen()

    def handle_back(_data=None) -> None:
        current_state = state_machine.current_state

        if current_state == AppState.MENU:
            state_machine.transition_to(AppState.IDLE, "back")
        elif current_state in [AppState.PLAYING, AppState.PAUSED]:
            state_machine.transition_to(AppState.MENU, "back")

    def handle_up(_data=None) -> None:
        current_state = state_machine.current_state

        if current_state == AppState.MENU:
            menu_screen.select_previous()
            screen_manager.refresh_current_screen()
        elif current_state in [AppState.PLAYING, AppState.PAUSED]:
            previous_track = context.previous_track()
            logger.info(f"Previous track: {previous_track.title if previous_track else 'None'}")
            screen_manager.refresh_current_screen()

    def handle_down(_data=None) -> None:
        current_state = state_machine.current_state

        if current_state == AppState.MENU:
            menu_screen.select_next()
            screen_manager.refresh_current_screen()
        elif current_state in [AppState.PLAYING, AppState.PAUSED]:
            next_track = context.next_track()
            logger.info(f"Next track: {next_track.title if next_track else 'None'}")
            screen_manager.refresh_current_screen()

    input_manager.on_action(InputAction.SELECT, handle_select)
    input_manager.on_action(InputAction.BACK, handle_back)
    input_manager.on_action(InputAction.UP, handle_up)
    input_manager.on_action(InputAction.DOWN, handle_down)

    screen_manager.replace_screen("home")
    input_manager.start()

    try:
        if display.simulate:
            logger.info("Simulation mode commands: a, b, x, y, state, quit")
            while True:
                try:
                    cmd = input("> ").strip().lower()
                except EOFError:
                    break

                if cmd in {"quit", "q", "exit"}:
                    break
                if cmd == "state":
                    logger.info(f"State: {state_machine.get_state_name()}")
                    logger.info(f"Track: {context.get_current_track().title if context.get_current_track() else 'None'}")
                    logger.info(f"Playing: {context.playback.is_playing}")
                    continue
                if cmd == "help":
                    logger.info("Commands: a, b, x, y, state, quit")
                    continue

                action = ACTION_MAP.get(cmd)
                if action is None:
                    logger.warning(f"Unknown command: {cmd}")
                    continue

                input_manager.simulate_action(action)
        else:
            logger.info("Demo running on hardware. Press Ctrl+C to exit.")
            while True:
                time.sleep(0.1)

    except KeyboardInterrupt:
        logger.info("Demo interrupted by user")
    finally:
        input_manager.stop()
        display.cleanup()
        logger.info(f"Final state: {state_machine.get_state_name()}")
        logger.info("Demo stopped")

    return 0


if __name__ == "__main__":
    sys.exit(main())
