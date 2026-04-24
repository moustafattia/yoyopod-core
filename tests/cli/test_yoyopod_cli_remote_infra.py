"""Tests for yoyopod_cli.remote_infra — power, rtc, service."""

from __future__ import annotations

from typer.testing import CliRunner

from yoyopod_cli.remote_infra import (
    app,
    _build_power,
    _build_rtc,
    _build_service_install,
    _build_service_uninstall,
    _build_service_action,
)
from yoyopod_cli.paths import HOST


def test_build_power_invokes_pi_power_battery() -> None:
    shell = _build_power(venv_relpath=".venv")
    assert ".venv/bin/python -m yoyopod_cli.main pi power battery" in shell


def test_build_rtc_status() -> None:
    shell = _build_rtc("status", venv_relpath=".venv", time_iso="", repeat_mask=127)
    assert ".venv/bin/python -m yoyopod_cli.main pi power rtc" in shell
    assert "status" in shell


def test_build_rtc_set_alarm_with_time() -> None:
    shell = _build_rtc(
        "set-alarm",
        venv_relpath=".venv",
        time_iso="2026-04-20T07:00:00",
        repeat_mask=127,
    )
    assert "set-alarm" in shell
    assert "2026-04-20T07:00:00" in shell
    assert "127" in shell


def test_build_rtc_set_alarm_without_time_fails() -> None:
    import pytest
    import typer

    with pytest.raises(typer.BadParameter):
        _build_rtc("set-alarm", venv_relpath=".venv", time_iso="", repeat_mask=127)


def test_build_service_install_uses_relative_template_path() -> None:
    shell = _build_service_install()
    assert "deploy/systemd/yoyopod@.service" in shell
    assert "systemctl daemon-reload" in shell
    assert "systemctl enable" in shell
    # Must NOT contain an absolute host path
    assert "/home/" not in shell
    assert "/Users/" not in shell
    assert "c:/users" not in shell.lower()


def test_build_service_action_start() -> None:
    shell = _build_service_action("start")
    assert shell == "sudo systemctl start yoyopod@$USER"


def test_power_cli_invokes_run_remote(monkeypatch) -> None:
    calls: list[tuple[object, str]] = []

    def fake_run_remote(conn, cmd, tty=False):
        calls.append((conn, cmd))
        return 0

    monkeypatch.setattr("yoyopod_cli.remote_infra.run_remote", fake_run_remote)
    monkeypatch.setenv("YOYOPOD_PI_HOST", "rpi-zero")

    runner = CliRunner()
    result = runner.invoke(app, ["power"])
    assert result.exit_code == 0, result.output
    assert len(calls) == 1


# --- Fix 2: checkout-local module invocation for power / rtc ---


def test_build_power_uses_checkout_python_module_invocation() -> None:
    shell = _build_power(venv_relpath=".venv")
    assert ".venv/bin/python -m yoyopod_cli.main pi power battery" in shell
    assert "source " not in shell
    assert "uv run" not in shell


def test_build_rtc_uses_checkout_python_module_invocation() -> None:
    shell = _build_rtc("status", venv_relpath=".venv", time_iso="", repeat_mask=127)
    assert ".venv/bin/python -m yoyopod_cli.main pi power rtc" in shell
    assert "source " not in shell
    assert "uv run" not in shell


# --- Fix 4: service install persists env file ---


def test_build_service_install_persists_project_dir_env_file() -> None:
    shell = _build_service_install()
    assert "/etc/default/yoyopod" in shell
    assert "YOYOPOD_PROJECT_DIR=" in shell
    # Must use $PWD (= conn.project_dir after cd) not a host-absolute path
    assert "$PWD" in shell
    # Must NOT embed the systemd unit host-absolute path
    assert "/home/" not in shell
    assert "/Users/" not in shell


def test_build_service_install_chains_env_write_with_copy_and_enable() -> None:
    shell = _build_service_install()
    assert "<<ENV_EOF && sudo cp deploy/systemd/yoyopod@.service" in shell
    assert "<<'ENV_EOF'" not in shell
    assert "sudo systemctl enable --now yoyopod@$USER" in shell


def test_build_service_install_copies_template_and_reloads() -> None:
    shell = _build_service_install()
    assert "deploy/systemd/yoyopod@.service" in shell
    assert "systemctl daemon-reload" in shell
    assert "systemctl enable" in shell


def test_build_service_uninstall_removes_env_file() -> None:
    """Uninstall should clean up /etc/default/yoyopod for a clean system."""
    shell = _build_service_uninstall()
    assert "/etc/default/yoyopod" in shell
    assert "systemctl disable" in shell
    assert "systemctl daemon-reload" in shell


def test_systemd_template_refreshes_native_shims_before_start() -> None:
    template = HOST.systemd_unit_template.read_text(encoding="utf-8")

    assert "ExecStartPre=" in template
    assert ".venv/bin/python" in template
    assert "-m yoyopod_cli.main build ensure-native" in template
