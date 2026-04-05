#!/usr/bin/env python3
"""Run a small hardware-in-the-loop LVGL soak pass on the Whisplay path."""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

from loguru import logger

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from yoyopy.app import YoyoPodApp
from yoyopy.events import UserActivityEvent


def configure_logging(verbose: bool) -> None:
    """Configure human-readable logging."""

    logger.remove()
    logger.add(
        sys.stderr,
        format="<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | <level>{message}</level>",
        level="DEBUG" if verbose else "INFO",
    )


def _pump_app(app: YoyoPodApp, duration_seconds: float) -> None:
    """Pump the coordinator-thread services without entering the full app run loop."""

    deadline = time.monotonic() + max(0.0, duration_seconds)
    while time.monotonic() < deadline:
        app._process_pending_main_thread_actions()
        now = time.monotonic()
        app._attempt_manager_recovery()
        app._poll_power_status(now=now)
        app._pump_lvgl_backend(now)
        app._feed_watchdog_if_due(now)
        app._update_screen_power(now)
        time.sleep(0.05)


def _exercise_sleep_wake(app: YoyoPodApp) -> tuple[bool, str]:
    """Force one sleep/wake cycle against the current app."""

    timeout_seconds = max(1.0, float(app._screen_timeout_seconds or 0.0))
    app._last_user_activity_at = time.monotonic() - timeout_seconds - 1.0
    _pump_app(app, 0.35)
    if app.context is None or app.context.screen_awake:
        return False, "screen did not enter sleep during soak"

    app.event_bus.publish(UserActivityEvent(source="lvgl_soak"))
    _pump_app(app, 0.35)
    if app.context is None or not app.context.screen_awake:
        return False, "screen did not wake after simulated activity"

    return True, "sleep/wake ok"


def run_lvgl_soak(
    *,
    config_dir: str = "config",
    simulate: bool = False,
    cycles: int = 2,
    hold_seconds: float = 0.2,
    exercise_sleep: bool = True,
) -> tuple[bool, str]:
    """Run a deterministic screen-transition soak and return success/details."""

    app = YoyoPodApp(config_dir=config_dir, simulate=simulate)
    if not app.setup():
        return False, "app setup failed"

    try:
        if app.display is None or app.screen_manager is None:
            return False, "display or screen manager not initialized"

        if app.display.backend_kind != "lvgl":
            return False, f"backend is {app.display.backend_kind}, expected lvgl"

        screens = [
            "hub",
            "listen",
            "playlists",
            "now_playing",
            "call",
            "call_history",
            "contacts",
            "voice_note_contacts",
            "voice_note",
            "ask",
            "power",
        ]

        transitions = 0
        for _cycle in range(max(1, cycles)):
            for screen_name in screens:
                if screen_name not in app.screen_manager.screens:
                    continue
                app.screen_manager.replace_screen(screen_name)
                _pump_app(app, hold_seconds)
                transitions += 1

        sleep_details = "sleep/wake skipped"
        if exercise_sleep:
            sleep_ok, sleep_details = _exercise_sleep_wake(app)
            if not sleep_ok:
                return False, sleep_details

        return True, f"backend=lvgl, transitions={transitions}, {sleep_details}"
    finally:
        app.stop()


def build_parser() -> argparse.ArgumentParser:
    """Create the command-line parser."""

    parser = argparse.ArgumentParser(
        description="Run a small LVGL screen-transition soak pass against YoyoPod."
    )
    parser.add_argument(
        "--config-dir",
        default="config",
        help="Configuration directory to use (default: config)",
    )
    parser.add_argument(
        "--simulate",
        action="store_true",
        help="Run against simulation instead of hardware",
    )
    parser.add_argument(
        "--cycles",
        type=int,
        default=2,
        help="How many full transition cycles to run (default: 2)",
    )
    parser.add_argument(
        "--hold-seconds",
        type=float,
        default=0.2,
        help="How long to keep each screen active during the soak (default: 0.2)",
    )
    parser.add_argument(
        "--skip-sleep",
        action="store_true",
        help="Skip the sleep/wake exercise",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable DEBUG logging",
    )
    return parser


def main() -> int:
    """CLI entry point."""

    parser = build_parser()
    args = parser.parse_args()
    configure_logging(args.verbose)

    ok, details = run_lvgl_soak(
        config_dir=args.config_dir,
        simulate=args.simulate,
        cycles=args.cycles,
        hold_seconds=args.hold_seconds,
        exercise_sleep=not args.skip_sleep,
    )
    if ok:
        logger.info(f"LVGL soak passed: {details}")
        return 0

    logger.error(f"LVGL soak failed: {details}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
