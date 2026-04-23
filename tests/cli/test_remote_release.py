from __future__ import annotations

import json
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest
from typer.testing import CliRunner

from yoyopod_cli.remote_release import app as release_app

runner = CliRunner()


def _write_slot(tmp_path: Path, version: str) -> Path:
    slot = tmp_path / version
    slot.mkdir()
    (slot / "manifest.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "version": version,
                "channel": "dev",
                "released_at": "2026-04-22T10:00:00Z",
                "artifacts": {"full": {"type": "full", "sha256": "a" * 64, "size": 100}},
                "requires": {"min_os_version": "0.0.0", "min_battery_pct": 0, "min_free_mb": 0},
            }
        )
    )
    (slot / "app").mkdir()
    (slot / "venv").mkdir()
    (slot / "bin").mkdir()
    launch = slot / "bin" / "launch"
    launch.write_text("#!/bin/sh\nexit 0\n")
    launch.chmod(0o755)
    return slot


@patch("yoyopod_cli.remote_release._rsync_to_pi")
@patch("yoyopod_cli.remote_release._run_preflight_on_pi")
@patch("yoyopod_cli.remote_release._flip_symlinks_on_pi")
@patch("yoyopod_cli.remote_release._run_live_probe_on_pi")
def test_push_runs_build_rsync_preflight_flip_live(
    live_probe: MagicMock,
    flip: MagicMock,
    preflight: MagicMock,
    rsync: MagicMock,
    tmp_path: Path,
) -> None:
    slot = _write_slot(tmp_path, "2026.04.22-abc")
    rsync.return_value = 0
    preflight.return_value = 0
    flip.return_value = 0
    live_probe.return_value = 0

    result = runner.invoke(release_app, ["push", str(slot)])
    assert result.exit_code == 0, result.stdout

    rsync.assert_called_once()
    preflight.assert_called_once()
    flip.assert_called_once()
    live_probe.assert_called_once()


@patch("yoyopod_cli.remote_release._rsync_to_pi")
@patch("yoyopod_cli.remote_release._run_preflight_on_pi")
@patch("yoyopod_cli.remote_release._flip_symlinks_on_pi")
def test_push_aborts_and_cleans_up_on_preflight_fail(
    flip: MagicMock,
    preflight: MagicMock,
    rsync: MagicMock,
    tmp_path: Path,
) -> None:
    slot = _write_slot(tmp_path, "2026.04.22-abc")
    rsync.return_value = 0
    preflight.return_value = 1

    result = runner.invoke(release_app, ["push", str(slot)])
    assert result.exit_code != 0
    flip.assert_not_called()


@patch("yoyopod_cli.remote_release._rsync_to_pi")
@patch("yoyopod_cli.remote_release._run_preflight_on_pi")
@patch("yoyopod_cli.remote_release._flip_symlinks_on_pi")
@patch("yoyopod_cli.remote_release._run_live_probe_on_pi")
@patch("yoyopod_cli.remote_release._rollback_on_pi")
def test_push_rolls_back_on_live_fail(
    rollback: MagicMock,
    live: MagicMock,
    flip: MagicMock,
    preflight: MagicMock,
    rsync: MagicMock,
    tmp_path: Path,
) -> None:
    slot = _write_slot(tmp_path, "2026.04.22-abc")
    rsync.return_value = 0
    preflight.return_value = 0
    flip.return_value = 0
    live.return_value = 1

    result = runner.invoke(release_app, ["push", str(slot)])
    assert result.exit_code != 0
    rollback.assert_called_once()


def test_push_rejects_non_slot_directory(tmp_path: Path) -> None:
    bogus = tmp_path / "not_a_slot"
    bogus.mkdir()
    result = runner.invoke(release_app, ["push", str(bogus)])
    assert result.exit_code != 0


@patch("yoyopod_cli.remote_release._rollback_on_pi")
def test_rollback_invokes_pi_side(rollback: MagicMock) -> None:
    rollback.return_value = 0
    result = runner.invoke(release_app, ["rollback"])
    assert result.exit_code == 0
    rollback.assert_called_once()


@patch("yoyopod_cli.remote_release._status_from_pi")
def test_status_prints_current_and_previous(status: MagicMock) -> None:
    status.return_value = (
        "current=2026.04.22-abc\nprevious=2026.04.20-def\nhealth=ok\n"
    )
    result = runner.invoke(release_app, ["status"])
    assert result.exit_code == 0
    assert "2026.04.22-abc" in result.stdout
    assert "2026.04.20-def" in result.stdout
