#!/usr/bin/env python3
"""Build the pinned LVGL shim for the current platform."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

LVGL_VERSION = "9.5.0"
LVGL_TAG = f"v{LVGL_VERSION}"
LVGL_REPO = "https://github.com/lvgl/lvgl.git"


def run(command: list[str], cwd: Path | None = None) -> None:
    subprocess.run(command, cwd=str(cwd) if cwd else None, check=True)


def has_valid_lvgl_source(source_dir: Path) -> bool:
    required_paths = (
        source_dir / "CMakeLists.txt",
        source_dir / ".git",
        source_dir / ".git" / "index",
        source_dir / "src" / "misc" / "lv_ext_data.h",
    )
    if any(not path.exists() for path in required_paths):
        return False

    status = subprocess.run(
        ["git", "-C", str(source_dir), "status", "--short", "--untracked-files=no"],
        check=False,
        capture_output=True,
        text=True,
    )
    return status.returncode == 0 and not status.stdout.strip()


def ensure_lvgl_source(source_dir: Path) -> None:
    if source_dir.exists():
        if has_valid_lvgl_source(source_dir):
            return
        shutil.rmtree(source_dir)
    source_dir.parent.mkdir(parents=True, exist_ok=True)
    run(["git", "clone", "--depth", "1", "--branch", LVGL_TAG, LVGL_REPO, str(source_dir)])


def build(native_dir: Path, source_dir: Path, build_dir: Path) -> None:
    build_dir.mkdir(parents=True, exist_ok=True)
    run(
        [
            "cmake",
            "-S",
            str(native_dir),
            "-B",
            str(build_dir),
            "-DCMAKE_BUILD_TYPE=Release",
            f"-DLVGL_SOURCE_DIR={source_dir}",
            "-DCONFIG_LV_BUILD_EXAMPLES=OFF",
            "-DCONFIG_LV_BUILD_DEMOS=OFF",
        ]
    )
    run(["cmake", "--build", str(build_dir), "--parallel", "2"])


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    repo_root = Path(__file__).resolve().parents[1]
    native_dir = repo_root / "yoyopy" / "ui" / "lvgl_binding" / "native"
    cache_source = repo_root / ".cache" / "lvgl" / f"lvgl-{LVGL_VERSION}"
    default_build = native_dir / "build"

    parser.add_argument("--source-dir", type=Path, default=cache_source)
    parser.add_argument("--build-dir", type=Path, default=default_build)
    parser.add_argument("--skip-fetch", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    native_dir = repo_root / "yoyopy" / "ui" / "lvgl_binding" / "native"

    if not args.skip_fetch:
        ensure_lvgl_source(args.source_dir)

    build(native_dir, args.source_dir, args.build_dir)
    print(f"Built LVGL shim in {args.build_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
