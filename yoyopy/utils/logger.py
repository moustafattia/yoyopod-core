"""
Logging utility for YoyoPod using Loguru.

Provides structured logging with both file and console output,
colorized messages, and automatic rotation.
"""

import sys
from pathlib import Path
from typing import Optional, TextIO
from loguru import logger


def init_logger(
    level: str = "INFO",
    log_dir: Optional[Path] = None,
    console: bool = True,
    file_logging: bool = True,
    console_stream: Optional[TextIO] = None,
    rotation: str = "100 MB",
    retention: str = "10 days",
    announce: bool = True,
) -> None:
    """
    Initialize the loguru logger with custom settings.

    Args:
        level: Logging level (TRACE, DEBUG, INFO, SUCCESS, WARNING, ERROR, CRITICAL)
        log_dir: Directory for log files (defaults to ./logs)
        console: Enable console output
        file_logging: Enable file output
        console_stream: Stream for console output (defaults to sys.stdout)
        rotation: When to rotate log files (size or time-based)
        retention: How long to keep old log files
        announce: Log a one-line initialization message after setup
    """
    # Remove default handler
    logger.remove()

    # Set up log directory
    if log_dir is None:
        log_dir = Path.cwd() / "logs"

    if file_logging:
        log_dir.mkdir(parents=True, exist_ok=True)

    # Console handler with colors
    if console:
        if console_stream is None:
            console_stream = sys.stdout
        logger.add(
            console_stream,
            format="<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | <level>{message}</level>",
            level=level,
            colorize=True,
        )

    # File handler with detailed format and rotation
    if file_logging:
        log_file = log_dir / "yoyopy_{time:YYYY-MM-DD}.log"
        logger.add(
            log_file,
            format="{time:YYYY-MM-DD HH:mm:ss} | {level: <8} | {name}:{function}:{line} | {message}",
            level="DEBUG",  # Always capture all logs to file
            rotation=rotation,
            retention=retention,
            compression="zip",  # Compress old logs
            enqueue=True,  # Thread-safe logging
        )

    if announce:
        logger.info(f"Logger initialized with level: {level}")


def get_logger():
    """
    Get the loguru logger instance.

    Returns:
        The loguru logger instance
    """
    return logger
