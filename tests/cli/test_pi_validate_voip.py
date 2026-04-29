from __future__ import annotations

import os
from pathlib import Path

from typer.testing import CliRunner

import yoyopod_cli.pi.validate.voip as pi_validate_voip
from yoyopod_cli.pi.validate import app as pi_validate_app


def test_voip_validation_loads_service_env_file(
    tmp_path: Path,
    monkeypatch,
) -> None:
    env_file = tmp_path / "yoyopod-dev.env"
    env_file.write_text(
        "\n".join(
            [
                "# service-style env file",
                "YOYOPOD_SIP_IDENTITY='sip:tifo@sip.linphone.org'",
                "YOYOPOD_SIP_USERNAME=tifo",
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("YOYOPOD_SIP_IDENTITY", "")
    monkeypatch.setenv("YOYOPOD_SIP_USERNAME", "")

    observed: list[tuple[str, str]] = []

    def fake_quick_check(config_dir: str, registration_timeout: float) -> None:
        observed.append(
            (
                os.environ["YOYOPOD_SIP_IDENTITY"],
                os.environ["YOYOPOD_SIP_USERNAME"],
            )
        )

    monkeypatch.setattr(pi_validate_voip, "_run_quick_voip_check", fake_quick_check)

    result = CliRunner().invoke(
        pi_validate_app,
        ["voip", "--env-file", str(env_file)],
    )

    assert result.exit_code == 0, result.output
    assert observed == [("sip:tifo@sip.linphone.org", "tifo")]
