"""EventBus and manager callback wiring for ``YoyoPodApp``."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from yoyopod.events import (
    NetworkGpsFixEvent,
    NetworkGpsNoFixEvent,
    NetworkPppDownEvent,
    NetworkPppUpEvent,
    NetworkSignalUpdateEvent,
    RecoveryAttemptCompletedEvent,
    ScreenChangedEvent,
    UserActivityEvent,
)
from yoyopod.power import (
    GracefulShutdownCancelled,
    GracefulShutdownRequested,
    LowBatteryWarningRaised,
)

if TYPE_CHECKING:
    from yoyopod.app import YoyoPodApp


class RuntimeEventWiring:
    """Own event-handler logic and EventBus subscription wiring."""

    def __init__(self, app: "YoyoPodApp") -> None:
        self.app = app

    def register(self) -> None:
        """Subscribe app runtime handlers on the typed EventBus."""
        self.app.event_bus.subscribe(ScreenChangedEvent, self.handle_screen_changed_event)
        self.app.event_bus.subscribe(UserActivityEvent, self.handle_user_activity_event)
        self.app.event_bus.subscribe(
            RecoveryAttemptCompletedEvent,
            self.handle_recovery_attempt_completed_event,
        )
        self.app.event_bus.subscribe(
            LowBatteryWarningRaised,
            self.handle_low_battery_warning_event,
        )
        self.app.event_bus.subscribe(
            GracefulShutdownRequested,
            self.handle_graceful_shutdown_requested_event,
        )
        self.app.event_bus.subscribe(
            GracefulShutdownCancelled,
            self.handle_graceful_shutdown_cancelled_event,
        )
        self.app.event_bus.subscribe(NetworkPppUpEvent, self.handle_network_ppp_up)
        self.app.event_bus.subscribe(NetworkSignalUpdateEvent, self.handle_network_signal_update)
        self.app.event_bus.subscribe(NetworkGpsFixEvent, self.handle_network_gps_fix)
        self.app.event_bus.subscribe(NetworkGpsNoFixEvent, self.handle_network_gps_no_fix)
        self.app.event_bus.subscribe(NetworkPppDownEvent, self.handle_network_ppp_down)

    def handle_voice_note_summary_changed(
        self,
        unread_voice_notes: int,
        latest_voice_note_by_contact: dict[str, dict[str, object]],
    ) -> None:
        """Keep Talk voice-note summary state in sync with the VoIP manager."""
        if self.app.context is None:
            return
        self.app.context.update_voice_note_summary(
            unread_voice_notes=unread_voice_notes,
            latest_voice_note_by_contact=latest_voice_note_by_contact,
        )
        self.refresh_talk_related_screen()

    def handle_voice_note_activity_changed(self, *_args: Any) -> None:
        """Refresh active draft state after a message or delivery update."""
        self.sync_active_voice_note_context()
        self.app._refresh_talk_summary()
        self.refresh_talk_related_screen()

    def handle_voice_note_failure(self, *_args: Any) -> None:
        """Refresh draft state after a failed message operation."""
        self.sync_active_voice_note_context()
        self.refresh_talk_related_screen()

    def sync_active_voice_note_context(self) -> None:
        """Mirror the active voice-note draft into the shared app context."""
        if self.app.context is None or self.app.voip_manager is None:
            return
        draft = self.app.voip_manager.get_active_voice_note()
        if draft is None:
            self.app.context.update_active_voice_note(send_state="idle")
            return
        self.app.context.update_active_voice_note(
            send_state=draft.send_state,
            status_text=draft.status_text,
            file_path=draft.file_path,
            duration_ms=draft.duration_ms,
        )

    def refresh_talk_related_screen(self) -> None:
        """Re-render Talk screens when their message state changes."""
        if self.app.screen_manager is None:
            return
        current_screen = self.app.screen_manager.get_current_screen()
        if current_screen is None:
            return
        if current_screen.route_name in {"call", "talk_contact", "voice_note"}:
            self.app.screen_manager.refresh_current_screen()

    def handle_screen_changed_event(self, event: ScreenChangedEvent) -> None:
        self.app.screen_power_service.handle_screen_changed_event(event)

    def handle_user_activity_event(self, event: UserActivityEvent) -> None:
        self.app.screen_power_service.handle_user_activity_event(event)

    def handle_recovery_attempt_completed_event(
        self,
        event: RecoveryAttemptCompletedEvent,
    ) -> None:
        self.app.recovery_service.handle_recovery_attempt_completed_event(event)

    def handle_low_battery_warning_event(self, event: LowBatteryWarningRaised) -> None:
        self.app.screen_power_service.handle_low_battery_warning_event(event)

    def handle_graceful_shutdown_requested_event(
        self,
        event: GracefulShutdownRequested,
    ) -> None:
        self.app.shutdown_service.handle_graceful_shutdown_requested_event(event)

    def handle_graceful_shutdown_cancelled_event(
        self,
        event: GracefulShutdownCancelled,
    ) -> None:
        self.app.shutdown_service.handle_graceful_shutdown_cancelled_event(event)

    def cellular_connection_type(self) -> str:
        """Return a best-effort cellular connection type for degraded status chrome."""
        if self.app.network_manager is None or not self.app.network_manager.config.enabled:
            return "none"

        from yoyopod.network.models import ModemPhase

        state = self.app.network_manager.modem_state
        if state.phase == ModemPhase.OFF:
            return "none"
        return "4g"

    def sync_network_context_from_manager(self) -> None:
        """Refresh AppContext network state from the current modem snapshot."""
        if self.app.context is None or self.app.network_manager is None:
            return

        state = self.app.network_manager.modem_state
        signal_bars = state.signal.bars if state.signal is not None else 0
        self.app.context.update_network_status(
            network_enabled=self.app.network_manager.config.enabled,
            signal_bars=signal_bars,
            connection_type=self.cellular_connection_type(),
            connected=self.app.network_manager.is_online,
            gps_has_fix=state.gps is not None,
        )

    def handle_network_ppp_up(self, event: NetworkPppUpEvent) -> None:
        """Refresh network connectivity state when PPP comes online."""
        if self.app.cloud_manager is not None:
            self.app.cloud_manager.note_network_change(connected=True)
        if self.app.network_manager is not None:
            self.sync_network_context_from_manager()
            return
        if self.app.context is not None:
            self.app.context.update_network_status(
                network_enabled=True,
                connected=True,
                connection_type=event.connection_type,
            )

    def handle_network_signal_update(self, event: NetworkSignalUpdateEvent) -> None:
        """Refresh signal bars when the modem reports new telemetry."""
        if self.app.network_manager is not None:
            self.sync_network_context_from_manager()
            return
        if self.app.context is not None:
            connection_type = self.app.context.connection_type
            if connection_type == "none":
                connection_type = "4g"
            self.app.context.update_network_status(
                network_enabled=True,
                signal_bars=event.bars,
                connection_type=connection_type,
            )

    def handle_network_gps_fix(self, event: NetworkGpsFixEvent) -> None:
        """Update GPS fix state in AppContext."""
        if self.app.network_manager is not None:
            self.sync_network_context_from_manager()
            return
        if self.app.context is not None:
            connection_type = self.app.context.connection_type
            if connection_type == "none":
                connection_type = "4g"
            self.app.context.update_network_status(
                network_enabled=True,
                connection_type=connection_type,
                gps_has_fix=True,
            )

    def handle_network_gps_no_fix(self, _event: NetworkGpsNoFixEvent) -> None:
        """Clear GPS fix state when a query completes without coordinates."""
        if self.app.network_manager is not None:
            self.sync_network_context_from_manager()
            return
        if self.app.context is not None:
            self.app.context.update_network_status(gps_has_fix=False)

    def handle_network_ppp_down(self, _event: NetworkPppDownEvent) -> None:
        """Reset network state in AppContext when PPP drops."""
        if self.app.cloud_manager is not None:
            self.app.cloud_manager.note_network_change(connected=False)
        if self.app.network_manager is not None:
            self.sync_network_context_from_manager()
            return
        if self.app.context is not None:
            self.app.context.update_network_status(
                network_enabled=True,
                connected=False,
                gps_has_fix=False,
            )
