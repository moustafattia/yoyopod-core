from __future__ import annotations

from pathlib import Path

from typer.testing import CliRunner

from yoyopod.ui.rust_sidecar.protocol import UiEnvelope
from yoyopod_cli.pi import app
import yoyopod_cli.pi.rust_ui_host as rust_ui_host


class _FakeSupervisor:
    instances: list["_FakeSupervisor"] = []

    def __init__(self, argv: list[str], cwd: Path | None = None) -> None:
        self.argv = argv
        self.cwd = cwd
        self.sent: list[UiEnvelope] = []
        self.instances.append(self)

    def start(self) -> UiEnvelope:
        return UiEnvelope(
            kind="event",
            type="ui.ready",
            payload={"display": {"width": 240}},
        )

    def send(self, envelope: UiEnvelope) -> None:
        self.sent.append(envelope)

    def read_event(self) -> UiEnvelope:
        return UiEnvelope(
            kind="event",
            type="ui.health",
            payload={"frames": 1, "button_events": 0},
        )

    def stop(self) -> None:
        return None


def test_rust_ui_host_help() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["rust-ui-host", "--help"])

    assert result.exit_code == 0
    assert "rust ui host" in result.output.lower()


def test_rust_ui_host_runs_supervisor(monkeypatch, tmp_path: Path) -> None:
    worker = tmp_path / "yoyopod-ui-host"
    worker.write_text("fake", encoding="utf-8")
    _FakeSupervisor.instances.clear()
    monkeypatch.setattr(rust_ui_host, "RustUiHostSupervisor", _FakeSupervisor)

    runner = CliRunner()
    result = runner.invoke(app, ["rust-ui-host", "--worker", str(worker), "--frames", "1"])

    assert result.exit_code == 0
    assert "ready" in result.output.lower()
    assert "frames=1" in result.output


def test_rust_ui_host_can_request_static_hub(monkeypatch, tmp_path: Path) -> None:
    worker = tmp_path / "yoyopod-ui-host"
    worker.write_text("fake", encoding="utf-8")
    _FakeSupervisor.instances.clear()
    monkeypatch.setattr(rust_ui_host, "RustUiHostSupervisor", _FakeSupervisor)

    runner = CliRunner()
    result = runner.invoke(
        app,
        [
            "rust-ui-host",
            "--worker",
            str(worker),
            "--frames",
            "1",
            "--screen",
            "hub",
            "--hub-renderer",
            "lvgl",
        ],
    )

    assert result.exit_code == 0
    sent = _FakeSupervisor.instances[-1].sent[0]
    assert sent.type == "ui.show_hub"
    assert sent.payload["renderer"] == "lvgl"
    assert sent.payload["title"] == "Listen"
