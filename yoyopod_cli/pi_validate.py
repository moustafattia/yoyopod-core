"""yoyopod_cli/pi_validate.py — staged Pi validation suite."""

from __future__ import annotations

import os
import platform
import shlex
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Annotated, Any

import typer

from yoyopod_cli.common import REPO_ROOT, configure_logging, resolve_config_dir

if TYPE_CHECKING:
    from yoyopod.audio.test_music import ProvisionedTestMusicLibrary
    from yoyopod.config import MediaConfig

app = typer.Typer(
    name="validate",
    help=(
        "Focused target-side validation suite for deploy, smoke, music, voip, "
        "and navigation stability checks."
    ),
    no_args_is_help=True,
)

# ---------------------------------------------------------------------------
# Shared result type (inlined from yoyopod.cli.pi.smoke)
# ---------------------------------------------------------------------------


@dataclass
class _CheckResult:
    """Result for one validation step."""

    name: str
    status: str
    details: str


def _print_summary(name: str, results: list[_CheckResult]) -> None:
    """Print a compact summary table for one validation command."""
    print("")
    print(f"YoyoPod target validation summary: {name}")
    print("=" * 48)
    for result in results:
        print(f"[{result.status.upper():4}] {result.name}: {result.details}")


# ---------------------------------------------------------------------------
# Deploy helpers (inlined from yoyopod.cli.pi.validate)
# ---------------------------------------------------------------------------


def _resolve_runtime_path(path_value: str) -> Path:
    """Resolve one repo-relative or absolute runtime path."""
    path = Path(path_value)
    if path.is_absolute():
        return path
    return REPO_ROOT / path


def _nearest_existing_parent(path: Path) -> Path:
    """Return the nearest existing parent for one path."""
    candidate = path if path.exists() and path.is_dir() else path.parent
    while not candidate.exists() and candidate != candidate.parent:
        candidate = candidate.parent
    return candidate


def _config_files_check(config_path: Path) -> _CheckResult:
    """Validate that the tracked runtime config files are present."""
    required_files = (
        config_path / "app" / "core.yaml",
        config_path / "audio" / "music.yaml",
        config_path / "device" / "hardware.yaml",
        config_path / "voice" / "assistant.yaml",
        config_path / "communication" / "calling.yaml",
        config_path / "communication" / "messaging.yaml",
        config_path / "communication" / "integrations" / "liblinphone_factory.conf",
        config_path / "people" / "directory.yaml",
        config_path / "people" / "contacts.seed.yaml",
    )
    missing = [str(path.relative_to(REPO_ROOT)) for path in required_files if not path.exists()]
    if missing:
        return _CheckResult(
            name="config",
            status="fail",
            details=f"missing required config files: {', '.join(missing)}",
        )

    return _CheckResult(
        name="config",
        status="pass",
        details=", ".join(str(path.relative_to(REPO_ROOT)) for path in required_files),
    )


def _deploy_contract_check() -> tuple[_CheckResult, Any | None]:
    """Validate that the tracked deploy contract is readable."""
    from yoyopod.cli.remote.config import load_pi_deploy_config

    try:
        deploy_config = load_pi_deploy_config()
    except Exception as exc:
        return _CheckResult(name="deploy_contract", status="fail", details=str(exc)), None

    return (
        _CheckResult(
            name="deploy_contract",
            status="pass",
            details=(
                f"project_dir={deploy_config.project_dir}, "
                f"venv={deploy_config.venv}, "
                f"start_cmd={deploy_config.start_cmd}"
            ),
        ),
        deploy_config,
    )


