"""Loopback-mode tests for :class:`SidecarSupervisor`.

Loopback runs the sidecar entry point on a daemon thread inside the calling
process instead of spawning a subprocess. The protocol, handshake, restart
logic, and event dispatch flow are identical to the process runner, which
makes loopback the right harness for integration tests that exercise the
real sidecar code without paying ``spawn`` / ``forkserver`` cost or risking
multiprocessing-related CI flakes.
"""

from __future__ import annotations

import threading
import time
from collections.abc import Callable
from multiprocessing.connection import Connection
from typing import Any

import pytest

from yoyopod.integrations.call.sidecar_main import run_sidecar
from yoyopod.integrations.call.sidecar_protocol import (
    Hello,
    PROTOCOL_VERSION,
    Ping,
    Pong,
    Ready,
    Register,
    send_message,
)
from yoyopod.integrations.call.sidecar_supervisor import (
    RestartPolicy,
    SidecarSupervisor,
)

LOOPBACK_BUDGET_SECONDS = 2.0
"""Loopback has no spawn cost, so timeouts can be tight."""


# ---------------------------------------------------------------------------
# Module-level loopback targets (no pickling required, but kept module-level
# for symmetry with the process tests)
# ---------------------------------------------------------------------------


def _exit_after_first_command_loopback_target(conn: Connection) -> None:
    """Loopback target that handshakes then exits on the first command."""

    send_message(conn, Hello(version=PROTOCOL_VERSION))
    try:
        conn.recv_bytes()  # peer hello
    except (BrokenPipeError, EOFError, OSError):
        return
    send_message(conn, Ready())
    try:
        conn.recv_bytes()  # first command, then exit
    except (BrokenPipeError, EOFError, OSError):
        pass


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def collected_events() -> list[Any]:
    return []


@pytest.fixture
def event_handler(collected_events: list[Any]) -> Callable[[Any], None]:
    return collected_events.append


# ---------------------------------------------------------------------------
# Happy-path lifecycle
# ---------------------------------------------------------------------------


def test_loopback_start_drives_supervisor_to_running_state(
    event_handler: Callable[[Any], None],
) -> None:
    supervisor = SidecarSupervisor(
        on_event=event_handler,
        sidecar_target=run_sidecar,
        use_loopback=True,
        handshake_timeout_seconds=LOOPBACK_BUDGET_SECONDS,
    )
    try:
        supervisor.start()
        assert supervisor.wait_for_state("running", timeout_seconds=LOOPBACK_BUDGET_SECONDS)
    finally:
        supervisor.stop(timeout_seconds=LOOPBACK_BUDGET_SECONDS)
        assert supervisor.state_snapshot().state == "stopped"


def test_loopback_send_ping_round_trips_to_pong(
    collected_events: list[Any], event_handler: Callable[[Any], None]
) -> None:
    supervisor = SidecarSupervisor(
        on_event=event_handler,
        sidecar_target=run_sidecar,
        use_loopback=True,
        handshake_timeout_seconds=LOOPBACK_BUDGET_SECONDS,
    )
    try:
        supervisor.start()
        assert supervisor.wait_for_state("running", timeout_seconds=LOOPBACK_BUDGET_SECONDS)
        supervisor.send(Ping(cmd_id=42))

        deadline = time.monotonic() + LOOPBACK_BUDGET_SECONDS
        while time.monotonic() < deadline:
            if any(isinstance(event, Pong) and event.cmd_id == 42 for event in collected_events):
                break
            time.sleep(0.01)

        assert any(
            isinstance(event, Pong) and event.cmd_id == 42 for event in collected_events
        ), f"never received Pong; saw {[type(e).__name__ for e in collected_events]}"
    finally:
        supervisor.stop(timeout_seconds=LOOPBACK_BUDGET_SECONDS)


def test_loopback_register_command_flows_to_log_event(
    collected_events: list[Any], event_handler: Callable[[Any], None]
) -> None:
    """Verify the scaffold ack-as-Log path works through the loopback runner."""

    supervisor = SidecarSupervisor(
        on_event=event_handler,
        sidecar_target=run_sidecar,
        use_loopback=True,
        handshake_timeout_seconds=LOOPBACK_BUDGET_SECONDS,
    )
    try:
        supervisor.start()
        assert supervisor.wait_for_state("running", timeout_seconds=LOOPBACK_BUDGET_SECONDS)
        supervisor.send(Register(server="sip.example.com", user="alice", password="x", cmd_id=1))

        deadline = time.monotonic() + LOOPBACK_BUDGET_SECONDS
        while time.monotonic() < deadline:
            log_events = [event for event in collected_events if type(event).__name__ == "Log"]
            if any("alice" in event.message for event in log_events):
                break
            time.sleep(0.01)

        log_events = [event for event in collected_events if type(event).__name__ == "Log"]
        assert any(
            "alice" in event.message for event in log_events
        ), f"never saw Register ack log; collected {collected_events!r}"
    finally:
        supervisor.stop(timeout_seconds=LOOPBACK_BUDGET_SECONDS)


