"""
yoyopod — app launcher and CLI dispatcher.

Usage:
    yoyopod                      # Launch the YoyoPod app
    yoyopod deploy               # Sync code to the Pi and restart
    yoyopod status               # Pi health dashboard
    yoyopod logs [-f --errors]   # Tail logs from the Pi
    yoyopod restart              # Restart the app on the Pi
    yoyopod validate             # Run the validation suite on the Pi
    yoyopod remote <cmd>         # Dev-machine → Pi commands
    yoyopod pi <cmd>             # On-device commands
    yoyopod build <cmd>          # Native extension builds
    yoyopod setup <cmd>          # Host and Pi setup
"""

from __future__ import annotations

import typer

from yoyopod_cli import __version__

app = typer.Typer(
    name="yoyopod",
    help="YoyoPod app launcher and CLI.",
    no_args_is_help=False,
    add_completion=False,
)


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(f"yoyopod {__version__}")
        raise typer.Exit()


@app.callback(invoke_without_command=True)
def _root(
    ctx: typer.Context,
    version: bool = typer.Option(
        False,
        "--version",
        callback=_version_callback,
        is_eager=True,
        help="Show version and exit.",
    ),
) -> None:
    """Launch the YoyoPod app when invoked with no subcommand."""
    if ctx.invoked_subcommand is None:
        from yoyopod.main import main as launch_app

        launch_app()


def run() -> None:
    """Entry-point shim used by ``[project.scripts]``."""
    app()


# --- subapps ---
from yoyopod_cli import build as _build

app.add_typer(_build.app, name="build")