def _runtime_paths_check(deploy_config: Any) -> _CheckResult:
    """Validate that runtime file parents are reachable and writable."""
    path_map = {
        "log": _resolve_runtime_path(deploy_config.log_file),
        "error_log": _resolve_runtime_path(deploy_config.error_log_file),
        "pid": _resolve_runtime_path(deploy_config.pid_file),
        "screenshot": _resolve_runtime_path(deploy_config.screenshot_path),
    }

    details: list[str] = []
    failures: list[str] = []
    for name, path in path_map.items():
        parent = _nearest_existing_parent(path)
        writable = os.access(parent, os.W_OK)
        details.append(f"{name}_parent={parent}")
        if not writable:
            failures.append(f"{name}_parent_not_writable={parent}")

    if failures:
        return _CheckResult(
            name="runtime_paths",
            status="fail",
            details=", ".join(failures),
        )

    return _CheckResult(
        name="runtime_paths",
        status="pass",
        details=", ".join(details),
    )


def _entrypoint_check(deploy_config: Any) -> _CheckResult:
    """Validate repo entrypoints and the configured virtualenv activation path."""
    required_paths = {
        "app": REPO_ROOT / "yoyopod.py",
        "systemd": REPO_ROOT / "deploy" / "systemd" / "yoyopod@.service",
    }

    normalized_venv = Path(deploy_config.venv.rstrip("/"))
    if not normalized_venv.is_absolute():
        normalized_venv = REPO_ROOT / normalized_venv
    activate_path = (
        normalized_venv
        if normalized_venv.name == "activate"
        else normalized_venv / "bin" / "activate"
    )
    required_paths["venv_activate"] = activate_path

    missing = [
        f"{name}={path.relative_to(REPO_ROOT) if path.is_relative_to(REPO_ROOT) else path}"
        for name, path in required_paths.items()
        if not path.exists()
    ]
    if missing:
        return _CheckResult(
            name="entrypoints",
            status="fail",
            details=f"missing required paths: {', '.join(missing)}",
        )

    start_parts = shlex.split(deploy_config.start_cmd)
    if not start_parts:
        return _CheckResult(
            name="entrypoints",
            status="fail",
            details="start_cmd is empty in deploy/pi-deploy.yaml",
        )

    executable = start_parts[0]
    resolved_executable = shutil.which(executable)
    if resolved_executable is None:
        return _CheckResult(
            name="entrypoints",
            status="fail",
            details=f"configured start executable is not on PATH: {executable}",
        )

    return _CheckResult(
        name="entrypoints",
        status="pass",
        details=(
            f"start_executable={resolved_executable}, "
            f"venv_activate={activate_path.relative_to(REPO_ROOT) if activate_path.is_relative_to(REPO_ROOT) else activate_path}"
        ),
    )


# ---------------------------------------------------------------------------
# Smoke helpers (inlined from yoyopod.cli.pi.smoke)
# ---------------------------------------------------------------------------


def _load_app_config(config_dir: Path) -> dict[str, Any]:
    """Load the composed app config if present."""
    from loguru import logger

    from yoyopod.config import config_to_dict, load_composed_app_settings

    if not any(
        path.exists()
        for path in (
            config_dir / "app" / "core.yaml",
            config_dir / "device" / "hardware.yaml",
        )
    ):
        logger.warning("Composed app config not found under {}", config_dir)
    return config_to_dict(load_composed_app_settings(config_dir))


def _load_media_config(config_dir: Path) -> MediaConfig:
    """Load the typed composed media config if present."""
    from yoyopod.config import ConfigManager

    return ConfigManager(config_dir=str(config_dir)).get_media_settings()


def _environment_check() -> _CheckResult:
    """Capture the current execution environment."""
    system = platform.system()
    machine = platform.machine()
    python_version = platform.python_version()

    if system == "Linux" and ("arm" in machine.lower() or "aarch" in machine.lower()):
        status = "pass"
    else:
        status = "warn"

    return _CheckResult(
        name="environment",
        status=status,
        details=f"system={system}, machine={machine}, python={python_version}",
    )


