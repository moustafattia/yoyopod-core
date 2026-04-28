from __future__ import annotations

from typer.testing import CliRunner

from yoyopod_cli.pi import app


def test_rust_ui_poc_alias_still_works() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["rust-ui-poc", "--help"])

    assert result.exit_code == 0
    assert "rust ui host" in result.output.lower()
