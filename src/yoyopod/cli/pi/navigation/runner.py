"""Navigation soak runner orchestration."""

from __future__ import annotations

import os
import time
from contextlib import contextmanager
from pathlib import Path
from typing import TYPE_CHECKING, Iterator

from loguru import logger

from yoyopod.cli.pi.music_fixtures import provision_test_music_library
from yoyopod.cli.pi.navigation.exercises import _NavigationExercises
from yoyopod.cli.pi.navigation.pump import _RuntimePump
from yoyopod.cli.pi.navigation.stats import NavigationSoakFailure, NavigationSoakStats
from yoyopod.ui.input import InteractionProfile

if TYPE_CHECKING:
    from yoyopod.app import YoyoPodApp


@contextmanager
def _temporary_env_var(name: str, value: str | None) -> Iterator[None]:
    """Temporarily override one environment variable."""

    previous = os.environ.get(name)
    if value is None:
        yield
        return

    os.environ[name] = value
    try:
        yield
    finally:
        if previous is None:
            os.environ.pop(name, None)
        else:
            os.environ[name] = previous


class NavigationSoakRunner:
    """Exercise the target one-button UI with repeatable action-driven flows."""

    def __init__(
        self,
        *,
        config_dir: str,
        cycles: int,
        hold_seconds: float,
        idle_seconds: float,
        tail_idle_seconds: float,
        with_playback: bool,
        provision_test_music: bool,
        test_music_dir: str,
        skip_sleep: bool,
    ) -> None:
        self.config_dir = config_dir
        self.cycles = max(1, cycles)
        self.hold_seconds = max(0.05, hold_seconds)
        self.idle_seconds = max(0.0, idle_seconds)
        self.tail_idle_seconds = max(0.0, tail_idle_seconds)
        self.with_playback = with_playback
        self.provision_test_music = provision_test_music
        self.test_music_dir = test_music_dir
        self.skip_sleep = skip_sleep

        self.stats = NavigationSoakStats()
        self._app: YoyoPodApp | None = None
        self._pump: _RuntimePump | None = None
        self._exercises = _NavigationExercises(self)

    def run(self) -> tuple[bool, str]:
        """Run the full soak and return success plus one-line details."""

        from yoyopod.app import YoyoPodApp

        music_dir_override = None
        if self.with_playback and self.provision_test_music:
            library = provision_test_music_library(Path(self.test_music_dir))
            music_dir_override = str(library.target_dir)
            logger.info(
                "Navigation soak provisioned validation music at {}",
                music_dir_override,
            )

        with _temporary_env_var("YOYOPOD_MUSIC_DIR", music_dir_override):
            app = YoyoPodApp(config_dir=self.config_dir, simulate=False)
            self._app = app
            if not app.setup():
                try:
                    app.stop()
                except Exception:
                    logger.exception("Navigation soak cleanup failed after unsuccessful app setup")
                return False, "app setup failed"

            try:
                if app.display is None or app.screen_manager is None or app.input_manager is None:
                    return False, "display, screen manager, or input manager not initialized"
                if app.display.backend_kind != "lvgl":
                    return False, f"backend is {app.display.backend_kind}, expected lvgl"
                if app.input_manager.interaction_profile != InteractionProfile.ONE_BUTTON:
                    profile_name = app.input_manager.interaction_profile.value
                    return False, f"profile is {profile_name}, expected one_button"

                app.power_runtime.start_watchdog(now=time.monotonic())
                self._pump = _RuntimePump(app, self.stats)
                self._pump.run_for(self.hold_seconds)

                self._exercises.require_screen("hub")
                for cycle_number in range(1, self.cycles + 1):
                    logger.info(
                        "Navigation soak cycle {}/{}",
                        cycle_number,
                        self.cycles,
                    )
                    self._exercises.exercise_cycle()

                self._exercises.idle_phase("hub_tail_idle", self.tail_idle_seconds)
                self._exercises.exercise_sleep_wake()
            except NavigationSoakFailure as exc:
                return False, str(exc)
            finally:
                app.stop()

        return True, self._summary_details()

    @property
    def app(self) -> "YoyoPodApp":
        """Return the active app or raise when the runner is uninitialized."""

        if self._app is None:
            raise NavigationSoakFailure("navigation soak app is not initialized")
        return self._app

    @property
    def pump(self) -> _RuntimePump:
        """Return the runtime pump or raise when the runner is uninitialized."""

        if self._pump is None:
            raise NavigationSoakFailure("navigation soak runtime pump is not initialized")
        return self._pump

    def _summary_details(self) -> str:
        """Return a compact summary string for CLI output."""

        blocking_span = "none"
        if self.stats.heaviest_blocking_span_name is not None:
            blocking_span = (
                f"{self.stats.heaviest_blocking_span_name}:"
                f"{self.stats.heaviest_blocking_span_ms:.1f}ms"
            )

        screen_list = ",".join(sorted(self.stats.visited_screens)) or "none"
        playback_state = "verified" if self.stats.playback_verified else "not_requested"
        if self.with_playback and not self.stats.playback_verified:
            playback_state = "requested_not_verified"

        track_name = self.stats.last_track_name or "none"
        return (
            f"profile=one_button, cycles={self.cycles}, actions={self.stats.actions}, "
            f"screens={screen_list}, explicit_idle_s={self.stats.explicit_idle_seconds:.1f}, "
            f"playback={playback_state}, last_track={track_name}, "
            f"max_iteration_ms={self.stats.max_runtime_iteration_ms:.1f}, "
            f"max_loop_gap_ms={self.stats.max_runtime_loop_gap_ms:.1f}, "
            f"max_voip_delay_ms={self.stats.max_voip_schedule_delay_ms:.1f}, "
            f"heaviest_span={blocking_span}, sleep_wake={self.stats.sleep_wake_status}"
        )