def _display_check(
    app_config: dict[str, Any],
    hold_seconds: float,
) -> tuple[_CheckResult, object]:
    """Validate display initialization on target hardware."""
    from yoyopod.ui.display import Display, detect_hardware

    requested_hardware = str(app_config.get("display", {}).get("hardware", "auto")).lower()
    resolved_hardware = detect_hardware() if requested_hardware == "auto" else requested_hardware

    if resolved_hardware == "simulation":
        return (
            _CheckResult(
                name="display",
                status="fail",
                details=(
                    "hardware detection resolved to simulation; "
                    "no supported Raspberry Pi display hardware was found"
                ),
            ),
            None,
        )

    display = None
    try:
        display = Display(hardware=resolved_hardware, simulate=False)
        adapter = display.get_adapter()

        display.clear(display.COLOR_BLACK)
        display.text("YoyoPod Pi smoke", 10, 40, color=display.COLOR_WHITE, font_size=18)
        display.text("Display OK", 10, 75, color=display.COLOR_GREEN, font_size=18)
        display.update()

        if hold_seconds > 0:
            time.sleep(hold_seconds)

        if display.simulate:
            return (
                _CheckResult(
                    name="display",
                    status="fail",
                    details=(
                        f"adapter {adapter.__class__.__name__} fell back to simulation "
                        "instead of hardware mode"
                    ),
                ),
                display,
            )

        return (
            _CheckResult(
                name="display",
                status="pass",
                details=(
                    f"adapter={adapter.__class__.__name__}, "
                    f"size={display.WIDTH}x{display.HEIGHT}, "
                    f"orientation={display.ORIENTATION}, "
                    f"requested={requested_hardware}, resolved={resolved_hardware}"
                ),
            ),
            display,
        )
    except Exception as exc:
        if display is not None:
            try:
                display.cleanup()
            except Exception:
                pass
        return _CheckResult(name="display", status="fail", details=str(exc)), None


def _input_check(display: object, app_config: dict[str, Any]) -> _CheckResult:
    """Validate that the matching input adapter can be constructed."""
    from yoyopod.ui.input import get_input_manager

    input_manager = None

    try:
        input_manager = get_input_manager(
            display.get_adapter(),  # type: ignore[union-attr]
            config=app_config,
            simulate=False,
        )
        if input_manager is None:
            return _CheckResult(
                name="input",
                status="fail",
                details="no input adapter was created for the detected display hardware",
            )

        capabilities = sorted(action.value for action in input_manager.get_capabilities())
        interaction_profile = input_manager.interaction_profile.value
        input_manager.start()
        time.sleep(0.1)
        input_manager.stop()

        return _CheckResult(
            name="input",
            status="pass",
            details=(
                f"profile={interaction_profile}, "
                f"capabilities={', '.join(capabilities)}"
            ),
        )
    except Exception as exc:
        return _CheckResult(name="input", status="fail", details=str(exc))
    finally:
        if input_manager is not None:
            try:
                input_manager.stop()
            except Exception:
                pass


def _power_check(config_dir: Path) -> _CheckResult:
    """Validate PiSugar reachability and report a live battery snapshot."""
    from yoyopod.config import ConfigManager
    from yoyopod.power import PowerManager

    config_manager = ConfigManager(config_dir=str(config_dir))
    manager = PowerManager.from_config_manager(config_manager)

    if not manager.config.enabled:
        return _CheckResult(
            name="power",
            status="warn",
            details="power backend disabled in config/power/backend.yaml",
        )

    snapshot = manager.refresh()
    if not snapshot.available:
        details = snapshot.error or "power backend unavailable"
        return _CheckResult(name="power", status="fail", details=details)

    details = ", ".join(
        [
            f"model={snapshot.device.model or 'unknown'}",
            f"battery={snapshot.battery.level_percent:.1f}%" if snapshot.battery.level_percent is not None else "battery=unknown",
            f"charging={snapshot.battery.charging}",
            f"plugged={snapshot.battery.power_plugged}",
        ]
    )
    return _CheckResult(name="power", status="pass", details=details)


