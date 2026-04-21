"""Scaffold recovery integration for the Phase A spine rewrite."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from yoyopod.core.events import BackendStoppedEvent
from yoyopod.integrations.recovery.commands import RequestRecoveryCommand
from yoyopod.integrations.recovery.supervisor import RecoverySupervisor


@dataclass(slots=True)
class RecoveryIntegration:
    """Runtime handles owned by the scaffold recovery integration."""

    supervisor: RecoverySupervisor


__all__ = [
    "RecoveryIntegration",
    "RequestRecoveryCommand",
    "setup",
    "teardown",
]


def setup(
    app: Any,
    *,
    initial_delay_seconds: float = 1.0,
    max_delay_seconds: float = 30.0,
) -> RecoveryIntegration:
    """Register the scaffold recovery service and backend-stop subscriber."""

    supervisor = RecoverySupervisor(
        app,
        initial_delay_seconds=initial_delay_seconds,
        max_delay_seconds=max_delay_seconds,
    )
    integration = RecoveryIntegration(supervisor=supervisor)
    app.integrations["recovery"] = integration
    app.recovery_supervisor = supervisor
    app.bus.subscribe(BackendStoppedEvent, supervisor.on_backend_stopped)
    app.services.register(
        "recovery",
        "request_recovery",
        lambda data: _request_recovery(supervisor, data),
    )
    return integration


def teardown(app: Any) -> None:
    """Stop the supervisor and drop recovery helpers."""

    integration = app.integrations.pop("recovery", None)
    if integration is not None:
        integration.supervisor.stop()
    if hasattr(app, "recovery_supervisor"):
        delattr(app, "recovery_supervisor")


def _request_recovery(supervisor: RecoverySupervisor, data: RequestRecoveryCommand) -> None:
    if not isinstance(data, RequestRecoveryCommand):
        raise TypeError("recovery.request_recovery expects RequestRecoveryCommand")
    supervisor.request_recovery(data.domain, reason="manual")
