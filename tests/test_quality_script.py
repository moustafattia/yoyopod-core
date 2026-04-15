"""Tests for the repo-owned quality command wrapper."""

from __future__ import annotations

import runpy
import sys
from pathlib import Path

QUALITY_SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "quality.py"
QUALITY = runpy.run_path(str(QUALITY_SCRIPT))


def test_ci_steps_wrap_gate_plus_pytest() -> None:
    config = QUALITY["load_quality_config"]()

    gate_steps = QUALITY["build_gate_steps"](config)
    ci_steps = QUALITY["build_ci_steps"](config)

    assert ci_steps[:-1] == gate_steps
    assert ci_steps[-1].label == "pytest -q (CI test suite)"
    assert ci_steps[-1].command == (sys.executable, "-m", "pytest", "-q")


def test_parser_accepts_ci_command() -> None:
    parser = QUALITY["build_parser"]()

    args = parser.parse_args(["ci"])

    assert args.command == "ci"
