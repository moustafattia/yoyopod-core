"""
Package entry point for YoyoPod.

This module provides the console-script target used by `pyproject.toml`
and keeps the installed entry point aligned with the top-level launcher.
"""

import os
import sys
import signal
from pathlib import Path

from loguru import logger

from yoyopy import __version__
from yoyopy.app import YoyoPodApp
from yoyopy.config.models import YoyoPodConfig, load_config_model_from_yaml
from yoyopy.utils.logger import (
    LoggingRuntimeConfig,
    build_logging_runtime_config,
    get_subsystem_logger,
    init_logger,
    log_shutdown,
    log_startup,
    remove_pid_file,
    write_pid_file,
)


def load_app_settings(config_dir: str = "config") -> YoyoPodConfig:
    """Load app settings early enough to configure logging before app setup."""

    config_path = Path(config_dir) / "yoyopod_config.yaml"
    try:
        return load_config_model_from_yaml(YoyoPodConfig, config_path)
    except Exception:
        return YoyoPodConfig()


def configure_logger(config_dir: str = "config") -> LoggingRuntimeConfig:
    """Configure the shared logger using typed logging settings."""

    settings = load_app_settings(config_dir)
    logging_config = build_logging_runtime_config(
        settings.logging,
        base_dir=Path.cwd(),
    )
    return init_logger(
        config=logging_config,
        console=True,
        file_logging=True,
        console_stream=sys.stderr,
        announce=False,
    )


def main() -> int:
    """Run the integrated YoyoPod application."""
    logging_config = configure_logger()
    app_log = get_subsystem_logger("app")

    simulate = "--simulate" in sys.argv
    pid = os.getpid()
    startup_logged = False

    write_pid_file(logging_config.pid_file, pid)

    try:
        log_startup(version=__version__, pid=pid, runtime=logging_config)
        startup_logged = True

        if simulate:
            app_log.info("=" * 60)
            app_log.info("SIMULATION MODE")
            app_log.info("Running without physical hardware")
            app_log.info("Web server will start on http://localhost:5000")
            app_log.info("Open the URL in your browser to view the display")
            app_log.info("Use keyboard (Arrow keys, Enter, Esc) or web buttons for input")
            app_log.info("=" * 60)

        app_log.info("Initializing YoyoPod...")
        app = YoyoPodApp(config_dir="config", simulate=simulate)

        if not app.setup():
            app_log.error("Failed to setup application")
            app_log.error("Check that:")
            app_log.error("  - config/voip_config.yaml exists")
            app_log.error("  - config/contacts.yaml exists")
            app_log.error("  - linphonec is installed")
            app_log.error("  - Mopidy is running on localhost:6680")
            app.stop()
            return 1

        app_log.info("")
        app_log.info("=" * 60)
        app_log.info("YoyoPod Ready!")
        app_log.info("=" * 60)
        app_log.info("")
        app_log.info("Available Features:")
        app_log.info("  - Music streaming (Mopidy/Spotify)")
        app_log.info("  - VoIP calling (linphonec)")
        app_log.info("  - Auto-pause music on calls")
        app_log.info("  - Auto-resume after calls")
        app_log.info("  - Full UI navigation")
        app_log.info("")
        app_log.info("Current Configuration:")
        status = app.get_status()
        app_log.info(f"  Auto-resume: {status['auto_resume']}")
        app_log.info(f"  VoIP available: {status['voip_available']}")
        app_log.info(f"  Music available: {status['music_available']}")
        app_log.info("")
        app_log.info("Press Ctrl+C to exit")
        app_log.info("=" * 60)
        app_log.info("")

        def handle_shutdown_signal(signum, _frame) -> None:
            signal_name = signal.Signals(signum).name
            app_log.info(f"Received {signal_name}, shutting down...")
            raise KeyboardInterrupt

        previous_sigterm = signal.getsignal(signal.SIGTERM)
        signal.signal(signal.SIGTERM, handle_shutdown_signal)

        # Screenshot signal handlers (Unix only — SIGUSR1/SIGUSR2 unavailable on Windows).
        screenshot_path = "/tmp/yoyopod_screenshot.png"
        if hasattr(signal, "SIGUSR1"):

            def handle_screenshot_shadow(_signum: int, _frame: object) -> None:
                """SIGUSR1: save shadow buffer screenshot."""
                display = getattr(app, "display", None)
                adapter = getattr(display, "_adapter", None) if display else None
                save_fn = getattr(adapter, "save_screenshot", None)
                if save_fn:
                    save_fn(screenshot_path)
                else:
                    app_log.warning("Screenshot not available — no save_screenshot method")

            def handle_screenshot_readback(_signum: int, _frame: object) -> None:
                """SIGUSR2: save LVGL readback screenshot."""
                display = getattr(app, "display", None)
                adapter = getattr(display, "_adapter", None) if display else None
                save_fn = getattr(adapter, "save_screenshot_readback", None)
                if save_fn:
                    save_fn(screenshot_path)
                else:
                    app_log.warning("LVGL readback not available — no save_screenshot_readback method")

            signal.signal(signal.SIGUSR1, handle_screenshot_shadow)
            signal.signal(signal.SIGUSR2, handle_screenshot_readback)
            app_log.info(
                "Screenshot handlers installed (SIGUSR1=shadow, SIGUSR2=readback) -> {}",
                screenshot_path,
            )

        exit_code = 0
        try:
            app.run()
        except KeyboardInterrupt:
            app_log.info("Shutdown requested by signal or keyboard interrupt")
        except Exception:
            logger.exception("Unhandled exception escaped the YoyoPod main loop")
            exit_code = 1
        finally:
            signal.signal(signal.SIGTERM, previous_sigterm)
            app.stop()

        app_log.info("Goodbye!")
        return exit_code
    finally:
        if startup_logged:
            log_shutdown(pid=pid)
        remove_pid_file(logging_config.pid_file)
