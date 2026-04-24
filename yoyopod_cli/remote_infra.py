"""Remote infra commands: power snapshot, rtc, systemd service management."""

from __future__ import annotations

import typer

from yoyopod_cli.common import checkout_module_command, configure_logging
from yoyopod_cli.paths import load_pi_paths
from yoyopod_cli.remote_shared import build_remote_app, pi_conn
from yoyopod_cli.remote_transport import (
    run_remote,
    shell_quote,
    validate_config,
)

app = build_remote_app("infra", "Remote power, rtc, and service commands.")


def _build_power(*, venv_relpath: str) -> str:
    """Invoke ``yoyopod pi power battery`` on the Pi."""
    return checkout_module_command(venv_relpath, "pi", "power", "battery")


def _build_rtc(action: str, *, venv_relpath: str, time_iso: str, repeat_mask: int) -> str:
    """Build ``yoyopod pi power rtc <action>`` remote shell."""
    cmd = f"{checkout_module_command(venv_relpath, 'pi', 'power', 'rtc', action)}"
    if action == "set-alarm":
        if not time_iso:
            raise typer.BadParameter("set-alarm requires --time")
        cmd += f" --time {shell_quote(time_iso)} --repeat-mask {repeat_mask}"
    return cmd


def _build_service_install() -> str:
    """Shell that installs the systemd unit and writes the env file."""
    return (
        "sudo tee /etc/default/yoyopod > /dev/null <<ENV_EOF && "
        "sudo cp deploy/systemd/yoyopod@.service /etc/systemd/system/ && "
        "sudo systemctl daemon-reload && "
        "sudo systemctl enable --now yoyopod@$USER\n"
        'YOYOPOD_PROJECT_DIR="$PWD"\n'
        "ENV_EOF"
    )


def _build_service_uninstall() -> str:
    """Shell that removes the systemd unit and its env file."""
    return (
        "sudo systemctl disable --now yoyopod@$USER && "
        "sudo rm -f /etc/systemd/system/yoyopod@.service && "
        "sudo rm -f /etc/default/yoyopod && "
        "sudo systemctl daemon-reload"
    )


def _build_service_action(action: str) -> str:
    """Build `sudo systemctl <action> yoyopod@$USER`."""
    return f"sudo systemctl {action} yoyopod@$USER"


@app.command()
def power(ctx: typer.Context, verbose: bool = typer.Option(False, "--verbose")) -> None:
    """Query PiSugar state remotely."""
    configure_logging(verbose)
    conn = pi_conn(ctx)
    validate_config(conn)
    pi = load_pi_paths()
    raise typer.Exit(run_remote(conn, _build_power(venv_relpath=pi.venv)))


@app.command()
def rtc(
    ctx: typer.Context,
    action: str = typer.Argument(
        "status", help="status | sync-to | sync-from | set-alarm | disable-alarm"
    ),
    time: str = typer.Option("", "--time", help="ISO 8601 timestamp for set-alarm."),
    repeat_mask: int = typer.Option(
        127, "--repeat-mask", help="Repeat-bitmask (default every day)."
    ),
    verbose: bool = typer.Option(False, "--verbose"),
) -> None:
    """Inspect or control PiSugar RTC remotely."""
    configure_logging(verbose)
    conn = pi_conn(ctx)
    validate_config(conn)
    pi = load_pi_paths()
    raise typer.Exit(
        run_remote(
            conn, _build_rtc(action, venv_relpath=pi.venv, time_iso=time, repeat_mask=repeat_mask)
        )
    )


@app.command()
def service(
    ctx: typer.Context,
    action: str = typer.Argument(..., help="install | uninstall | status | start | stop"),
    verbose: bool = typer.Option(False, "--verbose"),
) -> None:
    """Manage the yoyopod@<user> systemd unit on the Pi."""
    configure_logging(verbose)
    conn = pi_conn(ctx)
    validate_config(conn)
    if action == "install":
        cmd = _build_service_install()
    elif action == "uninstall":
        cmd = _build_service_uninstall()
    elif action in ("status", "start", "stop"):
        cmd = _build_service_action(action)
    else:
        raise typer.BadParameter(f"unknown action: {action}")
    raise typer.Exit(run_remote(conn, cmd))
