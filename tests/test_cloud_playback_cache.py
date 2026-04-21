from __future__ import annotations

import os
from pathlib import Path

from yoyopod.cloud.playback_cache import RemotePlaybackCache


def test_prepare_sanitizes_track_id_and_stays_within_cache_root(tmp_path: Path) -> None:
    cache = RemotePlaybackCache(tmp_path, 64 * 1024 * 1024)

    def fake_download(*, media_url: str, target_path: Path, checksum_sha256: str | None) -> Path:
        target_path.write_bytes(b"audio")
        return target_path

    cache._download = fake_download  # type: ignore[method-assign]

    asset = cache.prepare(
        track_id="../../evil/nested-track",
        media_url="https://media.example.test/file.mp3",
        checksum_sha256="abc123",
    )

    resolved_root = tmp_path.resolve()
    resolved_path = Path(asset.path).resolve()

    assert resolved_path.parent == resolved_root
    assert ".." not in resolved_path.name
    assert resolved_path.name.startswith("evil-nested-track-abc123")


def test_prune_removes_oldest_files_first(tmp_path: Path) -> None:
    cache = RemotePlaybackCache(tmp_path, 32 * 1024 * 1024)
    cache.max_bytes = 8

    oldest = tmp_path / "oldest.mp3"
    newest = tmp_path / "newest.mp3"
    oldest.write_bytes(b"12345")
    newest.write_bytes(b"67890")

    os.utime(oldest, (100, 100))
    os.utime(newest, (200, 200))

    cache._prune()

    assert not oldest.exists()
    assert newest.exists()
