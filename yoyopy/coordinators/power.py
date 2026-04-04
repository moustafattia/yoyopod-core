"""
Power telemetry coordination for YoyoPod.
"""

from __future__ import annotations

from loguru import logger

from yoyopy.app_context import AppContext
from yoyopy.coordinators.runtime import CoordinatorRuntime
from yoyopy.coordinators.screen import ScreenCoordinator
from yoyopy.event_bus import EventBus
from yoyopy.power import (
    PowerAvailabilityChanged,
    PowerSnapshot,
    PowerSnapshotUpdated,
)


class PowerCoordinator:
    """Own power event publishing and UI/runtime power sync."""

    def __init__(
        self,
        runtime: CoordinatorRuntime,
        screen_coordinator: ScreenCoordinator,
        context: AppContext,
    ) -> None:
        self.runtime = runtime
        self.screen_coordinator = screen_coordinator
        self.context = context
        self._event_bus: EventBus | None = None
        self._bound = False

    def bind(self, event_bus: EventBus) -> None:
        """Bind typed power event subscriptions once."""
        if self._bound:
            return

        self._event_bus = event_bus
        event_bus.subscribe(PowerSnapshotUpdated, self._on_snapshot_updated)
        event_bus.subscribe(PowerAvailabilityChanged, self._on_availability_changed)
        self._bound = True

    def publish_snapshot(self, snapshot: PowerSnapshot) -> None:
        """Publish a refreshed power snapshot onto the event bus."""
        if self._event_bus is None:
            raise RuntimeError("PowerCoordinator is not bound to an EventBus")

        self._event_bus.publish(PowerSnapshotUpdated(snapshot=snapshot))

    def publish_availability_change(self, available: bool, reason: str = "") -> None:
        """Publish power backend availability changes onto the event bus."""
        if self._event_bus is None:
            raise RuntimeError("PowerCoordinator is not bound to an EventBus")

        self._event_bus.publish(PowerAvailabilityChanged(available=available, reason=reason))

    def _on_snapshot_updated(self, event: PowerSnapshotUpdated) -> None:
        self.handle_snapshot_updated(event.snapshot)

    def _on_availability_changed(self, event: PowerAvailabilityChanged) -> None:
        self.handle_availability_change(event.available, event.reason)

    def handle_snapshot_updated(self, snapshot: PowerSnapshot) -> None:
        """Apply the latest power telemetry to runtime and UI state."""
        previous_signature = (
            self.context.battery_percent,
            self.context.battery_charging,
            self.context.external_power,
            self.context.power_available,
            self.context.power_error,
        )

        self.runtime.set_power_snapshot(snapshot)
        self.context.update_power_status(snapshot)

        current_signature = (
            self.context.battery_percent,
            self.context.battery_charging,
            self.context.external_power,
            self.context.power_available,
            self.context.power_error,
        )

        if current_signature != previous_signature:
            self.screen_coordinator.refresh_current_screen()

    def handle_availability_change(self, available: bool, reason: str) -> None:
        """Track power backend reachability for the runtime."""
        self.runtime.set_power_available(available)
        if available:
            logger.info(f"Power backend available ({reason or 'ready'})")
            return

        logger.warning(f"Power backend unavailable ({reason or 'unknown'})")
