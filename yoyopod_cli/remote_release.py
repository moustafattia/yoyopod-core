"""yoyopod remote release {push,rollback,status} — slot-deploy CLI."""

from __future__ import annotations

import shlex
import subprocess
from pathlib import Path

import typer

from yoyopod_cli.release_manifest import load_manifest
from yoyopod_cli.remote_shared import _resolve_remote_connection
from yoyopod_cli.remote_transport import run_remote, run_remote_capture

app = typer.Typer(name="release", help="Slot-deploy push/rollback/status.", no_args_is_help=True)

_SLOT_ROOT = "/opt/yoyopod"


def _conn() -> object:
    """Resolve RemoteConnection from env/YAML defaults (no CLI flags required)."""
    return _resolve_remote_connection("", "", "", "")


def _rsync_to_pi(slot: Path, version: str, pi_host: str, pi_user: str) -> int:
    """Rsync a local slot directory to the Pi release store."""
    target = (
        f"{pi_user}@{pi_host}:{_SLOT_ROOT}/releases/{version}/"
        if pi_user
        else f"{pi_host}:{_SLOT_ROOT}/releases/{version}/"
    )
    cmd = ["rsync", "-az", "--delete", f"{slot}/", target]
    return subprocess.run(cmd, check=False).returncode


def _run_preflight_on_pi(version: str) -> int:
    """Run the preflight health check for the uploaded slot on the Pi."""
    conn = _conn()
    cmd = f"yoyopod health preflight --slot {_SLOT_ROOT}/releases/{shlex.quote(version)}"
    return run_remote(conn, cmd)  # type: ignore[arg-type]


def _flip_symlinks_on_pi(version: str) -> int:
    """Atomically flip current → new slot and restart the service."""
    conn = _conn()
    resolve_cmd = f"readlink -f {_SLOT_ROOT}/current 2>/dev/null || echo NONE"
    proc = run_remote_capture(conn, resolve_cmd)  # type: ignore[arg-type]
    current_target = proc.stdout.strip()
    flip_cmd = "; ".join(
        [
            (
                f"if [ '{current_target}' != 'NONE' ]; then "
                f"ln -sfn {shlex.quote(current_target)} {_SLOT_ROOT}/previous.new && "
                f"mv -T {_SLOT_ROOT}/previous.new {_SLOT_ROOT}/previous; fi"
            ),
            f"ln -sfn {_SLOT_ROOT}/releases/{shlex.quote(version)} {_SLOT_ROOT}/current.new",
            f"mv -T {_SLOT_ROOT}/current.new {_SLOT_ROOT}/current",
            "sudo systemctl restart yoyopod-slot.service",
        ]
    )
    return run_remote(conn, flip_cmd)  # type: ignore[arg-type]


def _run_live_probe_on_pi(version: str, timeout_s: int = 60) -> int:
    """Poll the Pi until the new version reports as live, or timeout."""
    conn = _conn()
    cmd = (
        f"for i in $(seq 1 {timeout_s}); do "
        f"out=$(yoyopod health live 2>/dev/null) && "
        f"echo \"$out\" | grep -q {shlex.quote('version=' + version)} && exit 0; "
        f"sleep 1; done; exit 1"
    )
    return run_remote(conn, cmd)  # type: ignore[arg-type]


def _rollback_on_pi() -> int:
    """Invoke the rollback script on the Pi (swaps current ↔ previous)."""
    conn = _conn()
    return run_remote(conn, f"sudo {_SLOT_ROOT}/bin/rollback.sh")  # type: ignore[arg-type]


def _status_from_pi() -> str:
    """Retrieve current/previous/health status lines from the Pi."""
    conn = _conn()
    cmd = (
        f"echo current=$(readlink -f {_SLOT_ROOT}/current 2>/dev/null | xargs -n1 basename) && "
        f"echo previous=$(readlink -f {_SLOT_ROOT}/previous 2>/dev/null | xargs -n1 basename) && "
        f"echo health=$(yoyopod health live 2>/dev/null >/dev/null && echo ok || echo fail)"
    )
    proc = run_remote_capture(conn, cmd)  # type: ignore[arg-type]
    return proc.stdout


def _cleanup_remote_slot(version: str) -> None:
    """Remove a partially-uploaded slot from the Pi."""
    conn = _conn()
    run_remote(conn, f"rm -rf {_SLOT_ROOT}/releases/{shlex.quote(version)}")  # type: ignore[arg-type]


@app.command("push")
def push(
    slot: Path = typer.Argument(..., help="Local release slot dir from build_release."),
) -> None:
    """Push a pre-built slot dir to the Pi and atomically switch to it."""
    manifest_path = slot / "manifest.json"
    if not manifest_path.exists():
        typer.echo(f"not a release slot (no manifest.json): {slot}", err=True)
        raise typer.Exit(code=2)
    try:
        manifest = load_manifest(manifest_path)
    except ValueError as exc:
        typer.echo(f"invalid manifest: {exc}", err=True)
        raise typer.Exit(code=2) from exc

    conn = _conn()
    host: str = getattr(conn, "host", "")
    user: str = getattr(conn, "user", "")

    typer.echo(f"rsync -> {user}@{host}:{_SLOT_ROOT}/releases/{manifest.version}/")
    rc = _rsync_to_pi(slot, manifest.version, host, user)
    if rc != 0:
        typer.echo("rsync failed", err=True)
        raise typer.Exit(code=rc)

    typer.echo("preflight...")
    rc = _run_preflight_on_pi(manifest.version)
    if rc != 0:
        typer.echo("preflight failed -- removing uploaded slot", err=True)
        _cleanup_remote_slot(manifest.version)
        raise typer.Exit(code=rc)

    typer.echo("flip + restart...")
    rc = _flip_symlinks_on_pi(manifest.version)
    if rc != 0:
        typer.echo("symlink flip / restart failed", err=True)
        raise typer.Exit(code=rc)

    typer.echo("live probe...")
    rc = _run_live_probe_on_pi(manifest.version)
    if rc != 0:
        typer.echo("live probe failed -- rolling back", err=True)
        _rollback_on_pi()
        raise typer.Exit(code=rc)

    typer.echo(f"released {manifest.version}")


@app.command("rollback")
def rollback() -> None:
    """Swap current <-> previous on the Pi and restart."""
    rc = _rollback_on_pi()
    if rc != 0:
        raise typer.Exit(code=rc)
    typer.echo("rollback complete")


@app.command("status")
def status() -> None:
    """Print current / previous / health from the Pi."""
    typer.echo(_status_from_pi())
