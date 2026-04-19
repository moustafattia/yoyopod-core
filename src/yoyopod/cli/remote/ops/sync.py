"""Rsync and archive-sync helpers for remote deployments."""

from __future__ import annotations

import fnmatch
import json
import os
import shutil
import tarfile
import tempfile
import sys
from pathlib import Path
from typing import Optional, Sequence

import typer

from yoyopod.cli.common import REPO_ROOT
from yoyopod.cli.remote.config import (
    PiDeployConfig,
    RemoteConfig,
    load_pi_deploy_config,
    resolve_remote_config,
)
from yoyopod.cli.remote.transport import run_local, run_remote, validate_config

from .commands import build_restart_command, build_sync_command


def resolve_local_executable(program: str) -> str | None:
    """Resolve one local executable, including common Windows install paths."""
    resolved = shutil.which(program)
    if resolved:
        return resolved

    if not sys.platform.startswith("win"):
        return None

    candidate_paths: list[Path] = []
    if program.lower() == "rsync":
        candidate_paths.extend(
            [
                Path(r"C:\msys64\usr\bin\rsync.exe"),
                Path(r"C:\Program Files\Git\usr\bin\rsync.exe"),
                Path(r"C:\Program Files\Git\bin\rsync.exe"),
            ]
        )
    elif program.lower() == "scp":
        candidate_paths.extend(
            [
                Path(r"C:\Windows\System32\OpenSSH\scp.exe"),
            ]
        )

    for candidate in candidate_paths:
        if candidate.exists():
            return str(candidate)
    return None


def should_use_direct_rsync(rsync_binary: str | None) -> bool:
    """Return whether the local rsync binary is safe for direct remote sync."""
    if not rsync_binary:
        return False

    force_rsync = os.getenv("YOYOPOD_PI_FORCE_RSYNC", "").strip().lower()
    if force_rsync in {"1", "true", "yes", "on"}:
        return True

    if not sys.platform.startswith("win"):
        return True

    normalized = str(Path(rsync_binary)).replace("/", "\\").lower()
    known_windows_rsyncs = {
        r"c:\msys64\usr\bin\rsync.exe",
        r"c:\program files\git\usr\bin\rsync.exe",
        r"c:\program files\git\bin\rsync.exe",
    }
    return normalized not in known_windows_rsyncs


def sync_path_is_excluded(
    rel_path: str,
    patterns: Sequence[str],
    *,
    is_dir: bool,
) -> bool:
    """Return whether one repo-relative path should be skipped during sync."""
    normalized = rel_path.strip("/")
    if not normalized:
        return False

    segments = normalized.split("/")
    basename = segments[-1]
    for pattern in patterns:
        candidate = str(pattern).strip()
        if not candidate:
            continue

        if candidate.endswith("/"):
            dir_pattern = candidate.rstrip("/")
            if any(fnmatch.fnmatch(segment, dir_pattern) for segment in segments):
                return True
            continue

        if fnmatch.fnmatch(normalized, candidate) or fnmatch.fnmatch(basename, candidate):
            return True

    return False


def build_sync_file_manifest(
    repo_root: Path,
    deploy_config: PiDeployConfig,
) -> list[str]:
    """Collect the repo-relative file list that should be mirrored to the Pi."""
    manifest: list[str] = []
    for current_root, dirnames, filenames in os.walk(repo_root):
        current_root_path = Path(current_root)
        relative_root = current_root_path.relative_to(repo_root)

        kept_dirnames: list[str] = []
        for dirname in sorted(dirnames):
            rel_dir = (relative_root / dirname).as_posix()
            if rel_dir == ".":
                rel_dir = dirname
            if sync_path_is_excluded(rel_dir, deploy_config.rsync_exclude, is_dir=True):
                continue
            kept_dirnames.append(dirname)
        dirnames[:] = kept_dirnames

        for filename in sorted(filenames):
            rel_file = (relative_root / filename).as_posix()
            if rel_file == ".":
                rel_file = filename
            if sync_path_is_excluded(rel_file, deploy_config.rsync_exclude, is_dir=False):
                continue
            manifest.append(rel_file)

    return manifest


def build_rsync_command(
    config: RemoteConfig,
    deploy_config: PiDeployConfig,
    *,
    executable: str = "rsync",
) -> list[str]:
    """Create the local rsync command for a dirty-tree sync."""
    command = [executable, "-avz", "--delete"]
    for pattern in deploy_config.rsync_exclude:
        command.extend(["--exclude", pattern])

    remote_dir = config.project_dir.rstrip("/")
    command.extend(["./", f"{config.ssh_target}:{remote_dir}/"])
    return command