# ---------------------------------------------------------------------------
# Restart and shutdown discipline
# ---------------------------------------------------------------------------


def test_loopback_unexpected_exit_triggers_restart(
    event_handler: Callable[[Any], None],
) -> None:
    """An unexpected sidecar-thread exit should still trip the restart policy."""

    supervisor = SidecarSupervisor(
        on_event=event_handler,
        sidecar_target=_exit_after_first_command_loopback_target,
        use_loopback=True,
        handshake_timeout_seconds=LOOPBACK_BUDGET_SECONDS,
        restart_policy=RestartPolicy(
            max_failures=5,
            failure_window_seconds=60.0,
            backoff_initial_seconds=0.05,
            backoff_factor=1.0,
            backoff_max_seconds=0.05,
        ),
    )
    try:
        supervisor.start()
        assert supervisor.wait_for_state("running", timeout_seconds=LOOPBACK_BUDGET_SECONDS)
        supervisor.send(Ping(cmd_id=1))

        deadline = time.monotonic() + LOOPBACK_BUDGET_SECONDS
        while time.monotonic() < deadline:
            snapshot = supervisor.state_snapshot()
            if snapshot.restart_count >= 1 and snapshot.state == "running":
                break
            time.sleep(0.01)

        snapshot = supervisor.state_snapshot()
        assert snapshot.restart_count >= 1, snapshot
        assert snapshot.state == "running", snapshot
    finally:
        supervisor.stop(timeout_seconds=LOOPBACK_BUDGET_SECONDS)


def test_loopback_intentional_stop_does_not_trigger_restart(
    event_handler: Callable[[Any], None],
) -> None:
    supervisor = SidecarSupervisor(
        on_event=event_handler,
        sidecar_target=run_sidecar,
        use_loopback=True,
        handshake_timeout_seconds=LOOPBACK_BUDGET_SECONDS,
        restart_policy=RestartPolicy(
            max_failures=5,
            failure_window_seconds=60.0,
            backoff_initial_seconds=0.05,
            backoff_factor=1.0,
            backoff_max_seconds=0.05,
        ),
    )
    supervisor.start()
    assert supervisor.wait_for_state("running", timeout_seconds=LOOPBACK_BUDGET_SECONDS)

    supervisor.stop(timeout_seconds=LOOPBACK_BUDGET_SECONDS)
    time.sleep(0.2)
    snapshot = supervisor.state_snapshot()
    assert snapshot.state == "stopped"
    assert snapshot.restart_count == 0


# ---------------------------------------------------------------------------
# Loopback-specific safety
# ---------------------------------------------------------------------------


def test_loopback_uses_thread_runner_not_subprocess(
    event_handler: Callable[[Any], None],
) -> None:
    """Sanity check: loopback-mode runner is a Thread, not a Process."""

    supervisor = SidecarSupervisor(
        on_event=event_handler,
        sidecar_target=run_sidecar,
        use_loopback=True,
        handshake_timeout_seconds=LOOPBACK_BUDGET_SECONDS,
    )
    try:
        supervisor.start()
        assert supervisor.wait_for_state("running", timeout_seconds=LOOPBACK_BUDGET_SECONDS)
        # Read the private runner reference for this assertion only; it is the
        # one place the test actually needs to verify which runner class is in use.
        assert isinstance(supervisor._process, threading.Thread)
    finally:
        supervisor.stop(timeout_seconds=LOOPBACK_BUDGET_SECONDS)


def test_loopback_handshake_completes_faster_than_process_budget(
    event_handler: Callable[[Any], None],
) -> None:
    """Loopback must not depend on Python interpreter spawn; handshake should be sub-second."""

    supervisor = SidecarSupervisor(
        on_event=event_handler,
        sidecar_target=run_sidecar,
        use_loopback=True,
        handshake_timeout_seconds=0.5,  # would never pass with spawn on Windows
    )
    try:
        started_at = time.monotonic()
        supervisor.start()
        elapsed = time.monotonic() - started_at
        assert supervisor.state_snapshot().state == "running"
        # Sub-second handshake confirms no process-spawn cost.
        assert elapsed < 0.5, f"loopback start took {elapsed:.3f}s, expected <0.5s"
    finally:
        supervisor.stop(timeout_seconds=LOOPBACK_BUDGET_SECONDS)