def _rtc_check(config_dir: Path) -> _CheckResult:
    """Validate PiSugar RTC reachability and report the current RTC state."""
    from yoyopod.config import ConfigManager
    from yoyopod.power import PowerManager

    config_manager = ConfigManager(config_dir=str(config_dir))
    manager = PowerManager.from_config_manager(config_manager)

    if not manager.config.enabled:
        return _CheckResult(
            name="rtc",
            status="warn",
            details="power backend disabled in config/power/backend.yaml",
        )

    snapshot = manager.refresh()
    if not snapshot.available:
        details = snapshot.error or "power backend unavailable"
        return _CheckResult(name="rtc", status="fail", details=details)

    if snapshot.rtc.time is None:
        return _CheckResult(
            name="rtc",
            status="fail",
            details="PiSugar backend responded but rtc_time is unavailable",
        )

    details = ", ".join(
        [
            f"time={snapshot.rtc.time.isoformat()}",
            f"alarm_enabled={snapshot.rtc.alarm_enabled}",
            f"alarm_time={snapshot.rtc.alarm_time.isoformat() if snapshot.rtc.alarm_time is not None else 'none'}",
            f"repeat_mask={snapshot.rtc.alarm_repeat_mask if snapshot.rtc.alarm_repeat_mask is not None else 'unknown'}",
        ]
    )
    return _CheckResult(name="rtc", status="pass", details=details)


def _music_check(
    media_settings: MediaConfig,
    timeout_seconds: int,
    *,
    expected_library: ProvisionedTestMusicLibrary | None = None,
) -> _CheckResult:
    """Validate music-backend startup and basic state queries."""
    from yoyopod.audio import LocalMusicService, MpvBackend, MusicConfig

    config = MusicConfig.from_media_settings(media_settings)
    backend = MpvBackend(config)
    try:
        started_at = time.monotonic()
        if not backend.start():
            return _CheckResult(
                name="music",
                status="fail",
                details=(
                    "could not start the mpv music backend "
                    f"(binary={config.mpv_binary}, socket={config.mpv_socket})"
                ),
            )

        while not backend.is_connected and (time.monotonic() - started_at) < timeout_seconds:
            time.sleep(0.1)

        if not backend.is_connected:
            return _CheckResult(
                name="music",
                status="fail",
                details=f"music backend did not report ready within {timeout_seconds}s",
            )

        if expected_library is not None:
            missing_assets = [
                path
                for path in expected_library.expected_asset_paths
                if not path.exists()
            ]
            if missing_assets:
                missing_list = ", ".join(str(path) for path in missing_assets)
                return _CheckResult(
                    name="music",
                    status="fail",
                    details=f"missing provisioned test assets: {missing_list}",
                )

            music_service = LocalMusicService(backend, music_dir=expected_library.target_dir)
            playlist_path = expected_library.default_playlist_path
            playlists = music_service.list_playlists()
            if str(playlist_path) not in {playlist.uri for playlist in playlists}:
                return _CheckResult(
                    name="music",
                    status="fail",
                    details=f"provisioned playlist not discoverable under {expected_library.target_dir}",
                )

            if not music_service.load_playlist(str(playlist_path)):
                return _CheckResult(
                    name="music",
                    status="fail",
                    details=f"mpv could not load the provisioned playlist {playlist_path}",
                )

            expected_track_uris = {str(path) for path in expected_library.track_paths}
            loaded_track = None
            while (time.monotonic() - started_at) < timeout_seconds:
                loaded_track = backend.get_current_track()
                if loaded_track is not None and loaded_track.uri in expected_track_uris:
                    break
                time.sleep(0.1)

            if loaded_track is None or loaded_track.uri not in expected_track_uris:
                current_uri = loaded_track.uri if loaded_track is not None else "none"
                return _CheckResult(
                    name="music",
                    status="fail",
                    details=(
                        "music backend started, but it did not load one of the "
                        f"provisioned validation tracks from {expected_library.target_dir}; "
                        f"current_track={current_uri}"
                    ),
                )

            playback_state = backend.get_playback_state()
            return _CheckResult(
                name="music",
                status="pass",
                details=(
                    f"binary={config.mpv_binary}, socket={config.mpv_socket}, "
                    f"music_dir={expected_library.target_dir}, "
                    f"playlist={playlist_path.name}, state={playback_state}, "
                    f"track={loaded_track.name}"
                ),
            )

        playback_state = backend.get_playback_state()
        track = backend.get_current_track()
        track_name = track.name if track else "none"

        return _CheckResult(
            name="music",
            status="pass",
            details=(
                f"binary={config.mpv_binary}, socket={config.mpv_socket}, "
                f"state={playback_state}, track={track_name}"
            ),
        )
    except Exception as exc:
        return _CheckResult(name="music", status="fail", details=str(exc))
    finally:
        backend.stop()


