"""Compatibility shim for :mod:`yoyopod.cli.remote.ops`.

This package preserves the previous public import surface of the historical
monolithic ``ops.py`` module while moving implementation into
``ops/*.py`` submodules.
"""

from __future__ import annotations

# Re-export these module objects so legacy monkeypatch paths like
# ``yoyopod.cli.remote.ops.subprocess.run`` continue to work.
import shutil
import subprocess
import sys
from argparse import Namespace
from yoyopod.cli.remote.config import (
    DEPLOY_CONFIG_PATH,
    DEFAULT_PI_PROJECT_DIR,
    LOCAL_DEPLOY_CONFIG_PATH,
    PiDeployConfig,
    RemoteConfig,
    load_pi_deploy_config,
    pi_deploy_config_to_dict,
    resolve_remote_config,
)
from yoyopod.cli.remote.transport import (
    quote_remote_project_dir,
    run_local,
    run_local_capture,
    run_remote,
    run_remote_capture,
    shell_quote,
    validate_config,
)

from .commands import (
    build_deploy_validation_command,
    build_logs_command,
    build_local_preflight_commands,
    build_native_shim_refresh_command,
    build_provision_test_music_command,
    build_restart_command,
    build_rtc_command,
    build_smoke_command,
    build_startup_verification_command,
    build_status_command,
    build_sync_command,
    build_validation_inspection_command,
    build_whisplay_command,
)
from .lifecycle import logs, restart, rtc, status, whisplay
from .parser import build_parser
from .screenshot import run_screenshot as _run_screenshot, screenshot
from .sync import (
    build_archive_sync_extract_command,
    build_rsync_command,
    build_sync_file_manifest,
    resolve_local_executable,
    run_rsync_deploy,
    should_use_direct_rsync,
    rsync,
    sync,
    sync_path_is_excluded,
)
from .validation import (
    _capture_local_git,
    _resolve_remote_config,
    remote_preflight,
    remote_provision_test_music,
    remote_smoke,
    remote_validate,
    resolve_local_validation_target,
)


def run_screenshot(
    config: RemoteConfig,
    deploy_config: PiDeployConfig,
    args: Namespace,
) -> int:
    """Capture a screenshot from the remote app and copy it locally.

    This wrapper keeps monkeypatching on ``yoyopod.cli.remote.ops`` working for
    both ``run_remote_capture`` and ``subprocess.run``.
    """

    return _run_screenshot(
        config,
        deploy_config,
        args,
        run_remote_capture_fn=run_remote_capture,
        subprocess_run_fn=subprocess.run,
    )


__all__ = [
    "DEFAULT_PI_PROJECT_DIR",
    "DEPLOY_CONFIG_PATH",
    "LOCAL_DEPLOY_CONFIG_PATH",
    "PiDeployConfig",
    "RemoteConfig",
    "_resolve_remote_config",
    "build_archive_sync_extract_command",
    "build_deploy_validation_command",
    "build_local_preflight_commands",
    "build_logs_command",
    "build_native_shim_refresh_command",
    "build_parser",
    "build_provision_test_music_command",
    "build_restart_command",
    "build_rtc_command",
    "build_rsync_command",
    "build_smoke_command",
    "build_startup_verification_command",
    "build_status_command",
    "build_sync_command",
    "build_sync_file_manifest",
    "build_validation_inspection_command",
    "build_whisplay_command",
    "load_pi_deploy_config",
    "pi_deploy_config_to_dict",
    "quote_remote_project_dir",
    "remote_preflight",
    "remote_provision_test_music",
    "remote_smoke",
    "remote_validate",
    "resolve_local_executable",
    "resolve_local_validation_target",
    "resolve_remote_config",
    "run_local",
    "run_local_capture",
    "run_remote",
    "run_remote_capture",
    "run_rsync_deploy",
    "run_screenshot",
    "screenshot",
    "rsync",
    "should_use_direct_rsync",
    "shell_quote",
    "shutil",
    "status",
    "sync",
    "sync_path_is_excluded",
    "sys",
    "validate_config",
    "whisplay",
    "restart",
    "logs",
    "rtc",
    "subprocess",
    "_capture_local_git",
]
