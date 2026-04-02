"""Tests for the split FSM orchestration layer and compatibility bridge."""

from yoyopy.app_context import AppContext
from yoyopy.fsm import CallFSM, CallInterruptionPolicy, CallSessionState, MusicFSM, MusicState
from yoyopy.state_machine import AppState, StateMachine


def test_music_fsm_transitions() -> None:
    """MusicFSM should support play, pause, and stop transitions."""
    fsm = MusicFSM()

    assert fsm.state == MusicState.IDLE
    assert fsm.transition("play")
    assert fsm.state == MusicState.PLAYING
    assert fsm.transition("pause")
    assert fsm.state == MusicState.PAUSED
    assert fsm.transition("play")
    assert fsm.state == MusicState.PLAYING
    assert fsm.transition("stop")
    assert fsm.state == MusicState.IDLE
    assert not fsm.transition("pause")


def test_call_fsm_transitions() -> None:
    """CallFSM should model incoming, outgoing, connect, and end flows."""
    fsm = CallFSM()

    assert fsm.state == CallSessionState.IDLE
    assert fsm.transition("incoming")
    assert fsm.state == CallSessionState.INCOMING
    assert fsm.transition("connect")
    assert fsm.state == CallSessionState.ACTIVE
    assert fsm.transition("end")
    assert fsm.state == CallSessionState.IDLE
    assert fsm.transition("dial")
    assert fsm.state == CallSessionState.OUTGOING
    assert fsm.transition("end")
    assert fsm.state == CallSessionState.IDLE
    assert not fsm.transition("connect")


def test_call_interruption_policy_pauses_and_resumes_music() -> None:
    """The interruption policy should remember whether playback was interrupted."""
    music_fsm = MusicFSM()
    policy = CallInterruptionPolicy()

    assert not policy.pause_for_call(music_fsm)
    assert not policy.should_auto_resume(auto_resume=True)

    assert music_fsm.transition("play")
    assert policy.pause_for_call(music_fsm)
    assert music_fsm.state == MusicState.PAUSED
    assert policy.should_auto_resume(auto_resume=True)
    assert not policy.should_auto_resume(auto_resume=False)

    policy.clear()
    assert not policy.music_interrupted_by_call


def test_compatibility_state_machine_derives_legacy_state_from_split_fsms() -> None:
    """The compatibility facade should derive legacy AppState values from the new FSMs."""
    state_machine = StateMachine(AppContext())

    state_machine.set_ui_state(AppState.MENU, trigger="test_menu")
    assert state_machine.current_state == AppState.MENU

    state_machine.music_fsm.transition("play")
    state_machine.sync_from_models("playback_playing")
    assert state_machine.current_state == AppState.PLAYING

    state_machine.set_voip_ready(True)
    assert state_machine.current_state == AppState.PLAYING_WITH_VOIP
    assert state_machine.is_playing_with_voip()

    state_machine.call_interruption_policy.pause_for_call(state_machine.music_fsm)
    state_machine.sync_from_models("auto_pause_for_call")
    assert state_machine.current_state == AppState.PAUSED_BY_CALL
    assert state_machine.is_music_paused_by_call()

    state_machine.call_fsm.transition("incoming")
    state_machine.sync_from_models("incoming_call")
    assert state_machine.current_state == AppState.CALL_INCOMING
    assert state_machine.is_in_call()

    state_machine.call_fsm.transition("connect")
    state_machine.sync_from_models("call_connected")
    assert state_machine.current_state == AppState.CALL_ACTIVE_MUSIC_PAUSED
    assert state_machine.has_paused_music_for_call()

    state_machine.call_fsm.transition("end")
    state_machine.music_fsm.transition("play")
    state_machine.call_interruption_policy.clear()
    state_machine.sync_from_models("call_ended")
    assert state_machine.current_state == AppState.PLAYING_WITH_VOIP


def test_legacy_transition_api_still_supports_existing_phase1_flow() -> None:
    """The old transition_to API should still work during the bridge period."""
    state_machine = StateMachine(AppContext())

    assert state_machine.transition_to(AppState.MENU, "open_menu")
    assert state_machine.transition_to(AppState.PLAYING_WITH_VOIP, "select_media_with_voip")
    assert state_machine.transition_to(AppState.PAUSED_BY_CALL, "auto_pause_for_call")
    assert state_machine.transition_to(AppState.CALL_INCOMING, "incoming_call_ringing")
    assert state_machine.transition_to(
        AppState.CALL_ACTIVE_MUSIC_PAUSED,
        "answer_call_resume_after",
    )
    assert state_machine.transition_to(AppState.PLAYING_WITH_VOIP, "call_ended_auto_resume")

    assert state_machine.is_playing_with_voip()
    assert state_machine.is_music_playing()
