"""Shared slot-deploy structural contract helpers."""

from __future__ import annotations

from pathlib import Path

SLOT_REQUIRED_DIRS: tuple[str, ...] = ("venv", "app", "config")
SLOT_VENV_PYTHON = Path("venv") / "bin" / "python"

APP_NATIVE_RUNTIME_ARTIFACTS: tuple[Path, ...] = (
    Path("yoyopod") / "ui" / "lvgl_binding" / "native" / "build" / "libyoyopod_lvgl_shim.so",
    Path("yoyopod")
    / "backends"
    / "voip"
    / "shim_native"
    / "build"
    / "libyoyopod_liblinphone_shim.so",
)

SLOT_NATIVE_RUNTIME_ARTIFACTS: tuple[Path, ...] = tuple(
    Path("app") / relative for relative in APP_NATIVE_RUNTIME_ARTIFACTS
)

SELF_CONTAINED_REQUIRED_FILES: tuple[Path, ...] = (
    SLOT_VENV_PYTHON,
    *SLOT_NATIVE_RUNTIME_ARTIFACTS,
)


def missing_self_contained_paths(slot_dir: Path) -> tuple[Path, ...]:
    """Return the slot-relative files a self-contained release still lacks."""

    return tuple(
        relative
        for relative in SELF_CONTAINED_REQUIRED_FILES
        if not (slot_dir / relative).is_file()
    )


def is_self_contained_slot(slot_dir: Path) -> bool:
    """Return True when the slot contains its own runtime Python and native shims."""

    return not missing_self_contained_paths(slot_dir)