def _voip_check(config_dir: Path, registration_timeout: float) -> _CheckResult:
    """Validate Liblinphone startup and SIP registration."""
    from yoyopod.communication import VoIPConfig, VoIPManager
    from yoyopod.communication.integrations.liblinphone_binding import LiblinphoneBinding
    from yoyopod.config import ConfigManager

    config_manager = ConfigManager(config_dir=str(config_dir))
    voip_config = VoIPConfig.from_config_manager(config_manager)
    binding = LiblinphoneBinding.try_load()
    if binding is None:
        return _CheckResult(
            name="voip",
            status="fail",
            details="Liblinphone shim is unavailable; run yoyoctl build liblinphone on the Pi",
        )

    if not voip_config.sip_identity:
        return _CheckResult(
            name="voip",
            status="fail",
            details="sip_identity is empty in config/communication/calling.yaml",
        )

    manager = VoIPManager(
        voip_config,
        people_directory=None,
    )
    try:
        if not manager.start():
            return _CheckResult(
                name="voip",
                status="fail",
                details="VoIP manager failed to start",
            )

        deadline = time.time() + registration_timeout
        last_status = manager.get_status()

        while time.time() < deadline:
            manager.iterate()
            last_status = manager.get_status()
            if last_status["registered"]:
                return _CheckResult(
                    name="voip",
                    status="pass",
                    details=(
                        f"registered={last_status['registered']}, "
                        f"state={last_status['registration_state']}, "
                        f"identity={last_status['sip_identity']}"
                    ),
                )

            if last_status["registration_state"] == "failed":
                break

            time.sleep(0.5)

        return _CheckResult(
            name="voip",
            status="fail",
            details=(
                f"registration timed out or failed; "
                f"state={last_status['registration_state']}, "
                f"identity={last_status['sip_identity']}"
            ),
        )
    except Exception as exc:
        return _CheckResult(name="voip", status="fail", details=str(exc))
    finally:
        manager.stop()


# ---------------------------------------------------------------------------
# Subcommands
# ---------------------------------------------------------------------------


@app.command()
def deploy(
    config_dir: Annotated[
        str, typer.Option("--config-dir", help="Configuration directory to validate.")
    ] = "config",
    verbose: Annotated[bool, typer.Option("--verbose", help="Enable DEBUG logging.")] = False,
) -> None:
    """Validate deploy-readiness for the current target checkout without launching the app."""
    from loguru import logger

    configure_logging(verbose)
    config_path = resolve_config_dir(config_dir)

    logger.info("Running target deploy validation")

    deploy_result, deploy_config = _deploy_contract_check()
    results = [deploy_result, _config_files_check(config_path)]
    if deploy_config is not None:
        results.append(_runtime_paths_check(deploy_config))
        results.append(_entrypoint_check(deploy_config))

    _print_summary("deploy", results)
    if any(result.status == "fail" for result in results):
        raise typer.Exit(code=1)


