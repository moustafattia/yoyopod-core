# Phase B — Plan B1: Display HAL Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move display adapters and LVGL binding under `src/yoyopod/backends/display/`; create `src/yoyopod/integrations/display/` to own adapter selection and the UI tick callback. Delete `src/yoyopod/ui/display/` and `src/yoyopod/ui/lvgl_binding/`.

**Architecture:** Display integration's `setup(app)` constructs the correct backend from env/config, binds it as `app.display`, and sets `app._ui_tick_callback` so the main loop pumps it. Screens stay unchanged on the consumer side — they already took `app` in Phase A Plan 7 and called `app.display.render(...)`.

**Tech Stack:** Python 3.12+, pytest, uv, existing cffi + LVGL C shim. No new runtime dependencies.

**Spec reference:** `docs/superpowers/specs/2026-04-21-phase-b-hal-consolidation-design.md` §4.1, §5, §6.

**Prerequisite:** Phase A complete and merged to main. Branch `arch/phase-b-hal-consolidation` off main.

---

## File Structure

### Files to create

- `src/yoyopod/backends/display/__init__.py`
- `src/yoyopod/backends/display/api.py` — common display API (`DisplayBackend` protocol)
- `src/yoyopod/backends/display/pimoroni.py` (moved)
- `src/yoyopod/backends/display/whisplay.py` (moved, simplified)
- `src/yoyopod/backends/display/simulation.py` (moved)
- `src/yoyopod/backends/display/lvgl/__init__.py`
- `src/yoyopod/backends/display/lvgl/binding.py` (moved from `ui/lvgl_binding/binding.py`)
- `src/yoyopod/backends/display/lvgl/backend.py` (moved from `ui/lvgl_binding/backend.py`)
- `src/yoyopod/backends/display/lvgl/native_shim/` (moved)
- `src/yoyopod/integrations/display/__init__.py`
- `src/yoyopod/integrations/display/commands.py`
- `tests/backends/test_display_api.py`
- `tests/integrations/test_display.py`

### Files to delete

- `src/yoyopod/ui/display/` (entire directory)
- `src/yoyopod/ui/lvgl_binding/` (entire directory)

---

## Task 1: Branch setup and build validation

- [ ] **Step 1.1: Create Phase B branch**

```bash
git checkout main
git pull origin main
git checkout -b arch/phase-b-hal-consolidation
```

- [ ] **Step 1.2: Validate current LVGL build**

```bash
yoyopod build lvgl
```

Expected: succeeds, producing the native shim `.so` in its current location. We need this baseline so we can detect if moving the source breaks the build.

---

## Task 2: Move LVGL binding and native shim

- [ ] **Step 2.1: Move files**

```bash
mkdir -p src/yoyopod/backends/display/lvgl
git mv src/yoyopod/ui/lvgl_binding/binding.py src/yoyopod/backends/display/lvgl/binding.py
git mv src/yoyopod/ui/lvgl_binding/backend.py src/yoyopod/backends/display/lvgl/backend.py 2>/dev/null || true

# Move native shim directory (name may be 'native' or 'native_shim' — check)
git mv src/yoyopod/ui/lvgl_binding/native src/yoyopod/backends/display/lvgl/native_shim
```

- [ ] **Step 2.2: Create `src/yoyopod/backends/display/lvgl/__init__.py`**

```python
"""LVGL native binding (confined to this package)."""

from __future__ import annotations

from yoyopod.backends.display.lvgl.binding import LvglBinding
from yoyopod.backends.display.lvgl.backend import LvglDisplayBackend

__all__ = ["LvglBinding", "LvglDisplayBackend"]
```

- [ ] **Step 2.3: Update the LVGL build script**

Open `src/yoyopod/cli/build.py` (or wherever `yoyopod build lvgl` is defined). Update the path it expects the native source at — from `src/yoyopod/ui/lvgl_binding/native/` to `src/yoyopod/backends/display/lvgl/native_shim/`.

- [ ] **Step 2.4: Re-run the LVGL build**

```bash
yoyopod build lvgl
```

Expected: succeeds at the new path.

- [ ] **Step 2.5: Update imports inside the moved files**

```bash
grep -rn "from yoyopod.ui.lvgl_binding" src/yoyopod/backends/display/lvgl/
```

Rewrite to `from yoyopod.backends.display.lvgl`. Also check for relative imports inside the binding that reference the native shim path.

- [ ] **Step 2.6: Commit**

