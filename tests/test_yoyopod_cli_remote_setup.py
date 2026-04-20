from __future__ import annotations

from typer.testing import CliRunner

from yoyopod_cli.remote_setup import app, _build_setup, _build_verify_setup


def test_build_setup_calls_pi_setup() -> None:
    assert "yoyopod setup pi" in _build_setup()


def test_build_verify_setup_calls_pi_verify() -> None:
    assert "yoyopod setup verify-pi" in _build_verify_setup()


def test_setup_help() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["setup", "--help"])
    assert result.exit_code == 0


def test_verify_setup_help() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["verify-setup", "--help"])
    assert result.exit_code == 0