def build_archive_sync_extract_command(
    config: RemoteConfig,
    *,
    archive_path: str,
    manifest_path: str,
) -> str:
    """Create the remote command that unpacks and mirrors an scp-uploaded archive."""
    project_dir_literal = repr(config.project_dir)
    archive_path_literal = repr(archive_path)
    manifest_path_literal = repr(manifest_path)
    return f"""python - <<'PY'
import fnmatch
import json
import os
import tarfile
from pathlib import Path


def is_excluded(rel_path: str, patterns: tuple[str, ...]) -> bool:
    normalized = rel_path.strip("/")
    if not normalized:
        return False

    segments = normalized.split("/")
    basename = segments[-1]
    for pattern in patterns:
        candidate = str(pattern).strip()
        if not candidate:
            continue

        if candidate.endswith("/"):
            dir_pattern = candidate.rstrip("/")
            if any(fnmatch.fnmatch(segment, dir_pattern) for segment in segments):
                return True
            continue

        if fnmatch.fnmatch(normalized, candidate) or fnmatch.fnmatch(basename, candidate):
            return True

    return False


project_dir = Path(os.path.expanduser({project_dir_literal})).resolve()
archive_path = Path({archive_path_literal})
manifest_path = Path({manifest_path_literal})

with manifest_path.open("r", encoding="utf-8") as handle:
    payload = json.load(handle)

expected_files = set(payload["files"])
exclude_patterns = tuple(payload["exclude"])
project_dir.mkdir(parents=True, exist_ok=True)

with tarfile.open(archive_path, "r:gz") as archive:
    members = archive.getmembers()
    for member in members:
        member_path = Path(member.name)
        if member_path.is_absolute() or ".." in member_path.parts:
            raise SystemExit(f"Unsafe archive member: {{member.name}}")
    archive.extractall(project_dir)

for path in sorted(project_dir.rglob("*"), key=lambda item: len(item.parts), reverse=True):
    rel_path = path.relative_to(project_dir).as_posix()
    if is_excluded(rel_path, exclude_patterns):
        continue

    if path.is_file() or path.is_symlink():
        if rel_path not in expected_files:
            path.unlink()
        continue

    try:
        path.rmdir()
    except OSError:
        pass

archive_path.unlink(missing_ok=True)
manifest_path.unlink(missing_ok=True)
PY"""


def run_rsync_deploy(
    config: RemoteConfig,
    deploy_config: PiDeployConfig,
    *,
    skip_restart: bool,
) -> int:
    """Rsync the local working tree to the Pi and optionally restart the app."""
    rsync_binary = resolve_local_executable("rsync")
    if should_use_direct_rsync(rsync_binary):
        if rsync_binary is None:
            raise SystemExit(
                "Direct rsync requested, but the local rsync binary could not be resolved."
            )
        exit_code = run_local(
            build_rsync_command(config, deploy_config, executable=rsync_binary),
            "rsync",
        )
    else:
        scp_binary = resolve_local_executable("scp")
        if not scp_binary:
            raise SystemExit(
                "Neither `rsync` nor `scp` is available locally. "
                "Install one of them or run from a machine with SSH copy tools."
            )

        print("")
        if rsync_binary:
            print(
                "[pi-remote] info=local Windows rsync is not reliable for remote Unix paths; "
                "falling back to archive+scp sync"
            )
        else:
            print("[pi-remote] info=local rsync not found, falling back to archive+scp sync")
        print("")

        manifest = build_sync_file_manifest(REPO_ROOT, deploy_config)
        remote_archive_path = "/tmp/yoyopod_sync.tar.gz"
        remote_manifest_path = "/tmp/yoyopod_sync_manifest.json"

        with tempfile.TemporaryDirectory(prefix="yoyopod-sync-") as temp_dir_raw:
            temp_dir = Path(temp_dir_raw)
            archive_path = temp_dir / "yoyopod_sync.tar.gz"
            manifest_path = temp_dir / "yoyopod_sync_manifest.json"

            with tarfile.open(archive_path, "w:gz") as archive:
                for rel_path in manifest:
                    archive.add(REPO_ROOT / rel_path, arcname=rel_path)

            manifest_payload = {
                "files": manifest,
                "exclude": list(deploy_config.rsync_exclude),
            }
            manifest_path.write_text(
                json.dumps(manifest_payload, indent=2, sort_keys=True),
                encoding="utf-8",
            )

            upload_command = [
                scp_binary,
                str(archive_path),
                str(manifest_path),
                f"{config.ssh_target}:/tmp/",
            ]
            exit_code = run_local(upload_command, "scp-sync-upload")
            if exit_code == 0:
                exit_code = run_remote(
                    config,
                    build_archive_sync_extract_command(
                        config,
                        archive_path=remote_archive_path,
                        manifest_path=remote_manifest_path,
                    ),
                )

    if exit_code != 0 or skip_restart:
        return exit_code

    return run_remote(config, build_restart_command(deploy_config))


def sync(
    host: str = "",
    user: str = "",
    project_dir: str = "",
    branch: str = "",
    sha: Optional[str] = None,
    skip_uv_sync: bool = False,
) -> None:
    """Sync the stable Pi checkout to one committed branch or exact commit."""
    config = resolve_remote_config(host, user, project_dir, branch)
    validate_config(config)
    rc = run_remote(config, build_sync_command(config, skip_uv_sync, target_sha=sha))
    if rc != 0:
        raise typer.Exit(code=rc)


def rsync(
    host: str = "",
    user: str = "",
    project_dir: str = "",
    branch: str = "",
    skip_restart: bool = False,
    verbose: bool = False,
) -> None:
    """Rare-case escape hatch: mirror the local dirty working tree to the Pi."""
    config = resolve_remote_config(host, user, project_dir, branch)
    validate_config(config)
    deploy_config = load_pi_deploy_config()
    print(
        "[pi-remote] warning=dirty-tree rsync is a debugging escape hatch; "
        "prefer `yoyoctl remote validate` for committed branch/SHA validation"
    )
    _ = verbose
    rc = run_rsync_deploy(config, deploy_config, skip_restart=skip_restart)
    if rc != 0:
        raise typer.Exit(code=rc)


__all__ = [
    "build_archive_sync_extract_command",
    "build_rsync_command",
    "build_sync_file_manifest",
    "resolve_local_executable",
    "run_rsync_deploy",
    "should_use_direct_rsync",
    "rsync",
    "sync",
    "sync_path_is_excluded",
]
