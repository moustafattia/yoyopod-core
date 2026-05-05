from __future__ import annotations

import subprocess
from pathlib import Path

import tomllib


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_ROOT = REPO_ROOT / "yoyopod_rs"


def _cargo_workspace_members() -> set[str]:
    with (RUST_ROOT / "Cargo.toml").open("rb") as handle:
        payload = tomllib.load(handle)
    return set(payload["workspace"]["members"])


def _git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def test_rust_workspace_has_no_generated_target_member() -> None:
    members = _cargo_workspace_members()

    assert "target" not in members
    assert not any(member.startswith("target/") for member in members)


def test_rust_workspace_tracks_lockfile_even_when_cargo_lock_is_ignored() -> None:
    assert (RUST_ROOT / "Cargo.lock").is_file()

    tracked = _git("ls-files", "--error-unmatch", "yoyopod_rs/Cargo.lock")
    assert tracked.returncode == 0

    ignored = _git("check-ignore", "-q", "yoyopod_rs/Cargo.lock")
    assert ignored.returncode != 0
