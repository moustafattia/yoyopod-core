"""src/yoyopod/cli/remote/navigation.py — navigation soak over SSH."""

from __future__ import annotations

import shlex
from typing import Annotated

import typer

from yoyopod.cli.pi.music_fixtures import DEFAULT_TEST_MUSIC_TARGET_DIR
from yoyopod.cli.remote.config import load_pi_deploy_config
from yoyopod.cli.remote.ops import (
    _resolve_remote_config,
    run_remote,
    validate_config,
)


def build_navigation_soak_command(
    *,
    cycles: int = 2,
    hold_seconds: float = 0.35,
    idle_seconds: float = 3.0,
    tail_idle_seconds: float = 10.0,
    with_playback: bool = True,
    provision_test_music: bool = True,
    test_music_target_dir: str | None = None,
    skip_sleep: bool = False,
    verbose: bool = False,
) -> str:
    """Create the remote target navigation-soak command."""

    parts = ["uv run yoyoctl pi validate navigation"]
    if verbose:
        parts.append("--verbose")
    if cycles != 2:
        parts.extend(["--cycles", str(cycles)])
    if hold_seconds != 0.35:
        parts.extend(["--hold-seconds", str(hold_seconds)])
    if idle_seconds != 3.0:
        parts.extend(["--idle-seconds", str(idle_seconds)])
    if tail_idle_seconds != 10.0:
        parts.extend(["--tail-idle-seconds", str(tail_idle_seconds)])
    if not with_playback:
        parts.append("--no-with-playback")
    if not provision_test_music:
        parts.append("--no-provision-test-music")
    if test_music_target_dir and test_music_target_dir != DEFAULT_TEST_MUSIC_TARGET_DIR:
        parts.extend(["--test-music-dir", shlex.quote(test_music_target_dir)])
    if skip_sleep:
        parts.append("--skip-sleep")
    return " ".join(parts)


def navigation_soak(
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
        int, typer.Option("--cycles", help="How many full navigation cycles to run.")
    ] = 2,
    hold_seconds: Annotated[
        float,
        typer.Option(
            "--hold-seconds",
            help="How long to pump after each simulated click or route change.",
        ),
    ] = 0.35,
    idle_seconds: Annotated[
        float,
        typer.Option(
            "--idle-seconds",
            help="How long to leave each exercised screen idle before the next action.",
        ),
    ] = 3.0,
    tail_idle_seconds: Annotated[
        float,
        typer.Option(
            "--tail-idle-seconds",
            help="Final idle dwell on the hub after all navigation cycles complete.",
        ),
    ] = 10.0,
    with_playback: Annotated[
        bool,
        typer.Option(
            "--with-playback/--no-with-playback",
            help="Drive playlist and shuffle playback paths during the soak.",
        ),
    ] = True,
    provision_test_music: Annotated[
        bool,
        typer.Option(
            "--provision-test-music/--no-provision-test-music",
            help="Seed deterministic validation music before playback-driven navigation.",
        ),
    ] = True,
    test_music_dir: Annotated[
        str,
        typer.Option(
            "--test-music-dir",
            help="Dedicated target directory for validation-only test music assets.",
        ),
    ] = "",
    skip_sleep: Annotated[
        bool, typer.Option("--skip-sleep", help="Skip the final sleep/wake exercise.")
    ] = False,
    verbose: Annotated[
        bool, typer.Option("--verbose", help="Enable verbose soak logging.")
    ] = False,
) -> None:
    """Run the target navigation and idle soak helper remotely."""

    config = _resolve_remote_config(host, user, project_dir, branch)
    validate_config(config)
    deploy_config = load_pi_deploy_config()
    resolved_test_music_dir = test_music_dir or deploy_config.test_music_target_dir
    rc = run_remote(
        config,
        build_navigation_soak_command(
            cycles=cycles,
            hold_seconds=hold_seconds,
            idle_seconds=idle_seconds,
            tail_idle_seconds=tail_idle_seconds,
            with_playback=with_playback,
            provision_test_music=provision_test_music,
            test_music_target_dir=resolved_test_music_dir,
            skip_sleep=skip_sleep,
            verbose=verbose,
        ),
        tty=True,
    )
    if rc != 0:
        raise typer.Exit(code=rc)