@app.command()
def smoke(
    config_dir: Annotated[
        str, typer.Option("--config-dir", help="Configuration directory to use.")
    ] = "config",
    with_power: Annotated[
        bool, typer.Option("--with-power", help="Also validate PiSugar power telemetry.")
    ] = False,
    with_rtc: Annotated[
        bool, typer.Option("--with-rtc", help="Also validate PiSugar RTC state and alarm.")
    ] = False,
    display_hold_seconds: Annotated[
        float,
        typer.Option(
            "--display-hold-seconds",
            help="How long to keep the display confirmation text visible.",
        ),
    ] = 0.5,
    verbose: Annotated[bool, typer.Option("--verbose", help="Enable DEBUG logging.")] = False,
) -> None:
    """Validate core target hardware paths: environment, display, input, and optional PiSugar state."""
    from loguru import logger

    configure_logging(verbose)
    config_path = resolve_config_dir(config_dir)

    logger.info("Running target smoke validation")

    app_config = _load_app_config(config_path)
    results: list[_CheckResult] = [_environment_check()]
    display = None

    try:
        display_result, display = _display_check(app_config, display_hold_seconds)
        results.append(display_result)

        if display_result.status == "pass" and display is not None:
            results.append(_input_check(display, app_config))

        if with_power:
            results.append(_power_check(config_path))

        if with_rtc:
            results.append(_rtc_check(config_path))
    finally:
        if display is not None:
            try:
                display.cleanup()
            except Exception as exc:
                logger.warning(f"Display cleanup failed: {exc}")

    _print_summary("smoke", results)
    if any(result.status == "fail" for result in results):
        raise typer.Exit(code=1)


@app.command()
def music(
    config_dir: Annotated[
        str, typer.Option("--config-dir", help="Configuration directory to use.")
    ] = "config",
    timeout: Annotated[
        int, typer.Option("--timeout", help="Startup timeout in seconds for the music backend.")
    ] = 5,
    verbose: Annotated[bool, typer.Option("--verbose", help="Enable DEBUG logging.")] = False,
) -> None:
    """Validate the mpv music backend on the target without starting the full app."""
    from loguru import logger

    configure_logging(verbose)
    config_path = resolve_config_dir(config_dir)

    logger.info("Running target music validation")

    media_config = _load_media_config(config_path)
    results = [_music_check(media_config, timeout)]

    _print_summary("music", results)
    if any(result.status == "fail" for result in results):
        raise typer.Exit(code=1)


@app.command()
def voip(
    config_dir: Annotated[
        str, typer.Option("--config-dir", help="Configuration directory to use.")
    ] = "config",
    timeout: Annotated[
        float, typer.Option("--timeout", help="Registration timeout in seconds.")
    ] = 90.0,
    verbose: Annotated[bool, typer.Option("--verbose", help="Enable DEBUG logging.")] = False,
) -> None:
    """Validate Liblinphone startup and SIP registration on the target."""
    from loguru import logger

    configure_logging(verbose)
    config_path = resolve_config_dir(config_dir)

    logger.info("Running target VoIP validation")

    results = [_voip_check(config_path, timeout)]

    _print_summary("voip", results)
    if any(result.status == "fail" for result in results):
        raise typer.Exit(code=1)