```bash
git add -A
git commit -m "refactor(display): relocate LVGL binding + native shim under backends/display/lvgl/

Build script updated to the new native-source path.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Move display adapters under `backends/display/`

- [ ] **Step 3.1: Move facade into a common API module**

```bash
git mv src/yoyopod/ui/display/facade.py src/yoyopod/backends/display/api.py
# If contracts.py exists separately, fold it into api.py manually; then:
[ -f src/yoyopod/ui/display/contracts.py ] && git rm src/yoyopod/ui/display/contracts.py
```

Open `src/yoyopod/backends/display/api.py` and verify it defines a clean `DisplayBackend` abstract class (using `abc.ABC` or `typing.Protocol`) with the methods screens actually call:

```python
class DisplayBackend(Protocol):
    """Common API every display backend implements."""

    backend_kind: str

    def initialize(self) -> None: ...
    def shutdown(self) -> None: ...
    def render(self, canvas: Any) -> None: ...
    def capture_screenshot(self, path: str) -> None: ...
    def tick(self) -> None: ...
    def set_brightness(self, percent: int) -> None: ...
```

Add any helpers the current code needs. Remove any adapter-specific quirks that leaked into the facade — those belong in the adapter.

- [ ] **Step 3.2: Move adapters**

```bash
git mv src/yoyopod/ui/display/adapters/pimoroni.py src/yoyopod/backends/display/pimoroni.py
git mv src/yoyopod/ui/display/adapters/whisplay.py src/yoyopod/backends/display/whisplay.py
git mv src/yoyopod/ui/display/adapters/simulation.py src/yoyopod/backends/display/simulation.py
```

- [ ] **Step 3.3: Create `src/yoyopod/backends/display/__init__.py`**

```python
"""Display backend adapters."""

from __future__ import annotations

from yoyopod.backends.display.api import DisplayBackend
from yoyopod.backends.display.pimoroni import PimoroniDisplayBackend
from yoyopod.backends.display.simulation import SimulationDisplayBackend
from yoyopod.backends.display.whisplay import WhisplayDisplayBackend

