"""Opt-in responsiveness watchdog for target-hardware investigations."""

from __future__ import annotations

import threading
import time
from dataclasses import dataclass
from typing import Callable, Mapping

from yoyopod.utils.logger import get_subsystem_logger

app_logger = get_subsystem_logger("app")

StatusProvider = Callable[[], Mapping[str, object]]
CaptureCallback = Callable[["ResponsivenessWatchdogDecision", Mapping[str, object]], None]
TimeProvider = Callable[[], float]


@dataclass(frozen=True, slots=True)
class ResponsivenessWatchdogDecision:
    """Normalized watchdog decision describing why evidence should be captured."""

    reason: str
    suspected_scope: str
    summary: str


def evaluate_responsiveness_status(
    status: Mapping[str, object],
    *,
    stall_threshold_seconds: float,
    recent_input_window_seconds: float,
) -> ResponsivenessWatchdogDecision | None:
    """Return one capture decision when the runtime looks unresponsive."""

    loop_age = _coerce_float(status.get("loop_heartbeat_age_seconds"))
    if loop_age is None or loop_age < max(0.1, stall_threshold_seconds):
        return None

    input_age = _coerce_float(status.get("input_activity_age_seconds"))
    handled_input_age = _coerce_float(status.get("handled_input_activity_age_seconds"))
    lvgl_age = _coerce_float(status.get("lvgl_pump_age_seconds"))
    pending_callbacks = max(0, _coerce_int(status.get("pending_main_thread_callbacks")) or 0)
    pending_events = max(0, _coerce_int(status.get("pending_event_bus_events")) or 0)
    last_input_action = str(status.get("last_input_action") or "none")
    last_handled_input_action = str(status.get("last_handled_input_action") or "none")
    current_screen = str(status.get("current_screen") or "none")
    current_state = str(status.get("state") or "unknown")
    display_backend = str(status.get("display_backend") or "unknown")

    recent_input = input_age is not None and input_age <= max(0.1, recent_input_window_seconds)
    handled_input_lagging = recent_input and (
        handled_input_age is None
        or handled_input_age >= stall_threshold_seconds
        or handled_input_age > (input_age + 0.5)
    )
    if handled_input_lagging:
        handled_text = "never" if handled_input_age is None else f"{handled_input_age:.1f}s"
        return ResponsivenessWatchdogDecision(
            reason="coordinator_stall_after_input",
            suspected_scope="input_to_runtime_handoff",
            summary=(
                "Loop heartbeat stalled at "
                f"{loop_age:.1f}s while input stayed alive (input_age={input_age:.1f}s, "
                f"handled_input_age={handled_text}, last_input={last_input_action}, "
                f"last_handled_input={last_handled_input_action}, "
                f"pending_callbacks={pending_callbacks}, pending_events={pending_events}, "
                f"screen={current_screen}, state={current_state})"
            ),
        )

    if pending_callbacks > 0 or pending_events > 0:
        return ResponsivenessWatchdogDecision(
            reason="coordinator_stall_with_pending_work",
            suspected_scope="runtime",
            summary=(
                "Loop heartbeat stalled at "
                f"{loop_age:.1f}s with queued work pending "
                f"(callbacks={pending_callbacks}, events={pending_events}, "
                f"screen={current_screen}, state={current_state})"
            ),
        )

    if (
        display_backend == "lvgl"
        and lvgl_age is not None
        and lvgl_age >= stall_threshold_seconds
    ):
        return ResponsivenessWatchdogDecision(
            reason="ui_and_runtime_stall",
            suspected_scope="ui_and_runtime",
            summary=(
                "Loop and LVGL pump both stopped advancing "
                f"(loop_age={loop_age:.1f}s, lvgl_age={lvgl_age:.1f}s, "
                f"screen={current_screen}, state={current_state})"
            ),
        )

    return ResponsivenessWatchdogDecision(
        reason="broad_runtime_stall",
        suspected_scope="runtime",
        summary=(
            "Loop heartbeat stalled without fresh input evidence "
            f"(loop_age={loop_age:.1f}s, screen={current_screen}, state={current_state})"
        ),
    )


class ResponsivenessWatchdog:
    """Background observer that captures evidence when the app loop stops advancing."""

    def __init__(
        self,
        *,
        status_provider: StatusProvider,
        capture_callback: CaptureCallback,
        stall_threshold_seconds: float,
        recent_input_window_seconds: float,
        poll_interval_seconds: float,
        capture_cooldown_seconds: float,
        time_provider: TimeProvider | None = None,
    ) -> None:
        self._status_provider = status_provider
        self._capture_callback = capture_callback
        self._stall_threshold_seconds = max(0.1, float(stall_threshold_seconds))
        self._recent_input_window_seconds = max(
            0.1,
            float(recent_input_window_seconds),
        )
        self._poll_interval_seconds = max(0.1, float(poll_interval_seconds))
        self._capture_cooldown_seconds = max(0.0, float(capture_cooldown_seconds))
        self._time_provider = time.monotonic if time_provider is None else time_provider
        self._stop_event = threading.Event()
        self._thread: threading.Thread | None = None
        self._stall_active = False
        self._last_capture_at = 0.0

    def start(self) -> None:
        """Start the background watchdog thread once."""
        if self._thread is not None:
            return

        self._stop_event.clear()
        self._thread = threading.Thread(
            target=self._run,
            daemon=True,
            name="responsiveness-watchdog",
        )
        self._thread.start()
        app_logger.info(
            "Responsiveness watchdog armed (threshold={}s, poll={}s)",
            self._stall_threshold_seconds,
            self._poll_interval_seconds,
        )

    def stop(self, *, timeout_seconds: float = 2.0) -> None:
        """Stop the background watchdog thread."""
        thread = self._thread
        if thread is None:
            return

        self._stop_event.set()
        thread.join(timeout=max(0.1, timeout_seconds))
        self._thread = None
        app_logger.info("Responsiveness watchdog stopped")

    def poll_once(self) -> ResponsivenessWatchdogDecision | None:
        """Run one check cycle and capture evidence when needed."""

        try:
            status = self._status_provider()
        except Exception:
            app_logger.exception("Responsiveness watchdog failed to collect status")
            return None

        decision = evaluate_responsiveness_status(
            status,
            stall_threshold_seconds=self._stall_threshold_seconds,
            recent_input_window_seconds=self._recent_input_window_seconds,
        )
        if decision is None:
            self._stall_active = False
            return None

        now = self._time_provider()
        if self._stall_active:
            return None
        if (
            self._last_capture_at > 0.0
            and (now - self._last_capture_at) < self._capture_cooldown_seconds
        ):
            self._stall_active = True
            return None

        self._stall_active = True
        self._last_capture_at = now
        try:
            self._capture_callback(decision, status)
        except Exception:
            app_logger.exception("Responsiveness watchdog capture failed")
        return decision

    def _run(self) -> None:
        """Background watchdog loop."""
        while not self._stop_event.wait(self._poll_interval_seconds):
            self.poll_once()


def _coerce_float(value: object) -> float | None:
    """Best-effort float conversion for status values."""

    if value is None:
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def _coerce_int(value: object) -> int | None:
    """Best-effort integer conversion for status values."""

    if value is None:
        return None
    try:
        return int(value)
    except (TypeError, ValueError):
        return None
