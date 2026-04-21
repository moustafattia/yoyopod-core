"""Per-domain retry supervision for the scaffold recovery integration."""

from __future__ import annotations

import threading
from dataclasses import dataclass
from typing import Any, Callable

from yoyopod.core.events import BackendStoppedEvent
from yoyopod.integrations.recovery.events import RecoveryAttemptedEvent


RetryHandler = Callable[[], bool]


@dataclass(slots=True)
class _DomainState:
    """Mutable retry bookkeeping for one domain."""

    attempt_count: int = 0
    next_delay_seconds: float = 1.0
    scheduled: bool = False


class RecoverySupervisor:
    """Coordinate recovery retries for integration-owned backends."""

    def __init__(
        self,
        app: Any,
        *,
        initial_delay_seconds: float = 1.0,
        max_delay_seconds: float = 30.0,
        retry_handlers: dict[str, RetryHandler] | None = None,
    ) -> None:
        self._app = app
        self._initial_delay_seconds = max(0.0, float(initial_delay_seconds))
        self._max_delay_seconds = max(self._initial_delay_seconds, float(max_delay_seconds))
        self._retry_handlers = dict(retry_handlers or {})
        self._domains: dict[str, _DomainState] = {}
        self._lock = threading.Lock()
        self._stop = threading.Event()

    def stop(self) -> None:
        """Prevent further retries from being scheduled."""

        self._stop.set()

    def register_retry_handler(self, domain: str, handler: RetryHandler) -> None:
        """Register one recoverable domain."""

        self._retry_handlers[str(domain)] = handler

    def on_backend_stopped(self, event: BackendStoppedEvent) -> None:
        """Schedule recovery after a backend-stopped signal."""

        self.request_recovery(event.domain, reason=event.reason or "backend_stopped")

    def request_recovery(self, domain: str, *, reason: str = "manual") -> None:
        """Schedule one retry cycle for a recoverable domain."""

        domain_name = str(domain)
        if not domain_name or self._stop.is_set():
            return

        with self._lock:
            state = self._domains.setdefault(
                domain_name,
                _DomainState(next_delay_seconds=self._initial_delay_seconds),
            )
            if state.scheduled:
                return
            state.scheduled = True
            delay = state.next_delay_seconds

        worker = threading.Thread(
            target=self._sleep_then_attempt,
            args=(domain_name, reason, delay),
            daemon=True,
            name=f"recovery-{domain_name}",
        )
        worker.start()

    def _sleep_then_attempt(self, domain: str, reason: str, delay: float) -> None:
        if self._stop.wait(delay):
            return
        self._app.scheduler.run_on_main(lambda: self._attempt(domain, reason))

    def _attempt(self, domain: str, reason: str) -> None:
        if self._stop.is_set():
            return

        handler = self._retry_handlers.get(domain)
        with self._lock:
            state = self._domains.setdefault(
                domain,
                _DomainState(next_delay_seconds=self._initial_delay_seconds),
            )
            state.scheduled = False
            state.attempt_count += 1

        success = False
        if handler is not None:
            success = bool(handler())

        self._app.bus.publish(
            RecoveryAttemptedEvent(domain=domain, success=success, reason=reason)
        )

        with self._lock:
            if success:
                state.attempt_count = 0
                state.next_delay_seconds = self._initial_delay_seconds
                return
            state.next_delay_seconds = min(
                max(self._initial_delay_seconds, state.next_delay_seconds * 2.0),
                self._max_delay_seconds,
            )

        self.request_recovery(domain, reason=f"retry_{state.attempt_count}")
