"""Boot-time composition helpers for the runtime layer."""

from __future__ import annotations

import time
from typing import TYPE_CHECKING

from loguru import logger

from yoyopod.audio import (
    LocalMusicService,
    MpvBackend,
    MusicConfig,
    OutputVolumeController,
    RecentTrackHistoryStore,
)
from yoyopod.config import ConfigManager
from yoyopod.coordinators import (
    AppRuntimeState,
    CallCoordinator,
    CoordinatorRuntime,
    PlaybackCoordinator,
    PowerCoordinator,
    ScreenCoordinator,
)
from yoyopod.device import AudioDeviceCatalog
from yoyopod.core import ScreenChangedEvent
from yoyopod.integrations.call import CallHistoryStore, VoIPConfig, VoIPManager
from yoyopod.integrations.cloud.manager import CloudManager
from yoyopod.integrations.contacts.directory import PeopleManager
from yoyopod.integrations.network import NetworkManager
from yoyopod.integrations.power import PowerManager
from yoyopod.ui.display import Display
from yoyopod.ui.display.contracts import (
    WhisplayProductionRenderContractError,
    build_whisplay_production_contract_message,
)
from yoyopod.ui.input import InteractionProfile, get_input_manager
from yoyopod.ui.lvgl_binding import LvglInputBridge
from yoyopod.ui.screens.manager import ScreenManager

from .components_boot import ComponentsBoot
from .config_boot import ConfigBoot
from .managers_boot import ManagersBoot
from .screens_boot import ScreensBoot

if TYPE_CHECKING:
    from yoyopod.app import YoyoPodApp


