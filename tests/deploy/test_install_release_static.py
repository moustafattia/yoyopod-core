from __future__ import annotations

from pathlib import Path


INSTALL_RELEASE_SH = (
    Path(__file__).resolve().parents[2] / "deploy" / "scripts" / "install_release.sh"
)


def test_install_release_passes_selected_service_name_to_rollback_fallback() -> None:
    script = INSTALL_RELEASE_SH.read_text(encoding="utf-8")

    assert 'YOYOPOD_SERVICE_NAME="${SERVICE_NAME}" "${ROOT}/bin/rollback.sh"' in script
    assert script.count('YOYOPOD_SERVICE_NAME="${SERVICE_NAME}" "${ROOT}/bin/rollback.sh"') == 2
