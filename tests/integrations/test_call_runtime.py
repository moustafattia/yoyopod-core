"""Focused tests for the call runtime's config-driven status behavior."""

from __future__ import annotations

from pathlib import Path

from yoyopod.core import AppContext
from yoyopod.core.app_state import AppStateRuntime
from yoyopod.integrations.call import (
    CallFSM,
    CallHistoryStore,
    CallInterruptionPolicy,
)
from yoyopod.integrations.call.runtime import CallRuntime
from yoyopod.integrations.call.models import CallState, RegistrationState
from yoyopod.integrations.music import MusicFSM


class _ScreenManagerStub:
    """Small screen-manager double for call-runtime tests."""

    def __init__(self) -> None:
        self.refresh_calls = 0

    def refresh_call_screen_if_visible(self) -> None:
        self.refresh_calls += 1

    def show_incoming_call(self, caller_address: str, caller_name: str) -> None:
        return

    def show_outgoing_call(self, callee_address: str, callee_name: str) -> None:
        return

    def show_in_call(self) -> None:
        return

    def pop_call_screens(self) -> None:
        return

    def refresh_now_playing_screen(self) -> None:
        return


class _ConfigManagerStub:
    """Small config-manager double exposing only the SIP accessors under test."""

    def __init__(self, *, sip_identity: str = "", sip_username: str = "") -> None:
        self._sip_identity = sip_identity
        self._sip_username = sip_username

    def get_sip_identity(self) -> str:
        return self._sip_identity

    def get_sip_username(self) -> str:
        return self._sip_username


class _VoipManagerStub:
    """Tiny VoIP manager surface used for call-history classification tests."""

    def __init__(self) -> None:
        self._caller_info = {
            "address": "sip:friend@example.com",
            "name": "Friend",
            "display_name": "Friend",
        }
        self._call_duration = 0
        self._pending_terminal_action: str | None = None

    def get_caller_info(self) -> dict[str, str]:
        return dict(self._caller_info)

    def get_call_duration(self) -> int:
        return self._call_duration

    def consume_pending_terminal_action(self) -> str | None:
        action = self._pending_terminal_action
        self._pending_terminal_action = None
        return action


def _build_runtime() -> AppStateRuntime:
    """Create the minimal derived runtime required by CallRuntime."""

    return AppStateRuntime(
        music_fsm=MusicFSM(),
        call_fsm=CallFSM(),
        call_interruption_policy=CallInterruptionPolicy(),
    )


def test_registration_change_uses_config_manager_for_voip_configured_status() -> None:
    """VoIP status should come from the canonical config manager, not legacy app config dicts."""

    context = AppContext()
    config_manager = _ConfigManagerStub(sip_username="kid@example.com")
    runtime = _build_runtime()
    screen_manager = _ScreenManagerStub()
    runtime_owner = CallRuntime(
        runtime=runtime,
        screen_manager=screen_manager,
        auto_resume_after_call=True,
        config_manager=config_manager,
        context=context,
        music_backend=None,
        voip_manager_provider=lambda: None,
    )

    runtime_owner.handle_registration_change(RegistrationState.OK)

    assert context.voip.configured is True
    assert context.voip.ready is True
    assert context.voip.running is True
    assert context.voip.registration_state == RegistrationState.OK.value
    assert runtime.voip_ready is True
    assert screen_manager.refresh_calls == 1


def test_availability_change_uses_reported_registration_state() -> None:
    """Availability changes should cache the registration state reported by the backend."""

    context = AppContext()
    config_manager = _ConfigManagerStub(sip_username="kid@example.com")
    runtime = _build_runtime()
    runtime_owner = CallRuntime(
        runtime=runtime,
        screen_manager=_ScreenManagerStub(),
        auto_resume_after_call=True,
        config_manager=config_manager,
        context=context,
        music_backend=None,
        voip_manager_provider=lambda: None,
    )

    runtime_owner.handle_availability_change(False, "backend_stopped", RegistrationState.NONE)

    assert context.voip.configured is True
    assert context.voip.ready is False
    assert context.voip.running is False
    assert context.voip.registration_state == RegistrationState.NONE.value


def test_terminal_call_states_record_rejected_and_failed_history(tmp_path: Path) -> None:
    """Terminal backend states should classify rejected and failed calls explicitly."""

    context = AppContext()
    voip_manager = _VoipManagerStub()
    config_manager = _ConfigManagerStub(sip_username="kid@example.com")
    runtime = _build_runtime()
    runtime_owner = CallRuntime(
        runtime=runtime,
        screen_manager=_ScreenManagerStub(),
        auto_resume_after_call=True,
        config_manager=config_manager,
        context=context,
        music_backend=None,
        voip_manager_provider=lambda: voip_manager,
        call_history_store=CallHistoryStore(tmp_path / "call_history.json"),
    )

    runtime_owner.handle_incoming_call("sip:mama@example.com", "Mama")
    runtime_owner.handle_call_state_change(CallState.INCOMING)
    voip_manager._pending_terminal_action = "reject"
    runtime_owner.handle_call_state_change(CallState.END)

    voip_manager._caller_info = {
        "address": "sip:dad@example.com",
        "name": "Dad",
        "display_name": "Dad",
    }
    runtime_owner.handle_call_state_change(CallState.OUTGOING)
    runtime_owner.handle_call_state_change(CallState.ERROR)

    recent = runtime_owner.call_history_store.list_recent(2)  # type: ignore[union-attr]
    assert recent[0].outcome == "failed"
    assert recent[1].outcome == "rejected"
