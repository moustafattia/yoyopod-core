"""Helpers for importing systemd EnvironmentFile settings in Pi validators."""

from __future__ import annotations

import os
import shlex
from pathlib import Path

from yoyopod_cli.common import REPO_ROOT


def resolve_service_env_file(env_file: str) -> Path:
    """Resolve an EnvironmentFile path from CLI input."""

    env_path = Path(env_file)
    if not env_path.is_absolute():
        env_path = REPO_ROOT / env_path
    return env_path


def load_service_env_file(env_file: Path) -> list[str]:
    """Load service-style KEY=VALUE assignments into this process environment."""

    if not env_file.exists():
        return []

    loaded: list[str] = []
    for raw_line in env_file.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        try:
            parts = shlex.split(line, comments=True, posix=True)
        except ValueError:
            continue
        if not parts:
            continue
        if parts[0] == "export":
            parts = parts[1:]
        if not parts or "=" not in parts[0]:
            continue
        key, value = parts[0].split("=", 1)
        key = key.strip()
        if not key:
            continue
        os.environ[key] = value
        loaded.append(key)
    return loaded
