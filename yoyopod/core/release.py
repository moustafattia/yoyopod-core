"""Release metadata for the running app.

Reads /opt/yoyopod/current/manifest.json (location overridable via
YOYOPOD_RELEASE_MANIFEST env var). Returns None in dev mode — callers
should fall back to 'dev' / 'unknown' gracefully.

State directory is resolved from YOYOPOD_STATE_DIR; defaults to the
XDG data dir so dev mode doesn't write to /opt.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class ReleaseInfo:
    """A subset of the manifest the running app cares about."""

    version: str
    channel: str
    released_at: str


_DEFAULT_MANIFEST_PATH = "/opt/yoyopod/current/manifest.json"
_DEFAULT_STATE_DIR = Path.home() / ".local" / "share" / "yoyopod"


def _manifest_path() -> Path:
    return Path(os.environ.get("YOYOPOD_RELEASE_MANIFEST", _DEFAULT_MANIFEST_PATH))


def current_release() -> ReleaseInfo | None:
    """Return the currently-running release, or None if not in a slot deploy.

    Corrupt or missing manifests return None so dev-mode + fresh-boot paths
    never fail on metadata resolution.
    """
    path = _manifest_path()
    if not path.exists():
        return None
    try:
        raw = json.loads(path.read_text())
        return ReleaseInfo(
            version=raw["version"],
            channel=raw["channel"],
            released_at=raw["released_at"],
        )
    except (OSError, ValueError, KeyError):
        return None


def state_dir() -> Path:
    """Return the directory for persistent user state (contacts, history, config).

    Honours YOYOPOD_STATE_DIR (set by the slot launcher on the Pi).
    """
    override = os.environ.get("YOYOPOD_STATE_DIR")
    if override:
        return Path(override)
    return _DEFAULT_STATE_DIR
