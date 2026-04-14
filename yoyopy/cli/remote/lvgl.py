"""yoyopy/cli/remote/lvgl.py — LVGL soak over SSH."""

from __future__ import annotations

from typing import Annotated

import typer

from yoyopy.cli.remote.ops import (
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
            skip_sleep=skip_sleep,
            verbose=verbose,
        ),
        tty=True,
    )
    if rc != 0:
        raise typer.Exit(code=rc)
