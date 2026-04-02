"""
Shared runtime references for YoyoPod coordinators.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from yoyopy.fsm import CallFSM, CallInterruptionPolicy, MusicFSM
from yoyopy.state_machine import StateMachine


@dataclass(slots=True)
class CoordinatorRuntime:
    """Shared app runtime references used by coordinator modules."""

    state_machine: StateMachine
    music_fsm: MusicFSM
    call_fsm: CallFSM
    call_interruption_policy: CallInterruptionPolicy
    screen_manager: Any
    mopidy_client: Any
    now_playing_screen: Any
    call_screen: Any
    incoming_call_screen: Any
    outgoing_call_screen: Any
    in_call_screen: Any
    config: dict[str, Any]
    config_manager: Any
