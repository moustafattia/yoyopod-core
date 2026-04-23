"""Shared slot-deploy structural contract helpers."""

from __future__ import annotations

from pathlib import Path

SLOT_REQUIRED_DIRS: tuple[str, ...] = ("venv", "app", "config")
SLOT_VENV_PYTHON = Path("venv") / "bin" / "python"
SLOT_PYTHON_BIN = Path("python") / "bin" / "python3.12"
SLOT_PYTHON_STDLIB_MARKER = Path("python") / "lib" / "python3.12" / "os.py"

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
    SLOT_PYTHON_BIN,
    SLOT_PYTHON_STDLIB_MARKER,
    *SLOT_NATIVE_RUNTIME_ARTIFACTS,
)


def missing_self_contained_paths(slot_dir: Path) -> tuple[Path, ...]:
    """Return the slot-relative files a self-contained release still lacks."""

    # The launch interpreter must be an actual file inside the slot. A venv
    # symlink to the build host's Python can pass Path.is_file(), but is not
    # portable to the Pi.
    venv_python = slot_dir / SLOT_VENV_PYTHON
    if not venv_python.is_file() or venv_python.is_symlink():
        return (
            SLOT_VENV_PYTHON,
            *(
                relative
                for relative in SELF_CONTAINED_REQUIRED_FILES
                if relative != SLOT_VENV_PYTHON
                if not (slot_dir / relative).is_file()
            ),
        )

    return tuple(
        relative
        for relative in SELF_CONTAINED_REQUIRED_FILES
        if relative != SLOT_VENV_PYTHON
        if not (slot_dir / relative).is_file()
    )


def is_self_contained_slot(slot_dir: Path) -> bool:
    """Return True when the slot contains its own runtime Python and native shims."""

    return not missing_self_contained_paths(slot_dir)