@app.command()
def stability(
    config_dir: Annotated[
        str, typer.Option("--config-dir", help="Configuration directory to use.")
    ] = "config",
    cycles: Annotated[
        int, typer.Option("--cycles", help="How many full transition cycles to run.")
    ] = 2,
    hold_seconds: Annotated[
        float,
        typer.Option("--hold-seconds", help="How long to keep each screen active during the soak."),
    ] = 0.2,
    idle_seconds: Annotated[
        float,
        typer.Option("--idle-seconds", help="How long to idle after each full navigation cycle."),
    ] = 1.0,
    with_music: Annotated[
        bool,
        typer.Option(
            "--with-music",
            help="Also exercise playlist loading and now-playing actions during the soak.",
        ),
    ] = False,
    provision_test_music: Annotated[
        bool,
        typer.Option(
            "--provision-test-music/--no-provision-test-music",
            help="Seed deterministic validation music before playback soak steps.",
        ),
    ] = True,
    test_music_dir: Annotated[
        str,
        typer.Option(
            "--test-music-dir",
            help="Dedicated target directory for validation-only test music assets.",
        ),
    ] = "",
    skip_sleep: Annotated[
        bool, typer.Option("--skip-sleep", help="Skip the sleep and wake exercise.")
    ] = False,
    verbose: Annotated[bool, typer.Option("--verbose", help="Enable DEBUG logging.")] = False,
) -> None:
    """Run a repeated navigation and idle stability pass on the target checkout."""
    from yoyopod.audio.test_music import DEFAULT_TEST_MUSIC_TARGET_DIR
    from yoyopod.cli.pi.stability import NavigationSoakError, run_navigation_idle_soak

    configure_logging(verbose)
    resolved_music_dir = test_music_dir or DEFAULT_TEST_MUSIC_TARGET_DIR
    try:
        report = run_navigation_idle_soak(
            config_dir=config_dir,
            simulate=False,
            cycles=cycles,
            hold_seconds=hold_seconds,
            idle_seconds=idle_seconds,
            skip_sleep=skip_sleep,
            with_music=with_music,
            provision_test_music=provision_test_music,
            test_music_dir=resolved_music_dir,
        )
    except NavigationSoakError as exc:
        from loguru import logger

        logger.error(f"Stability soak failed: {exc}")
        raise typer.Exit(code=1)

    from loguru import logger

    logger.info(f"Stability soak passed: {report.summary()}")


@app.command()
def navigation(
    config_dir: Annotated[
        str, typer.Option("--config-dir", help="Configuration directory to use.")
    ] = "config",
    cycles: Annotated[
        int, typer.Option("--cycles", help="How many full navigation cycles to run.")
    ] = 2,
    hold_seconds: Annotated[
        float,
        typer.Option(
            "--hold-seconds",
            help="How long to pump after each simulated click or route change.",
        ),
    ] = 0.35,
    idle_seconds: Annotated[
        float,
        typer.Option(
            "--idle-seconds",
            help="How long to leave each exercised screen idle before the next action.",
        ),
    ] = 3.0,
    tail_idle_seconds: Annotated[
        float,
        typer.Option(
            "--tail-idle-seconds",
            help="Final idle dwell on the hub after all navigation cycles complete.",
        ),
    ] = 10.0,
    with_playback: Annotated[
        bool,
        typer.Option(
            "--with-playback/--no-with-playback",
            help="Drive playlist and shuffle playback paths during the soak.",
        ),
    ] = True,
    provision_test_music: Annotated[
        bool,
        typer.Option(
            "--provision-test-music/--no-provision-test-music",
            help="Seed deterministic validation music before playback-driven navigation.",
        ),
    ] = True,
    test_music_dir: Annotated[
        str,
        typer.Option(
            "--test-music-dir",
            help="Dedicated target directory for validation-only test music assets.",
        ),
    ] = "",
    skip_sleep: Annotated[
        bool, typer.Option("--skip-sleep", help="Skip the final sleep/wake exercise.")
    ] = False,
    verbose: Annotated[bool, typer.Option("--verbose", help="Enable DEBUG logging.")] = False,
) -> None:
    """Run the one-button target navigation and idle stability soak on LVGL hardware."""
    from loguru import logger

    from yoyopod.audio.test_music import DEFAULT_TEST_MUSIC_TARGET_DIR
    from yoyopod.cli.pi.navigation import run_navigation_soak

    configure_logging(verbose)
    resolved_music_dir = test_music_dir or DEFAULT_TEST_MUSIC_TARGET_DIR

    ok, details = run_navigation_soak(
        config_dir=config_dir,
        cycles=cycles,
        hold_seconds=hold_seconds,
        idle_seconds=idle_seconds,
        tail_idle_seconds=tail_idle_seconds,
        with_playback=with_playback,
        provision_test_music=provision_test_music,
        test_music_dir=resolved_music_dir,
        skip_sleep=skip_sleep,
    )
    if ok:
        logger.info("Navigation soak passed: {}", details)
        return

    logger.error("Navigation soak failed: {}", details)
    raise typer.Exit(code=1)
