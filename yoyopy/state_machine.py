"""
Compatibility state machine for YoyoPod.

This module keeps the legacy AppState API alive while delegating playback and
call orchestration to separate MusicFSM and CallFSM instances.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Callable, Dict, List, Optional

from loguru import logger

from yoyopy.app_context import AppContext
from yoyopy.fsm import CallFSM, CallInterruptionPolicy, CallSessionState, MusicFSM, MusicState


class AppState(Enum):
    """Legacy application states."""

    IDLE = "idle"
    MENU = "menu"
    PLAYING = "playing"
    PAUSED = "paused"
    SETTINGS = "settings"
    PLAYLIST = "playlist"
    PLAYLIST_BROWSER = "playlist_browser"
    CALL_IDLE = "call_idle"
    CALL_INCOMING = "call_incoming"
    CALL_OUTGOING = "call_outgoing"
    CALL_ACTIVE = "call_active"
    CALLING = "calling"
    CONNECTING = "connecting"
    ERROR = "error"
    PLAYING_WITH_VOIP = "playing_with_voip"
    PAUSED_BY_CALL = "paused_by_call"
    CALL_ACTIVE_MUSIC_PAUSED = "call_active_music_paused"


@dataclass
class StateTransition:
    """Represents a legacy compatibility transition."""

    from_state: AppState
    to_state: AppState
    trigger: str
    guard: Optional[Callable[[], bool]] = None


class StateMachine:
    """
    Compatibility facade over split playback and call state machines.

    Legacy callers can still transition with AppState values while new code can
    mutate `music_fsm`, `call_fsm`, and `call_interruption_policy` directly, then
    call `sync_from_models()` to refresh the derived AppState view.
    """

    _UI_STATES = {
        AppState.IDLE,
        AppState.MENU,
        AppState.SETTINGS,
        AppState.PLAYLIST,
        AppState.PLAYLIST_BROWSER,
        AppState.CALL_IDLE,
        AppState.CONNECTING,
        AppState.ERROR,
    }

    def __init__(self, context: AppContext) -> None:
        self.context = context
        self.music_fsm = MusicFSM()
        self.call_fsm = CallFSM()
        self.call_interruption_policy = CallInterruptionPolicy()
        self.voip_ready = False
        self.ui_state = AppState.IDLE

        self.current_state = AppState.IDLE
        self.previous_state: Optional[AppState] = None
        self.state_history: List[AppState] = [AppState.IDLE]

        self.on_enter_callbacks: Dict[AppState, List[Callable[[], None]]] = {
            state: [] for state in AppState
        }
        self.on_exit_callbacks: Dict[AppState, List[Callable[[], None]]] = {
            state: [] for state in AppState
        }
        self.transitions = self._define_transitions()

        logger.info(f"StateMachine initialized in {self.current_state.value} state")

    def _define_transitions(self) -> List[StateTransition]:
        """Retain the legacy transition table for compatibility validation."""
        return [
            StateTransition(AppState.IDLE, AppState.MENU, "open_menu"),
            StateTransition(AppState.IDLE, AppState.SETTINGS, "open_settings"),
            StateTransition(AppState.IDLE, AppState.CONNECTING, "connect"),
            StateTransition(AppState.IDLE, AppState.CALL_INCOMING, "incoming_call"),
            StateTransition(AppState.MENU, AppState.IDLE, "back"),
            StateTransition(AppState.MENU, AppState.PLAYING, "select_media"),
            StateTransition(AppState.MENU, AppState.PLAYLIST, "select_playlist"),
            StateTransition(AppState.MENU, AppState.PLAYLIST_BROWSER, "browse_playlists"),
            StateTransition(AppState.MENU, AppState.CALL_IDLE, "open_voip"),
            StateTransition(AppState.MENU, AppState.SETTINGS, "select_settings"),
            StateTransition(AppState.PLAYING, AppState.PAUSED, "pause"),
            StateTransition(AppState.PLAYING, AppState.MENU, "back"),
            StateTransition(AppState.PLAYING, AppState.IDLE, "stop"),
            StateTransition(AppState.PAUSED, AppState.PLAYING, "resume"),
            StateTransition(AppState.PAUSED, AppState.PLAYING_WITH_VOIP, "resume"),
            StateTransition(AppState.PAUSED, AppState.MENU, "back"),
            StateTransition(AppState.PAUSED, AppState.IDLE, "stop"),
            StateTransition(AppState.PLAYLIST, AppState.MENU, "back"),
            StateTransition(AppState.PLAYLIST, AppState.PLAYING, "select_track"),
            StateTransition(AppState.PLAYLIST_BROWSER, AppState.MENU, "back"),
            StateTransition(AppState.PLAYLIST_BROWSER, AppState.PLAYING, "load_playlist"),
            StateTransition(AppState.CALL_IDLE, AppState.MENU, "back"),
            StateTransition(AppState.CALL_IDLE, AppState.CALL_OUTGOING, "make_call"),
            StateTransition(AppState.CALL_IDLE, AppState.CALL_INCOMING, "incoming_call"),
            StateTransition(AppState.CALL_IDLE, AppState.CALLING, "make_call"),
            StateTransition(AppState.CALL_IDLE, AppState.CALLING, "incoming_call"),
            StateTransition(AppState.CALL_INCOMING, AppState.CALL_ACTIVE, "answer_call"),
            StateTransition(AppState.CALL_INCOMING, AppState.CALL_ACTIVE, "call_connected"),
            StateTransition(AppState.CALL_INCOMING, AppState.CALL_IDLE, "reject_call"),
            StateTransition(AppState.CALL_INCOMING, AppState.CALL_IDLE, "call_ended"),
            StateTransition(AppState.CALL_OUTGOING, AppState.CALL_ACTIVE, "call_connected"),
            StateTransition(AppState.CALL_OUTGOING, AppState.CALL_IDLE, "cancel_call"),
            StateTransition(AppState.CALL_OUTGOING, AppState.CALL_IDLE, "call_failed"),
            StateTransition(AppState.CALL_ACTIVE, AppState.CALL_IDLE, "end_call"),
            StateTransition(AppState.CALL_ACTIVE, AppState.CALL_IDLE, "call_ended"),
            StateTransition(AppState.CALLING, AppState.CALL_IDLE, "call_ended"),
            StateTransition(AppState.CALLING, AppState.MENU, "back"),
            StateTransition(AppState.SETTINGS, AppState.MENU, "back"),
            StateTransition(AppState.SETTINGS, AppState.IDLE, "home"),
            StateTransition(AppState.CONNECTING, AppState.IDLE, "cancel"),
            StateTransition(AppState.CONNECTING, AppState.MENU, "connected"),
            StateTransition(AppState.ERROR, AppState.IDLE, "reset"),
            StateTransition(AppState.PLAYING, AppState.PLAYING_WITH_VOIP, "voip_ready"),
            StateTransition(AppState.PLAYLIST_BROWSER, AppState.PLAYING_WITH_VOIP, "load_playlist_with_voip"),
            StateTransition(AppState.MENU, AppState.PLAYING_WITH_VOIP, "select_media_with_voip"),
            StateTransition(AppState.PLAYING_WITH_VOIP, AppState.PAUSED, "pause"),
            StateTransition(AppState.PLAYING_WITH_VOIP, AppState.MENU, "back"),
            StateTransition(AppState.PLAYING_WITH_VOIP, AppState.IDLE, "stop"),
            StateTransition(AppState.PLAYING_WITH_VOIP, AppState.PAUSED_BY_CALL, "auto_pause_for_call"),
            StateTransition(AppState.PLAYING_WITH_VOIP, AppState.CALL_INCOMING, "incoming_call"),
            StateTransition(AppState.PAUSED_BY_CALL, AppState.CALL_INCOMING, "incoming_call_ringing"),
            StateTransition(AppState.PAUSED_BY_CALL, AppState.CALL_ACTIVE_MUSIC_PAUSED, "call_answered"),
            StateTransition(AppState.PAUSED_BY_CALL, AppState.PLAYING_WITH_VOIP, "call_rejected_resume"),
            StateTransition(AppState.PAUSED_BY_CALL, AppState.PAUSED, "call_rejected_stay_paused"),
            StateTransition(AppState.CALL_INCOMING, AppState.CALL_ACTIVE_MUSIC_PAUSED, "answer_call_resume_after"),
            StateTransition(AppState.CALL_INCOMING, AppState.PAUSED, "reject_call_stay_paused"),
            StateTransition(AppState.CALL_INCOMING, AppState.PLAYING_WITH_VOIP, "reject_call_resume"),
            StateTransition(AppState.CALL_INCOMING, AppState.PLAYING_WITH_VOIP, "call_ended_auto_resume"),
            StateTransition(AppState.CALL_ACTIVE_MUSIC_PAUSED, AppState.PLAYING_WITH_VOIP, "call_ended_auto_resume"),
            StateTransition(AppState.CALL_ACTIVE_MUSIC_PAUSED, AppState.PAUSED, "call_ended_stay_paused"),
            StateTransition(AppState.CALL_ACTIVE_MUSIC_PAUSED, AppState.MENU, "call_ended_stop_music"),
            StateTransition(AppState.PLAYING_WITH_VOIP, AppState.CALL_OUTGOING, "make_call_pause_music"),
            StateTransition(AppState.CALL_OUTGOING, AppState.CALL_ACTIVE_MUSIC_PAUSED, "call_connected_music_paused"),
        ]

    def can_transition(self, to_state: AppState, trigger: str = "manual") -> bool:
        """Check if a legacy transition is currently allowed."""
        if to_state == self.current_state:
            return True

        for transition in self.transitions:
            if (
                transition.from_state == self.current_state
                and transition.to_state == to_state
                and transition.trigger == trigger
            ):
                if transition.guard and not transition.guard():
                    logger.warning(
                        f"Transition {self.current_state.value} -> {to_state.value} blocked by guard"
                    )
                    return False
                return True

        logger.warning(
            f"Invalid transition: {self.current_state.value} -> {to_state.value} "
            f"(trigger: {trigger})"
        )
        return False

    def transition_to(self, new_state: AppState, trigger: str = "manual") -> bool:
        """Perform a legacy transition and keep the new FSMs aligned."""
        if not self.can_transition(new_state, trigger):
            logger.error(
                f"Cannot transition from {self.current_state.value} to {new_state.value}"
            )
            return False

        if new_state == self.current_state:
            logger.debug(f"Already in {new_state.value} state")
            return True

        self._apply_legacy_state(new_state, trigger)
        self._set_state(new_state, trigger)
        return True

    def _apply_legacy_state(self, new_state: AppState, trigger: str) -> None:
        """Update the split FSMs to reflect a legacy transition."""
        if trigger == "voip_ready":
            self.voip_ready = True

        if new_state in self._UI_STATES:
            self.ui_state = new_state

        if new_state == AppState.IDLE:
            self.music_fsm.sync(MusicState.IDLE)
            self.call_fsm.sync(CallSessionState.IDLE)
            self.call_interruption_policy.clear()
            self.voip_ready = False if trigger == "reset" else self.voip_ready
            return

        if new_state in (AppState.PLAYING, AppState.PLAYING_WITH_VOIP):
            self.music_fsm.sync(MusicState.PLAYING)
            self.call_fsm.sync(CallSessionState.IDLE)
            self.call_interruption_policy.clear()
            if new_state == AppState.PLAYING_WITH_VOIP:
                self.voip_ready = True
            return

        if new_state == AppState.PAUSED:
            self.music_fsm.sync(MusicState.PAUSED)
            if trigger not in {"auto_pause_for_call", "call_answered", "answer_call_resume_after"}:
                self.call_interruption_policy.clear()
            return

        if new_state == AppState.PAUSED_BY_CALL:
            self.music_fsm.sync(MusicState.PAUSED)
            self.call_fsm.sync(CallSessionState.IDLE)
            self.call_interruption_policy.music_interrupted_by_call = True
            return

        if new_state == AppState.CALL_INCOMING:
            self.call_fsm.sync(CallSessionState.INCOMING)
            return

        if new_state == AppState.CALL_OUTGOING:
            self.call_fsm.sync(CallSessionState.OUTGOING)
            return

        if new_state in (AppState.CALL_ACTIVE, AppState.CALLING):
            self.call_fsm.sync(CallSessionState.ACTIVE)
            if new_state == AppState.CALL_ACTIVE:
                self.call_interruption_policy.clear()
            return

        if new_state == AppState.CALL_ACTIVE_MUSIC_PAUSED:
            self.music_fsm.sync(MusicState.PAUSED)
            self.call_fsm.sync(CallSessionState.ACTIVE)
            self.call_interruption_policy.music_interrupted_by_call = True

    def _set_state(self, new_state: AppState, trigger: str) -> None:
        """Update the current legacy state and fire callbacks."""
        old_state = self.current_state
        logger.info(f"Exiting state: {old_state.value}")
        self._fire_callbacks(self.on_exit_callbacks[old_state])

        self.previous_state = old_state
        self.current_state = new_state
        self.state_history.append(new_state)
        if len(self.state_history) > 50:
            self.state_history = self.state_history[-50:]

        logger.info(f"Entering state: {new_state.value} (trigger: {trigger})")
        self._fire_callbacks(self.on_enter_callbacks[new_state])

    def sync_from_models(self, trigger: str = "sync") -> AppState:
        """
        Refresh the legacy AppState view from the split FSMs.

        Returns:
            The newly derived AppState.
        """
        derived_state = self._derive_state()
        if derived_state == self.current_state:
            return derived_state

        self._set_state(derived_state, trigger)
        return derived_state

    def _derive_state(self) -> AppState:
        """Derive the compatibility AppState from the split FSMs."""
        if self.call_fsm.state == CallSessionState.INCOMING:
            return AppState.CALL_INCOMING

        if self.call_fsm.state == CallSessionState.OUTGOING:
            return AppState.CALL_OUTGOING

        if self.call_fsm.state == CallSessionState.ACTIVE:
            if self.call_interruption_policy.music_interrupted_by_call:
                return AppState.CALL_ACTIVE_MUSIC_PAUSED
            return AppState.CALL_ACTIVE

        if (
            self.call_interruption_policy.music_interrupted_by_call
            and self.music_fsm.state == MusicState.PAUSED
        ):
            return AppState.PAUSED_BY_CALL

        if self.music_fsm.state == MusicState.PLAYING:
            return AppState.PLAYING_WITH_VOIP if self.voip_ready else AppState.PLAYING

        if self.music_fsm.state == MusicState.PAUSED:
            return AppState.PAUSED

        return self.ui_state

    def set_ui_state(self, state: AppState, trigger: str = "ui_state") -> AppState:
        """Update the base UI state used when music/calls are idle."""
        if state not in self._UI_STATES:
            raise ValueError(f"{state.value} is not a base UI state")

        self.ui_state = state
        return self.sync_from_models(trigger)

    def set_voip_ready(self, ready: bool, trigger: str = "voip_ready") -> AppState:
        """Set whether VoIP is ready to receive calls and refresh derived state."""
        self.voip_ready = ready
        actual_trigger = trigger if ready else "voip_unavailable"
        return self.sync_from_models(actual_trigger)

    def go_back(self) -> bool:
        """Go back to the previous legacy state."""
        if len(self.state_history) < 2:
            logger.warning("Cannot go back: no previous state")
            return False

        previous = self.state_history[-2]
        return self.transition_to(previous, "back")

    def on_enter(self, state: AppState, callback: Callable[[], None]) -> None:
        """Register an on-enter callback."""
        self.on_enter_callbacks[state].append(callback)
        logger.debug(f"Registered on_enter callback for {state.value}")

    def on_exit(self, state: AppState, callback: Callable[[], None]) -> None:
        """Register an on-exit callback."""
        self.on_exit_callbacks[state].append(callback)
        logger.debug(f"Registered on_exit callback for {state.value}")

    def _fire_callbacks(self, callbacks: List[Callable[[], None]]) -> None:
        """Fire a callback list safely."""
        for callback in callbacks:
            try:
                callback()
            except Exception as exc:
                logger.error(f"Error in state callback: {exc}")

    def get_valid_transitions(self) -> List[AppState]:
        """Return legacy target states allowed from the current compatibility state."""
        valid_states: List[AppState] = []
        for transition in self.transitions:
            if transition.from_state == self.current_state:
                if not transition.guard or transition.guard():
                    if transition.to_state not in valid_states:
                        valid_states.append(transition.to_state)
        return valid_states

    def reset(self) -> None:
        """Reset to the initial idle state."""
        logger.info("Resetting state machine")
        self.music_fsm.sync(MusicState.IDLE)
        self.call_fsm.sync(CallSessionState.IDLE)
        self.call_interruption_policy.clear()
        self.voip_ready = False
        self.ui_state = AppState.IDLE
        self._set_state(AppState.IDLE, "reset")
        self.state_history = [AppState.IDLE]
        self.previous_state = None

    def open_menu(self) -> bool:
        """Transition to MENU state."""
        return self.transition_to(AppState.MENU, "open_menu")

    def start_playback(self) -> bool:
        """Transition to PLAYING state."""
        if self.transition_to(AppState.PLAYING, "select_media"):
            self.context.play()
            return True
        return False

    def pause_playback(self) -> bool:
        """Transition to PAUSED state."""
        if self.transition_to(AppState.PAUSED, "pause"):
            self.context.pause()
            return True
        return False

    def resume_playback(self) -> bool:
        """Transition to PLAYING state from PAUSED."""
        if self.transition_to(AppState.PLAYING, "resume"):
            self.context.resume()
            return True
        return False

    def stop_playback(self) -> bool:
        """Stop playback and return to IDLE."""
        if self.transition_to(AppState.IDLE, "stop"):
            self.context.stop()
            return True
        return False

    def toggle_playback(self) -> bool:
        """Toggle between playing and paused states."""
        if self.music_fsm.state == MusicState.PLAYING:
            self.pause_playback()
            return False
        if self.music_fsm.state == MusicState.PAUSED:
            self.resume_playback()
            return True

        self.start_playback()
        return True

    def open_settings(self) -> bool:
        """Transition to SETTINGS state."""
        return self.transition_to(AppState.SETTINGS, "select_settings")

    def get_state_name(self) -> str:
        """Get the current legacy state name."""
        return self.current_state.value

    def is_playing(self) -> bool:
        """Check whether music is actively playing."""
        return self.music_fsm.state == MusicState.PLAYING

    def is_idle(self) -> bool:
        """Check whether the app is idle."""
        return (
            self.music_fsm.state == MusicState.IDLE
            and self.call_fsm.state == CallSessionState.IDLE
            and self.current_state == AppState.IDLE
        )

    def is_playing_with_voip(self) -> bool:
        """Check whether music is playing while VoIP is ready."""
        return self.music_fsm.state == MusicState.PLAYING and self.voip_ready

    def is_music_paused_by_call(self) -> bool:
        """Check whether music is paused because of call interruption."""
        return self.call_interruption_policy.music_interrupted_by_call

    def is_in_call(self) -> bool:
        """Check if any call session is in progress."""
        return self.call_fsm.is_active

    def is_music_playing(self) -> bool:
        """Check whether music is currently playing."""
        return self.music_fsm.state == MusicState.PLAYING

    def has_paused_music_for_call(self) -> bool:
        """Check whether the policy is holding paused music for a call."""
        return self.call_interruption_policy.music_interrupted_by_call

    def is_call_active(self) -> bool:
        """Check if a call is in progress."""
        return self.call_fsm.is_active
