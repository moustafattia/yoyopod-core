"""
Package entry point for YoyoPod.

This module provides the console-script target used by `pyproject.toml`
and keeps the installed entry point aligned with the top-level launcher.
"""

import sys

from loguru import logger

from yoyopy.app import YoyoPodApp
from yoyopy.utils.logger import init_logger


def configure_logger() -> None:
    """Configure the default console logger for the app."""
    init_logger(
        level="INFO",
        console=True,
        file_logging=False,
        console_stream=sys.stderr,
        announce=False,
    )


def main() -> int:
    """Run the integrated YoyoPod application."""
    configure_logger()

    simulate = "--simulate" in sys.argv

    if simulate:
        logger.info("=" * 60)
        logger.info("SIMULATION MODE")
        logger.info("Running without physical hardware")
        logger.info("Web server will start on http://localhost:5000")
        logger.info("Open the URL in your browser to view the display")
        logger.info("Use keyboard (Arrow keys, Enter, Esc) or web buttons for input")
        logger.info("=" * 60)

    logger.info("Initializing YoyoPod...")
    app = YoyoPodApp(config_dir="config", simulate=simulate)

    if not app.setup():
        logger.error("Failed to setup application!")
        logger.error("Check that:")
        logger.error("  - config/voip_config.yaml exists")
        logger.error("  - config/contacts.yaml exists")
        logger.error("  - linphonec is installed")
        logger.error("  - Mopidy is running on localhost:6680")
        return 1

    logger.info("")
    logger.info("=" * 60)
    logger.info("YoyoPod Ready!")
    logger.info("=" * 60)
    logger.info("")
    logger.info("Available Features:")
    logger.info("  - Music streaming (Mopidy/Spotify)")
    logger.info("  - VoIP calling (linphonec)")
    logger.info("  - Auto-pause music on calls")
    logger.info("  - Auto-resume after calls")
    logger.info("  - Full UI navigation")
    logger.info("")
    logger.info("Current Configuration:")
    status = app.get_status()
    logger.info(f"  Auto-resume: {status['auto_resume']}")
    logger.info(f"  VoIP available: {status['voip_available']}")
    logger.info(f"  Music available: {status['music_available']}")
    logger.info("")
    logger.info("Press Ctrl+C to exit")
    logger.info("=" * 60)
    logger.info("")

    try:
        app.run()
    except KeyboardInterrupt:
        logger.info("")
        logger.info("=" * 60)
        logger.info("Shutting down...")
    finally:
        app.stop()

    logger.info("")
    logger.info("Goodbye!")
    return 0
