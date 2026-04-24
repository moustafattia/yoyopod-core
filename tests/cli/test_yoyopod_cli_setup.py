"""Tests for yoyopod_cli.setup — host + Pi setup commands."""

from __future__ import annotations

from typer.testing import CliRunner

from yoyopod_cli.setup import app, build_pi_setup_commands


def test_host_verify_help() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["verify-host", "--help"])
    assert result.exit_code == 0


def test_pi_verify_help() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["verify-pi", "--help"])
    assert result.exit_code == 0


def test_setup_lists_all_four_commands() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["--help"])
    assert result.exit_code == 0
    assert "host" in result.output
    assert "pi" in result.output
    assert "verify-host" in result.output
    assert "verify-pi" in result.output


def test_build_pi_setup_commands_bootstraps_checkout_venv_without_uv() -> None:
    commands = build_pi_setup_commands(
        with_voice=False,
        with_network=False,
        with_pisugar=False,
        skip_uv_sync=False,
        skip_builds=False,
    )

    argv_strings = [" ".join(step.command) for step in commands]
    assert any(argv.startswith("sudo apt install -y python3-venv ") for argv in argv_strings)
    assert "python3 -m venv .venv" in argv_strings
    assert ".venv/bin/python -m pip install --upgrade pip setuptools wheel" in argv_strings
    assert ".venv/bin/python -m pip install -e .[dev]" in argv_strings
    assert ".venv/bin/python -m yoyopod_cli.main build liblinphone" in argv_strings
    assert ".venv/bin/python -m yoyopod_cli.main build lvgl" in argv_strings
    assert not any("uv " in argv for argv in argv_strings)