__all__ = [
    "DisplayBackend",
    "PimoroniDisplayBackend",
    "SimulationDisplayBackend",
    "WhisplayDisplayBackend",
]
```

- [ ] **Step 3.4: Update imports in moved adapters**

```bash
grep -rn "from yoyopod.ui.display\|from yoyopod.ui.lvgl_binding" src/yoyopod/backends/display/
```

Rewrite to `yoyopod.backends.display` / `yoyopod.backends.display.lvgl`.

- [ ] **Step 3.5: Rename adapter classes if appropriate**

Current class names may be `PimoroniAdapter`, `WhisplayAdapter`, `SimulationAdapter`. Rename to `PimoroniDisplayBackend` / `WhisplayDisplayBackend` / `SimulationDisplayBackend` to match the new pattern. Update all references.

```bash
grep -rn "PimoroniAdapter\|WhisplayAdapter\|SimulationAdapter" src/ tests/
```

Rewrite each match.

- [ ] **Step 3.6: Delete `src/yoyopod/ui/display/factory.py`**

```bash
git rm src/yoyopod/ui/display/factory.py
```

Any existing consumers of `get_display_adapter()` / similar factory function will be updated in Task 4 (integration takes over adapter selection).

- [ ] **Step 3.7: Delete remaining `ui/display/` shell**

```bash
ls src/yoyopod/ui/display/
```

Whatever remains (empty `__init__.py`, residual files) — delete:

```bash
git rm -r src/yoyopod/ui/display/
```

- [ ] **Step 3.8: CI gate**

```bash
uv run python scripts/quality.py ci
```

Expected: any remaining `from yoyopod.ui.display` imports in `src/yoyopod/app.py`, `src/yoyopod/cli/*`, etc., fail type-check or import. Fix inline.

- [ ] **Step 3.9: Commit**

```bash
git add -A
git commit -m "refactor(display): move adapters under backends/display/; delete ui/display/

Pimoroni, Whisplay, Simulation adapters now in backends/display/.
ui/display/factory.py deleted — adapter selection moves into the
display integration in the next task. Common API defined in
backends/display/api.py as a Protocol.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Create `integrations/display/`

- [ ] **Step 4.1: Create `src/yoyopod/integrations/display/commands.py`**

```python
"""Typed commands for the display integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class CaptureScreenshotCommand:
    """Write a PNG screenshot of the current display to `path`."""

    path: str


@dataclass(frozen=True, slots=True)
class RefreshCommand:
    """Force a re-render of the current screen."""


@dataclass(frozen=True, slots=True)
class SetBrightnessOverrideCommand:
    """Override display brightness (overrides the screen integration's value)."""

    percent: int
```

- [ ] **Step 4.2: Create `src/yoyopod/integrations/display/__init__.py`**

```python
"""Display integration: chooses the right backend, wires ui_tick pump."""

from __future__ import annotations

import os
from typing import Any

from loguru import logger

from yoyopod.integrations.display.commands import (
    CaptureScreenshotCommand,
    RefreshCommand,
    SetBrightnessOverrideCommand,
)


_STATE_KEY = "_display_integration"


def _select_backend_class():
    backend_env = os.environ.get("YOYOPOD_DISPLAY", "").lower()
    whisplay_driver = os.environ.get("YOYOPOD_WHISPLAY_DRIVER", "lvgl").lower()

    if backend_env == "simulation":
        from yoyopod.backends.display import SimulationDisplayBackend
        return SimulationDisplayBackend

    if backend_env == "pimoroni":
        from yoyopod.backends.display import PimoroniDisplayBackend
        return PimoroniDisplayBackend

    if backend_env == "whisplay":
        from yoyopod.backends.display import WhisplayDisplayBackend
        return WhisplayDisplayBackend

    # No override: fall back to the per-hardware default from config.
    return None


def setup(app: Any, backend: Any | None = None) -> None:
    if backend is None:
        cls = _select_backend_class()
        if cls is None:
            # Use config fallback — hardware config names which class to use.
            hardware = getattr(app.config, "device", None) or getattr(app.config, "hardware", None)
            name = getattr(hardware, "display", "simulation") if hardware else "simulation"
            from yoyopod.backends.display import (
                PimoroniDisplayBackend,
                SimulationDisplayBackend,
                WhisplayDisplayBackend,
            )
            cls = {
                "pimoroni": PimoroniDisplayBackend,
                "whisplay": WhisplayDisplayBackend,
                "simulation": SimulationDisplayBackend,
            }.get(name, SimulationDisplayBackend)
        backend = cls(app.config)

    backend.initialize()
    app.display = backend

    # Pump the display on each UI tick. If another integration (e.g. screen) has
    # already registered a ui_tick_callback, chain them.
    previous_tick = getattr(app, "_ui_tick_callback", None)

    def ui_tick() -> None:
        try:
            backend.tick()
        except Exception as exc:
            logger.error("Display.tick failed: {}", exc)
        if previous_tick is not None:
            previous_tick()

    app._ui_tick_callback = ui_tick

    # Commands.
    def handle_capture_screenshot(cmd: CaptureScreenshotCommand) -> None:
        backend.capture_screenshot(cmd.path)

    def handle_refresh(_cmd: RefreshCommand) -> None:
        # The screen manager will know to re-render on next tick; we publish a
        # hint so screens can invalidate manually if they want.
        pass

    def handle_set_brightness_override(cmd: SetBrightnessOverrideCommand) -> None:
        backend.set_brightness(cmd.percent)

    app.services.register("display", "capture_screenshot", handle_capture_screenshot)
    app.services.register("display", "refresh", handle_refresh)
    app.services.register("display", "set_brightness_override", handle_set_brightness_override)

    setattr(app, _STATE_KEY, {"backend": backend})


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    try:
        state["backend"].shutdown()
    except Exception as exc:
        logger.error("Display.shutdown: {}", exc)
    if hasattr(app, "display"):
        delattr(app, "display")
    delattr(app, _STATE_KEY)
```

- [ ] **Step 4.3: Register the display integration in `src/yoyopod/app.py`**

Insert `"yoyopod.integrations.display"` in `INTEGRATION_MODULES`. Put it **before** `screen` so `app._ui_tick_callback` chain order is correct (display tick runs first, then screen's idle-timeout check).

- [ ] **Step 4.4: Create `tests/integrations/test_display.py`**

```python
from dataclasses import dataclass

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.display import setup as setup_display, teardown as teardown_display
from yoyopod.integrations.display.commands import (
    CaptureScreenshotCommand,
    SetBrightnessOverrideCommand,
)


@dataclass
class _FakeDisplayBackend:
    backend_kind: str = "fake"
    initialized: bool = False
    shutdown_count: int = 0
    ticks: int = 0
    renders: int = 0
    screenshots: list[str] = None
    brightness_calls: list[int] = None

    def __post_init__(self):
        self.screenshots = []
        self.brightness_calls = []

    def initialize(self):
        self.initialized = True

    def shutdown(self):
        self.shutdown_count += 1

    def tick(self):
        self.ticks += 1

    def render(self, canvas):
        self.renders += 1

    def capture_screenshot(self, path):
        self.screenshots.append(path)

    def set_brightness(self, percent):
        self.brightness_calls.append(percent)


@pytest.fixture
def app_with_display():
    app = build_test_app()
    backend = _FakeDisplayBackend()
    app.register_integration(
        "display",
        setup=lambda a: setup_display(a, backend=backend),
        teardown=lambda a: teardown_display(a),
    )
    app.setup()
    yield app, backend
    app.stop()


def test_setup_binds_app_display(app_with_display):
    app, backend = app_with_display
    assert app.display is backend
    assert backend.initialized is True


def test_ui_tick_callback_pumps_backend(app_with_display):
    app, backend = app_with_display
    app._ui_tick_callback()
    app._ui_tick_callback()
    assert backend.ticks == 2


def test_capture_screenshot_command(app_with_display):
    app, backend = app_with_display
    app.services.call("display", "capture_screenshot", CaptureScreenshotCommand(path="/tmp/x.png"))
    assert backend.screenshots == ["/tmp/x.png"]


def test_set_brightness_override_command(app_with_display):
    app, backend = app_with_display
    app.services.call("display", "set_brightness_override", SetBrightnessOverrideCommand(percent=50))
    assert backend.brightness_calls == [50]


def test_teardown_shuts_down_backend(app_with_display):
    app, backend = app_with_display
    app.stop()
    assert backend.shutdown_count == 1
    assert not hasattr(app, "display")
```

- [ ] **Step 4.5: Run, format, commit**

```bash
uv run pytest tests/integrations/test_display.py -v
uv run black src/yoyopod/integrations/display/ tests/integrations/test_display.py
uv run ruff check src/yoyopod/integrations/display/ tests/integrations/test_display.py
uv run mypy src/yoyopod/integrations/display/
git add -A
git commit -m "feat(integrations/display): adapter selection + ui_tick wiring + commands

setup() picks Pimoroni/Whisplay/Simulation backend from env/config,
binds as app.display, registers ui_tick callback that pumps the
backend. Commands: capture_screenshot, refresh, set_brightness_override.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Update screens to use the new `app.display`

Most screens were migrated in Phase A Plan 7 to take `app` and use `app.display`. Any lingering `ui/display` imports in screens need to be fixed.

- [ ] **Step 5.1: Search for stragglers**

```bash
grep -rn "from yoyopod.ui.display\|from yoyopod.ui.lvgl_binding" src/yoyopod/ui/screens/ src/yoyopod/app.py
```

Rewrite to `from yoyopod.backends.display` / `from yoyopod.backends.display.lvgl` as needed.

- [ ] **Step 5.2: Full CI gate**

```bash
uv run python scripts/quality.py ci
```

- [ ] **Step 5.3: Commit fixups**

```bash
git add -A
git commit -m "refactor(ui): update screen imports to yoyopod.backends.display

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: On-hardware validation

- [ ] **Step 6.1: Simulation smoke**

```bash
python yoyopod.py --simulate
```

Navigate to home screen; confirm it renders. Exit cleanly.

- [ ] **Step 6.2: Pi on-hardware**

```bash
yoyopod pi validate deploy
yoyopod pi validate smoke
yoyopod pi validate lvgl-soak
```

Expected: green on both Pimoroni and Whisplay.

- [ ] **Step 6.3: Screenshot check**

```bash
yoyopod pi screenshot --readback --output /tmp/home.png
```

Inspect `/tmp/home.png` — should match pre-Phase-B baseline visually (pixel-perfect comparison not required; structural/visual parity is the goal).

---

## Task 7: Final sweep

- [ ] **Step 7.1: Confirm no `ui/display` or `ui/lvgl_binding` references linger**

```bash
git grep -l "yoyopod.ui.display\|yoyopod.ui.lvgl_binding"
```

Expected: matches only in docs/ (spec/plan history).

- [ ] **Step 7.2: CI gate**

```bash
uv run python scripts/quality.py ci
```

- [ ] **Step 7.3: Commit any final fixup**

```bash
git add -A
git commit -m "refactor(display): final sweep for yoyopod.ui.display stragglers" 2>/dev/null || true
```

---

## Definition of Done

- `src/yoyopod/backends/display/` and `src/yoyopod/backends/display/lvgl/` populated.
- `src/yoyopod/integrations/display/` registered in `app.py`.
- `src/yoyopod/ui/display/` and `src/yoyopod/ui/lvgl_binding/` deleted.
- All three adapters (Pimoroni, Whisplay, Simulation) pass their tests + Pi validate.
- No raw LVGL imports outside `backends/display/lvgl/` and `backends/display/whisplay.py`.

---

## What's next (Plan B2)

Input HAL consolidation — same pattern for buttons and PTT.

---

*End of implementation plan.*
