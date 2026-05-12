"""YoYoPod operations CLI.

Usage:
    yoyopod --help               # Show command groups
    yoyopod release <cmd>        # Versioning and release artifacts
    yoyopod remote <cmd>         # Dev-machine → Pi commands
    yoyopod pi <cmd>             # On-device commands
    yoyopod build <cmd>          # Native extension builds
    yoyopod setup <cmd>          # Host and Pi setup
"""

from __future__ import annotations

import typer

from yoyopod_cli import __version__

app = typer.Typer(name="yoyopod", help="YoYoPod operations CLI.", add_completion=False)


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
    """Manage YoYoPod build, setup, release, remote Pi, and on-device operations."""
    if ctx.invoked_subcommand is None:
        typer.echo(ctx.get_help())
        raise typer.Exit(0)


def run() -> None:
    """Entry-point shim used by ``[project.scripts]``."""
    app()


# --- subapps ---
from yoyopod_cli import build as _build  # noqa: E402

app.add_typer(_build.app, name="build")

from yoyopod_cli import health as _health  # noqa: E402

app.add_typer(_health.app, name="health")

from yoyopod_cli import voice as _voice  # noqa: E402

app.add_typer(_voice.app, name="voice")

from yoyopod_cli import release as _release  # noqa: E402

app.add_typer(_release.app, name="release")

from yoyopod_cli import setup as _setup  # noqa: E402

app.add_typer(_setup.app, name="setup")

# --- remote group (assembled from flat sub-modules)
from yoyopod_cli import (  # noqa: E402
    remote_config as _remote_config,
    remote_infra as _remote_infra,
    remote_mode as _remote_mode,
    remote_ops as _remote_ops,
    remote_release as _remote_release,
    remote_setup as _remote_setup,
    remote_validate as _remote_validate,
)
from yoyopod_cli.remote_shared import build_remote_app as _build_remote_app  # noqa: E402

remote_app = _build_remote_app("remote", "Dev-machine -> Pi commands via SSH.")

# ops commands
remote_app.command(name="status")(_remote_ops.status)
remote_app.command(name="sync")(_remote_ops.sync)
remote_app.command(name="restart")(_remote_ops.restart)
remote_app.command(name="logs")(_remote_ops.logs)
remote_app.command(name="screenshot")(_remote_ops.screenshot)

# validate / preflight
remote_app.command(name="preflight")(_remote_validate.preflight)
remote_app.command(name="validate")(_remote_validate.validate)

# infra
remote_app.command(name="power")(_remote_infra.power)
remote_app.command(name="rtc")(_remote_infra.rtc)

# setup
remote_app.command(name="setup")(_remote_setup.setup)
remote_app.command(name="verify-setup")(_remote_setup.verify_setup)

# config (operates on local files — its own subgroup)
remote_app.add_typer(_remote_config.app, name="config")

# release (slot-deploy push/rollback/status)
remote_app.add_typer(_remote_release.app, name="release")

# mode (dev/prod lane switch)
remote_app.add_typer(_remote_mode.app, name="mode")

app.add_typer(remote_app, name="remote")

# --- pi group (commands that run on the Pi directly)
from yoyopod_cli.pi import app as pi_app  # noqa: E402

app.add_typer(pi_app, name="pi")


# --- dev utilities
dev_app = typer.Typer(name="dev", help="Developer utilities.", no_args_is_help=True)
app.add_typer(dev_app, name="dev")


@dev_app.command()
def docs() -> None:
    """Regenerate yoyopod_cli/COMMANDS.md from the live Typer tree."""
    from yoyopod_cli._docgen import generate_commands_md
    from yoyopod_cli.paths import HOST

    md = generate_commands_md(app)
    (HOST.repo_root / "yoyopod_cli" / "COMMANDS.md").write_text(md, encoding="utf-8")
    typer.echo("yoyopod_cli/COMMANDS.md regenerated.")


if __name__ == "__main__":
    run()
