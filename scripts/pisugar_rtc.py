#!/usr/bin/env python3
"""PiSugar RTC helper for status, sync, and alarm operations."""

from __future__ import annotations

import argparse
import sys
from datetime import datetime
from pathlib import Path

from loguru import logger

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from yoyopy.config import ConfigManager
from yoyopy.power import PowerManager, RTCState


def configure_logging(verbose: bool) -> None:
    """Configure human-readable logging."""
    logger.remove()
    logger.add(
        sys.stderr,
        format="<green>{time:HH:mm:ss}</green> | <level>{level: <8}</level> | <level>{message}</level>",
        level="DEBUG" if verbose else "INFO",
    )


def build_parser() -> argparse.ArgumentParser:
    """Create the command-line parser."""
    parser = argparse.ArgumentParser(
        description="Inspect and control PiSugar RTC state through YoyoPod's power module.",
    )
    parser.add_argument(
        "--config-dir",
        default="config",
        help="Configuration directory to use (default: config)",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable DEBUG logging",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("status", help="Show current RTC and alarm state")
    subparsers.add_parser("sync-to-rtc", help="Sync Raspberry Pi system time to the PiSugar RTC")
    subparsers.add_parser("sync-from-rtc", help="Sync PiSugar RTC time to the Raspberry Pi system clock")

    set_alarm = subparsers.add_parser("set-alarm", help="Set the PiSugar RTC wake alarm")
    set_alarm.add_argument(
        "--time",
        required=True,
        help="Alarm time as an ISO8601 timestamp, e.g. 2026-04-06T07:30:00+02:00",
    )
    set_alarm.add_argument(
        "--repeat-mask",
        type=int,
        default=127,
        help="Weekday repeat bitmask (default: 127 for every day)",
    )

    subparsers.add_parser("disable-alarm", help="Disable the PiSugar RTC wake alarm")
    return parser


def format_rtc_state(state: RTCState) -> list[str]:
    """Return a human-readable summary of RTC state."""
    lines = [
        f"rtc_time={state.time.isoformat() if state.time is not None else 'unknown'}",
        f"alarm_enabled={state.alarm_enabled}",
        f"alarm_time={state.alarm_time.isoformat() if state.alarm_time is not None else 'none'}",
        f"alarm_repeat_mask={state.alarm_repeat_mask if state.alarm_repeat_mask is not None else 'unknown'}",
        f"adjust_ppm={state.adjust_ppm if state.adjust_ppm is not None else 'unknown'}",
    ]
    return lines


def build_manager(config_dir: Path) -> PowerManager:
    """Create a power manager for the configured PiSugar backend."""
    config_manager = ConfigManager(config_dir=str(config_dir))
    manager = PowerManager.from_config_manager(config_manager)
    if not manager.config.enabled:
        raise RuntimeError("power backend disabled in yoyopod_config.yaml")
    return manager


def parse_alarm_time(value: str) -> datetime:
    """Parse an ISO8601 alarm timestamp."""
    normalized = value.strip()
    if normalized.endswith("Z"):
        normalized = normalized[:-1] + "+00:00"
    return datetime.fromisoformat(normalized)


def print_rtc_status(manager: PowerManager, heading: str) -> int:
    """Refresh and print the current RTC status."""
    snapshot = manager.refresh()
    if not snapshot.available:
        logger.error(snapshot.error or "power backend unavailable")
        return 1

    print("")
    print(heading)
    print("=" * len(heading))
    print(f"model={snapshot.device.model or 'unknown'}")
    for line in format_rtc_state(snapshot.rtc):
        print(line)
    return 0


def main() -> int:
    """Program entry point."""
    parser = build_parser()
    args = parser.parse_args()
    configure_logging(args.verbose)

    config_dir = Path(args.config_dir)
    if not config_dir.is_absolute():
        config_dir = REPO_ROOT / config_dir

    manager = build_manager(config_dir)

    if args.command == "status":
        return print_rtc_status(manager, "PiSugar RTC status")

    if args.command == "sync-to-rtc":
        state = manager.sync_time_to_rtc()
        print("")
        print("PiSugar RTC synced from Raspberry Pi system time")
        print("==============================================")
        for line in format_rtc_state(state):
            print(line)
        return 0

    if args.command == "sync-from-rtc":
        state = manager.sync_time_from_rtc()
        print("")
        print("Raspberry Pi system time synced from PiSugar RTC")
        print("===============================================")
        for line in format_rtc_state(state):
            print(line)
        return 0

    if args.command == "set-alarm":
        state = manager.set_rtc_alarm(
            parse_alarm_time(args.time),
            repeat_mask=args.repeat_mask,
        )
        print("")
        print("PiSugar RTC alarm updated")
        print("=========================")
        for line in format_rtc_state(state):
            print(line)
        return 0

    if args.command == "disable-alarm":
        state = manager.disable_rtc_alarm()
        print("")
        print("PiSugar RTC alarm disabled")
        print("==========================")
        for line in format_rtc_state(state):
            print(line)
        return 0

    parser.error(f"Unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
