"""src/yoyopod/cli/remote/lvgl.py — LVGL soak over SSH."""

from __future__ import annotations

import shlex
from typing import Annotated

import typer

from yoyopod.cli.pi.music_fixtures import DEFAULT_TEST_MUSIC_TARGET_DIR
from yoyopod.cli.remote.ops import (
    _resolve_remote_config,
    run_remote,
    validate_config,
)

# ---------------------------------------------------------------------------
# Command builder
# ---------------------------------------------------------------------------


def build_lvgl_soak_command(
    *,
    cycles: int = 2,
    hold_seconds: float = 0.2,
    idle_seconds: float = 1.0,
    with_music: bool = False,
    provision_test_music: bool = True,
    test_music_dir: str = DEFAULT_TEST_MUSIC_TARGET_DIR,
    skip_sleep: bool = False,
    verbose: bool = False,
) -> str:
    """Create the remote LVGL soak command."""
    parts = ["uv run yoyoctl pi lvgl soak"]
    if verbose:
        parts.append("--verbose")
    if cycles != 2:
        parts.extend(["--cycles", str(cycles)])
    if hold_seconds != 0.2:
        parts.extend(["--hold-seconds", str(hold_seconds)])
    if idle_seconds != 1.0:
        parts.extend(["--idle-seconds", str(idle_seconds)])
    if with_music:
        parts.append("--with-music")
        if not provision_test_music:
            parts.append("--no-provision-test-music")
        elif test_music_dir != DEFAULT_TEST_MUSIC_TARGET_DIR:
            parts.extend(["--test-music-dir", shlex.quote(test_music_dir)])
    if skip_sleep:
        parts.append("--skip-sleep")
    return " ".join(parts)


# ---------------------------------------------------------------------------
# Typer command
# ---------------------------------------------------------------------------


def lvgl_soak(
    host: Annotated[
        str, typer.Option("--host", help="SSH host or alias for the Raspberry Pi.")
    ] = "",
    user: Annotated[
        str, typer.Option("--user", help="SSH user for the Raspberry Pi (optional).")
    ] = "",
    project_dir: Annotated[
        str, typer.Option("--project-dir", help="Project directory on the Raspberry Pi.")
    ] = "",
    branch: Annotated[
        str, typer.Option("--branch", help="Git branch to sync on the Raspberry Pi.")
    ] = "",
    cycles: Annotated[
        int, typer.Option("--cycles", help="How many full transition cycles to run.")
    ] = 2,
    hold_seconds: Annotated[
        float, typer.Option("--hold-seconds", help="How long to keep each screen active.")
    ] = 0.2,
    idle_seconds: Annotated[
        float,
        typer.Option("--idle-seconds", help="How long to idle after each full navigation cycle."),
    ] = 1.0,
    with_music: Annotated[
        bool,
        typer.Option(
            "--with-music",
            help="Exercise playlist loading and now-playing actions during the soak.",
        ),
    ] = False,
    provision_test_music: Annotated[
        bool,
        typer.Option(
            "--provision-test-music/--no-provision-test-music",
            help="Seed deterministic validation music before playback soak steps.",
        ),
    ] = True,
    test_music_dir: Annotated[
        str,
        typer.Option(
            "--test-music-dir",
            help="Dedicated target directory for validation-only test music assets.",
        ),
    ] = DEFAULT_TEST_MUSIC_TARGET_DIR,
    skip_sleep: Annotated[
        bool, typer.Option("--skip-sleep", help="Skip the sleep/wake exercise.")
    ] = False,
    verbose: Annotated[
        bool, typer.Option("--verbose", help="Enable verbose soak logging.")
    ] = False,
) -> None:
    """Run the LVGL Whisplay soak helper remotely."""
    config = _resolve_remote_config(host, user, project_dir, branch)
    validate_config(config)
    rc = run_remote(
        config,
        build_lvgl_soak_command(
            cycles=cycles,
            hold_seconds=hold_seconds,
            idle_seconds=idle_seconds,
            with_music=with_music,
            provision_test_music=provision_test_music,
            test_music_dir=test_music_dir,
            skip_sleep=skip_sleep,
            verbose=verbose,
        ),
        tty=True,
    )
    if rc != 0:
        raise typer.Exit(code=rc)
