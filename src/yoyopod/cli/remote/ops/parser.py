"""Legacy argparse parser construction for remote operations."""

from __future__ import annotations

import argparse
import os

from yoyopod.cli.remote.config import DEFAULT_PI_PROJECT_DIR, PiDeployConfig


def build_parser(deploy_config: PiDeployConfig) -> argparse.ArgumentParser:
    """Create the legacy argparse command-line parser (mirrors old scripts/pi_remote.py)."""
    parser = argparse.ArgumentParser(
        description=(
            "Run common YoyoPod Raspberry Pi development tasks over SSH. "
            "Defaults can be provided with YOYOPOD_PI_HOST, "
            "YOYOPOD_PI_USER, YOYOPOD_PI_PROJECT_DIR, and YOYOPOD_PI_BRANCH, "
            "or through deploy/pi-deploy.yaml plus the optional "
            "deploy/pi-deploy.local.yaml override."
        )
    )
    parser.add_argument(
        "--host",
        default=os.getenv("YOYOPOD_PI_HOST", deploy_config.host),
        help="SSH host or alias for the Raspberry Pi",
    )
    parser.add_argument(
        "--user",
        default=os.getenv("YOYOPOD_PI_USER", deploy_config.user),
        help="SSH user for the Raspberry Pi (optional)",
    )
    parser.add_argument(
        "--project-dir",
        default=os.getenv("YOYOPOD_PI_PROJECT_DIR", deploy_config.project_dir),
        help=f"Project directory on the Raspberry Pi (default: {DEFAULT_PI_PROJECT_DIR})",
    )
    parser.add_argument(
        "--branch",
        default=os.getenv("YOYOPOD_PI_BRANCH", deploy_config.branch),
        help="Git branch to sync on the Raspberry Pi (default: main)",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    config_parser = subparsers.add_parser(
        "config",
        help="Show or edit the merged Raspberry Pi deploy config",
    )
    config_parser.add_argument(
        "config_action",
        nargs="?",
        default="show",
        choices=["show", "paths", "init-local", "edit"],
        help="Config action to run locally (default: show)",
    )
    config_parser.add_argument(
        "--editor",
        help="Override the editor command for `config edit`",
    )

    subparsers.add_parser(
        "status",
        help="Show remote repo, music backend, and process status",
    )

    sync_parser = subparsers.add_parser(
        "sync",
        help="Sync the stable Pi checkout to one committed branch or exact commit",
    )
    sync_parser.add_argument(
        "--sha",
        help="Exact commit SHA to check out after syncing the branch",
    )
    sync_parser.add_argument(
        "--skip-uv-sync",
        action="store_true",
        help="Skip `uv sync --extra dev` after syncing the requested revision",
    )

    validate_parser = subparsers.add_parser(
        "validate",
        help="Validate one committed branch/SHA on the Pi and leave the app running",
    )
    validate_parser.add_argument(
        "--sha",
        help="Exact commit SHA to validate (defaults to local HEAD for the current branch)",
    )
    validate_parser.add_argument("--skip-uv-sync", action="store_true")
    validate_parser.add_argument("--with-power", action="store_true")
    validate_parser.add_argument("--with-rtc", action="store_true")
    validate_parser.add_argument("--with-music", action="store_true")
    validate_parser.add_argument("--no-provision-test-music", action="store_true")
    validate_parser.add_argument(
        "--test-music-dir",
        default=deploy_config.test_music_target_dir,
    )
    validate_parser.add_argument("--with-voip", action="store_true")
    validate_parser.add_argument("--with-navigation-soak", action="store_true")
    validate_parser.add_argument("--with-lvgl-soak", action="store_true")
    validate_parser.add_argument("--verbose", action="store_true")
    validate_parser.add_argument("--music-timeout", type=int, default=5)
    validate_parser.add_argument("--voip-timeout", type=float, default=90.0)
    validate_parser.add_argument("--lines", type=int, default=20)

    screenshot_parser = subparsers.add_parser(
        "screenshot",
        help="Capture a screenshot from the running app on the Raspberry Pi",
    )
    screenshot_parser.add_argument(
        "--readback",
        action="store_true",
        help=(
            "Use LVGL readback/default capture (SIGUSR1) instead of the "
            "legacy shadow-first path (SIGUSR2)"
        ),
    )
    screenshot_parser.add_argument(
        "--output",
        default="pi_screenshot.png",
        help="Local path for the downloaded screenshot (default: ./pi_screenshot.png)",
    )

    smoke_parser = subparsers.add_parser(
        "smoke",
        help="Run the Raspberry Pi smoke validator remotely",
    )
    smoke_parser.add_argument("--with-power", action="store_true")
    smoke_parser.add_argument("--with-rtc", action="store_true")
    smoke_parser.add_argument("--with-music", action="store_true")
    smoke_parser.add_argument("--no-provision-test-music", action="store_true")
    smoke_parser.add_argument(
        "--test-music-dir",
        default=deploy_config.test_music_target_dir,
    )
    smoke_parser.add_argument("--with-voip", action="store_true")
    smoke_parser.add_argument("--with-navigation-soak", action="store_true")
    smoke_parser.add_argument("--with-lvgl-soak", action="store_true")
    smoke_parser.add_argument("--verbose", action="store_true")
    smoke_parser.add_argument("--music-timeout", type=int, default=5)
    smoke_parser.add_argument("--voip-timeout", type=float, default=10.0)

    provision_music_parser = subparsers.add_parser(
        "provision-test-music",
        help="Seed the deterministic validation music library on the Raspberry Pi",
    )
    provision_music_parser.add_argument(
        "--target-dir",
        default=deploy_config.test_music_target_dir,
    )
    provision_music_parser.add_argument("--verbose", action="store_true")

    whisplay_parser = subparsers.add_parser(
        "whisplay",
        help="Run the Whisplay gesture-tuning helper remotely",
    )
    whisplay_parser.add_argument("--verbose", action="store_true")
    whisplay_parser.add_argument("--duration-seconds", type=float, default=30.0)
    whisplay_parser.add_argument("--debounce-ms", type=int)
    whisplay_parser.add_argument("--double-tap-ms", type=int)
    whisplay_parser.add_argument("--long-hold-ms", type=int)
    whisplay_parser.add_argument("--no-display", action="store_true")

    lvgl_soak_parser = subparsers.add_parser(
        "lvgl-soak",
        help="Run the LVGL Whisplay soak helper remotely",
    )
    lvgl_soak_parser.add_argument("--cycles", type=int, default=2)
    lvgl_soak_parser.add_argument("--hold-seconds", type=float, default=0.2)
    lvgl_soak_parser.add_argument("--idle-seconds", type=float, default=1.0)
    lvgl_soak_parser.add_argument("--with-music", action="store_true")
    lvgl_soak_parser.add_argument("--no-provision-test-music", action="store_true")
    lvgl_soak_parser.add_argument(
        "--test-music-dir",
        default=deploy_config.test_music_target_dir,
    )
    lvgl_soak_parser.add_argument("--skip-sleep", action="store_true")
    lvgl_soak_parser.add_argument("--verbose", action="store_true")

    navigation_soak_parser = subparsers.add_parser(
        "navigation-soak",
        help="Run the target navigation and idle stability soak remotely",
    )
    navigation_soak_parser.add_argument("--cycles", type=int, default=2)
    navigation_soak_parser.add_argument("--hold-seconds", type=float, default=0.35)
    navigation_soak_parser.add_argument("--idle-seconds", type=float, default=3.0)
    navigation_soak_parser.add_argument("--tail-idle-seconds", type=float, default=10.0)
    navigation_soak_parser.add_argument("--no-with-playback", action="store_true")
    navigation_soak_parser.add_argument("--no-provision-test-music", action="store_true")
    navigation_soak_parser.add_argument(
        "--test-music-dir",
        default=deploy_config.test_music_target_dir,
    )
    navigation_soak_parser.add_argument("--skip-sleep", action="store_true")
    navigation_soak_parser.add_argument("--verbose", action="store_true")

    rtc_parser = subparsers.add_parser(
        "rtc",
        help="Inspect or control PiSugar RTC state remotely",
    )
    rtc_parser.add_argument(
        "rtc_action",
        nargs="?",
        default="status",
        choices=["status", "sync-to", "sync-from", "set-alarm", "disable-alarm"],
    )
    rtc_parser.add_argument("--time")
    rtc_parser.add_argument("--repeat-mask", type=int, default=127)
    rtc_parser.add_argument("--verbose", action="store_true")

    power_parser = subparsers.add_parser(
        "power",
        help="Inspect PiSugar power telemetry remotely",
    )
    power_parser.add_argument("--verbose", action="store_true")

    logs_parser = subparsers.add_parser(
        "logs",
        help="Tail the file-based YoyoPod logs on the Raspberry Pi",
    )
    logs_parser.add_argument("--errors", action="store_true")
    logs_parser.add_argument("--follow", action="store_true")
    logs_parser.add_argument("--filter")
    logs_parser.add_argument("--lines", type=int, default=100)

    service_parser = subparsers.add_parser(
        "service",
        help="Install or inspect the production YoyoPod systemd service",
    )
    service_parser.add_argument(
        "service_action",
        nargs="?",
        default="status",
        choices=["status", "install", "start", "stop", "restart", "logs"],
    )
    service_parser.add_argument("--lines", type=int, default=100)

    rsync_parser = subparsers.add_parser(
        "rsync",
        help="Rare-case escape hatch: mirror the local dirty working tree to the Pi",
    )
    rsync_parser.add_argument("--skip-restart", action="store_true")

    return parser