class RuntimeBootService:
    """Own the boot-time composition of the application runtime."""

    def __init__(self, app: "YoyoPodApp") -> None:
        self.app = app
        self._config_boot = ConfigBoot(
            app,
            logger=logger,
            config_manager_cls=ConfigManager,
            people_manager_cls=PeopleManager,
            call_history_store_cls=CallHistoryStore,
            recent_track_history_store_cls=RecentTrackHistoryStore,
            audio_device_catalog_cls=AudioDeviceCatalog,
        )
        self._components_boot = ComponentsBoot(
            app,
            logger=logger,
            display_cls=Display,
            get_input_manager_fn=get_input_manager,
            screen_manager_cls=ScreenManager,
            lvgl_input_bridge_cls=LvglInputBridge,
            contract_error_cls=WhisplayProductionRenderContractError,
            build_contract_message_fn=build_whisplay_production_contract_message,
        )
        self._managers_boot = ManagersBoot(
            app,
            logger=logger,
            voip_config_cls=VoIPConfig,
            voip_manager_cls=VoIPManager,
            music_config_cls=MusicConfig,
            mpv_backend_cls=MpvBackend,
            local_music_service_cls=LocalMusicService,
            output_volume_controller_cls=OutputVolumeController,
            power_manager_cls=PowerManager,
            network_manager_cls=NetworkManager,
            cloud_manager_cls=CloudManager,
        )
        self._screens_boot = ScreensBoot(app, logger=logger)

    def setup(self) -> bool:
        """Initialize all components and register callbacks."""
        try:
            if not self.load_configuration():
                logger.error("Failed to load configuration")
                return False

            if not self.init_core_components():
                logger.error("Failed to initialize core components")
                return False

            if not self.init_managers():
                logger.error("Failed to initialize managers")
                return False

            if not self.setup_screens():
                logger.error("Failed to setup screens")
                return False

            self.ensure_coordinators()
            self.bind_coordinator_events()
            self.setup_voip_callbacks()
            self.setup_music_callbacks()
            self.app.shutdown_service.register_power_shutdown_hooks()
            self.app.power_runtime.poll_status(force=True, now=time.monotonic())

            logger.info("YoyoPod setup complete")
            return True
        except Exception:
            logger.exception("Setup failed")
            return False

    def load_configuration(self) -> bool:
        return self._config_boot.load_configuration()

    def resolve_screen_timeout_seconds(self) -> float:
        return self.app.screen_power_service.resolve_screen_timeout_seconds()

    def resolve_active_brightness(self) -> float:
        return self.app.screen_power_service.resolve_active_brightness()

    def init_core_components(self) -> bool:
        return self._components_boot.init_core_components()

    def init_managers(self) -> bool:
        return self._managers_boot.init_managers()

    def setup_screens(self) -> bool:
        return self._screens_boot.setup_screens()

    def get_interaction_profile(self) -> InteractionProfile:
        return self._screens_boot.get_interaction_profile()

    def get_initial_screen_name(self) -> str:
        return self._screens_boot.get_initial_screen_name()

    def setup_voip_callbacks(self) -> None:
        """Register VoIP event callbacks."""

        logger.info("Setting up VoIP callbacks...")

        if not self.app.voip_manager:
            logger.warning("  VoIPManager not available, skipping callbacks")
            return

        call_coordinator = self.app.call_coordinator
        if call_coordinator is None:
            logger.warning("  CallCoordinator not available, skipping VoIP callbacks")
            return

        self.app.voip_manager.on_incoming_call(call_coordinator.handle_incoming_call)
        self.app.voip_manager.on_call_state_change(
            call_coordinator.handle_call_state_change
        )
        self.app.voip_manager.on_registration_change(
            call_coordinator.handle_registration_change
        )
        self.app.voip_manager.on_availability_change(
            call_coordinator.handle_availability_change
        )
        self.app.voip_manager.on_message_summary_change(
            self.app.voice_note_events.handle_voice_note_summary_changed
        )
        self.app.voip_manager.on_message_received(
            self.app.voice_note_events.handle_voice_note_activity_changed
        )
        self.app.voip_manager.on_message_delivery_change(
            self.app.voice_note_events.handle_voice_note_activity_changed
        )
        self.app.voip_manager.on_message_failure(
            self.app.voice_note_events.handle_voice_note_failure
        )
        self.app.voice_note_events.sync_talk_summary_context()
        self.app.voice_note_events.sync_active_voice_note_context()
        logger.info("  VoIP callbacks registered")

    def setup_music_callbacks(self) -> None:
        """Register music event callbacks."""

        logger.info("Setting up music callbacks...")

        if not self.app.music_backend:
            logger.warning("  MusicBackend not available, skipping callbacks")
            return

        playback_coordinator = self.app.playback_coordinator
        if playback_coordinator is None:
            logger.warning("  PlaybackCoordinator not available, skipping music callbacks")
            return

        self.app.music_backend.on_track_change(playback_coordinator.handle_track_change)
        self.app.music_backend.on_playback_state_change(
            playback_coordinator.handle_playback_state_change
        )
        if self.app.audio_volume_controller is not None:
            self.app.music_backend.on_connection_change(
                self.app.audio_volume_controller.sync_output_volume_on_music_connect
            )
        self.app.music_backend.on_connection_change(
            playback_coordinator.handle_availability_change
        )
        logger.info("  Music callbacks registered")

    def bind_coordinator_events(self) -> None:
        """Bind coordinator-level event handlers to the EventBus."""

        logger.info("Setting up event subscriptions...")
        call_coordinator = self.app.call_coordinator
        playback_coordinator = self.app.playback_coordinator
        power_coordinator = self.app.power_coordinator
        if (
            call_coordinator is None
            or playback_coordinator is None
            or power_coordinator is None
        ):
            logger.warning("  Coordinators not available, skipping event subscriptions")
            return

        call_coordinator.bind(self.app.event_bus)
        playback_coordinator.bind(self.app.event_bus)
        power_coordinator.bind(self.app.event_bus)
        logger.info("  Event subscriptions registered")

    def setup_event_subscriptions(self) -> None:
        """Backward-compatible alias for coordinator event binding."""
        self.ensure_coordinators()
        self.bind_coordinator_events()

    def ensure_coordinators(self) -> None:
        """Build coordinator helpers around the initialized runtime."""

        if self.app.coordinator_runtime is not None:
            return

        assert self.app.music_fsm is not None
        assert self.app.call_fsm is not None
        assert self.app.call_interruption_policy is not None
        assert self.app.context is not None
        current_screen = (
            self.app.screen_manager.get_current_screen()
            if self.app.screen_manager is not None
            else None
        )
        current_route_name = current_screen.route_name if current_screen is not None else None
        initial_ui_state = (
            CoordinatorRuntime.ui_state_for_screen_name(current_route_name)
            or AppRuntimeState.IDLE
        )
        self.app.coordinator_runtime = CoordinatorRuntime(
            music_fsm=self.app.music_fsm,
            call_fsm=self.app.call_fsm,
            call_interruption_policy=self.app.call_interruption_policy,
            screen_manager=self.app.screen_manager,
            music_backend=self.app.music_backend,
            power_manager=self.app.power_manager,
            now_playing_screen=self.app.now_playing_screen,
            call_screen=self.app.call_screen,
            power_screen=self.app.power_screen,
            incoming_call_screen=self.app.incoming_call_screen,
            outgoing_call_screen=self.app.outgoing_call_screen,
            in_call_screen=self.app.in_call_screen,
            config_manager=self.app.config_manager,
            context=self.app.context,
            ui_state=initial_ui_state,
            voip_ready=self.app._voip_registered,
        )
        self.app.screen_coordinator = ScreenCoordinator(self.app.coordinator_runtime)
        self.app.call_coordinator = CallCoordinator(
            runtime=self.app.coordinator_runtime,
            screen_coordinator=self.app.screen_coordinator,
            auto_resume_after_call=self.app.auto_resume_after_call,
            call_history_store=self.app.call_history_store,
            initial_voip_registered=self.app._voip_registered,
        )
        self.app.playback_coordinator = PlaybackCoordinator(
            runtime=self.app.coordinator_runtime,
            screen_coordinator=self.app.screen_coordinator,
            local_music_service=self.app.local_music_service,
        )
        self.app.power_coordinator = PowerCoordinator(
            runtime=self.app.coordinator_runtime,
            screen_coordinator=self.app.screen_coordinator,
            context=self.app.context,
            cloud_manager=self.app.cloud_manager,
        )
        if self.app.screen_manager is not None:
            self.app.screen_manager.on_screen_changed = (
                lambda screen_name: self.app.event_bus.publish(
                    ScreenChangedEvent(screen_name=screen_name)
                )
            )
            self.app.event_bus.publish(ScreenChangedEvent(screen_name=current_route_name))


__all__ = ["RuntimeBootService"]
