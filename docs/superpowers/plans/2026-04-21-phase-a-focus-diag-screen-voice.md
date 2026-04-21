# Phase A — Plan 4: Focus + Diagnostics + Screen + Voice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the four cross-cutting integrations that subsequent plans depend on. `focus` is the AudioFocus arbiter (replaces `CallInterruptionPolicy`) — required before music/call migrations in Plans 5 and 6. `diagnostics` owns the structured event log, snapshot command, and responsiveness watchdog. `screen` folds `ScreenPowerService` into integration form. `voice` rehomes `VoiceRuntimeCoordinator` as an integration.

**Architecture:** Same per-integration shape as prior plans. Focus is stateless beyond its arbiter field; diagnostics subscribes to the bus wildly and writes JSONL; screen owns a single `ui.tick()` callback registered into `YoyoPodApp`; voice orchestrates STT engine + TTS engine + command matcher.

**Tech Stack:** Python 3.12+, pytest, uv, existing STT (`vosk`) and TTS backends. No new runtime dependencies.

**Spec reference:** `docs/superpowers/specs/2026-04-21-phase-a-spine-rewrite-design.md` §5, §6, §7, §8, §9.1, §9.2, §11.2 (steps 5-7).

**Prerequisite:** Plans 1-3 executed; `integrations/power/`, `network/`, `location/`, `contacts/`, `cloud/` working.

---

## File Structure

**Focus:**
- `src/yoyopod/integrations/focus/__init__.py`
- `src/yoyopod/integrations/focus/commands.py`
- `src/yoyopod/integrations/focus/events.py`
- `src/yoyopod/integrations/focus/arbiter.py`
- `tests/integrations/test_focus.py`

**Diagnostics:**
- `src/yoyopod/integrations/diagnostics/__init__.py`
- `src/yoyopod/integrations/diagnostics/commands.py`
- `src/yoyopod/integrations/diagnostics/events.py`
- `src/yoyopod/integrations/diagnostics/log_writer.py`
- `src/yoyopod/integrations/diagnostics/snapshot.py`
- `src/yoyopod/integrations/diagnostics/watchdog.py`
- `tests/integrations/test_diagnostics.py`

**Screen:**
- `src/yoyopod/integrations/screen/__init__.py`
- `src/yoyopod/integrations/screen/commands.py`
- `src/yoyopod/integrations/screen/handlers.py`
- `tests/integrations/test_screen.py`

**Voice:**
- `src/yoyopod/integrations/voice/__init__.py`
- `src/yoyopod/integrations/voice/commands.py`
- `src/yoyopod/integrations/voice/events.py`
- `src/yoyopod/integrations/voice/pipeline.py`
- `src/yoyopod/backends/voice/__init__.py` (moved)
- `src/yoyopod/backends/voice/stt.py` (moved from `src/yoyopod/voice/stt*.py`)
- `src/yoyopod/backends/voice/tts.py` (moved from `src/yoyopod/voice/tts*.py`)
- `tests/integrations/test_voice.py`

---

## Task 1: Branch state verification

- [ ] **Step 1.1: Confirm Plan 3 is landed**

Run:
```bash
git log --oneline -15
ls src/yoyopod/integrations/
uv run pytest tests/core/ tests/integrations/ -q
```

Expected: Plan 3 commits visible; 5 integrations exist (`power`, `network`, `location`, `contacts`, `cloud`); all tests green.

---

## Task 2: Focus integration (AudioFocus arbiter)

The focus integration is the single source of truth for "who owns audio right now." Replaces `CallInterruptionPolicy` and its cross-domain call-interrupts-music logic.

**Entities:** `focus.owner` (`"call" | "music" | "voice" | None`; attrs: `preempted`, `requested_at`).

**Events:** `AudioFocusGrantedEvent(owner, preempted)`, `AudioFocusLostEvent(owner, preempted_by)`.

**Commands:** `request_focus`, `release_focus`.

### Subtask 2.1: Focus events

- [ ] **Step 2.1.1: Create `src/yoyopod/integrations/focus/events.py`**

```python
"""Audio focus events."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class AudioFocusGrantedEvent:
    """Focus was granted to `owner`. If `preempted` is non-empty, they were kicked."""

    owner: str
    preempted: str | None


@dataclass(frozen=True, slots=True)
class AudioFocusLostEvent:
    """`owner` lost focus because `preempted_by` took it."""

    owner: str
    preempted_by: str
```

### Subtask 2.2: Focus commands

- [ ] **Step 2.2.1: Create `src/yoyopod/integrations/focus/commands.py`**

```python
"""Typed commands for the focus integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class RequestFocusCommand:
    """Request audio focus on behalf of `owner`. Grants immediately; pre-empts existing owner."""

    owner: str


@dataclass(frozen=True, slots=True)
class ReleaseFocusCommand:
    """Release audio focus if `owner` currently holds it. No-op otherwise."""

    owner: str
```

### Subtask 2.3: Arbiter implementation + test

- [ ] **Step 2.3.1: Create `tests/integrations/test_focus.py`**

```python
import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.focus import setup as setup_focus, teardown as teardown_focus
from yoyopod.integrations.focus.commands import (
    ReleaseFocusCommand,
    RequestFocusCommand,
)
from yoyopod.integrations.focus.events import (
    AudioFocusGrantedEvent,
    AudioFocusLostEvent,
)


@pytest.fixture
def app_with_focus():
    app = build_test_app()
    app.register_integration(
        "focus",
        setup=lambda a: setup_focus(a),
        teardown=lambda a: teardown_focus(a),
    )
    app.setup()
    yield app
    app.stop()


def test_initial_focus_is_none(app_with_focus):
    assert app_with_focus.states.get_value("focus.owner") is None


def test_request_grants_focus_when_none_owned(app_with_focus):
    app = app_with_focus
    granted: list[AudioFocusGrantedEvent] = []
    app.bus.subscribe(AudioFocusGrantedEvent, lambda ev: granted.append(ev))

    app.services.call("focus", "request_focus", RequestFocusCommand(owner="music"))
    app.drain()

    assert app.states.get_value("focus.owner") == "music"
    assert len(granted) == 1
    assert granted[0].owner == "music"
    assert granted[0].preempted is None


def test_request_preempts_current_owner(app_with_focus):
    app = app_with_focus
    lost: list[AudioFocusLostEvent] = []
    granted: list[AudioFocusGrantedEvent] = []
    app.bus.subscribe(AudioFocusLostEvent, lambda ev: lost.append(ev))
    app.bus.subscribe(AudioFocusGrantedEvent, lambda ev: granted.append(ev))

    app.services.call("focus", "request_focus", RequestFocusCommand(owner="music"))
    app.drain()
    app.services.call("focus", "request_focus", RequestFocusCommand(owner="call"))
    app.drain()

    assert app.states.get_value("focus.owner") == "call"
    assert len(lost) == 1
    assert lost[0].owner == "music"
    assert lost[0].preempted_by == "call"
    assert granted[-1].preempted == "music"


def test_request_same_owner_is_idempotent(app_with_focus):
    app = app_with_focus
    granted: list[AudioFocusGrantedEvent] = []
    app.bus.subscribe(AudioFocusGrantedEvent, lambda ev: granted.append(ev))

    app.services.call("focus", "request_focus", RequestFocusCommand(owner="call"))
    app.drain()
    app.services.call("focus", "request_focus", RequestFocusCommand(owner="call"))
    app.drain()

    assert len(granted) == 1  # no duplicate grant
    assert app.states.get_value("focus.owner") == "call"


def test_release_clears_focus_for_correct_owner(app_with_focus):
    app = app_with_focus
    app.services.call("focus", "request_focus", RequestFocusCommand(owner="music"))
    app.drain()
    app.services.call("focus", "release_focus", ReleaseFocusCommand(owner="music"))
    app.drain()

    assert app.states.get_value("focus.owner") is None


def test_release_noop_when_different_owner(app_with_focus):
    app = app_with_focus
    app.services.call("focus", "request_focus", RequestFocusCommand(owner="music"))
    app.drain()
    app.services.call("focus", "release_focus", ReleaseFocusCommand(owner="call"))
    app.drain()

    # music still owns; release by call is a no-op
    assert app.states.get_value("focus.owner") == "music"


def test_release_without_owner_is_noop(app_with_focus):
    app = app_with_focus
    app.services.call("focus", "release_focus", ReleaseFocusCommand(owner="music"))
    app.drain()

    assert app.states.get_value("focus.owner") is None
```

- [ ] **Step 2.3.2: Create `src/yoyopod/integrations/focus/arbiter.py` and `__init__.py`**

`src/yoyopod/integrations/focus/arbiter.py`:

```python
"""Stateless audio focus arbiter helper."""

from __future__ import annotations

import time
from typing import Any

from yoyopod.integrations.focus.events import (
    AudioFocusGrantedEvent,
    AudioFocusLostEvent,
)


def request_focus(app: Any, new_owner: str) -> None:
    """Grant focus to new_owner. Pre-empts any current owner."""
    current = app.states.get_value("focus.owner")
    if current == new_owner:
        return  # idempotent

    if current is not None and current != new_owner:
        app.bus.publish(AudioFocusLostEvent(owner=current, preempted_by=new_owner))

    app.states.set(
        "focus.owner",
        new_owner,
        attrs={"preempted": current, "requested_at": time.time()},
    )
    app.bus.publish(AudioFocusGrantedEvent(owner=new_owner, preempted=current))


def release_focus(app: Any, owner: str) -> None:
    """Release focus iff owner holds it."""
    current = app.states.get_value("focus.owner")
    if current != owner:
        return
    app.states.set("focus.owner", None, attrs={"released_by": owner})
```

`src/yoyopod/integrations/focus/__init__.py`:

```python
"""Focus integration: audio focus arbiter (replaces CallInterruptionPolicy)."""

from __future__ import annotations

from typing import Any

from yoyopod.integrations.focus.arbiter import release_focus, request_focus
from yoyopod.integrations.focus.commands import (
    ReleaseFocusCommand,
    RequestFocusCommand,
)


def setup(app: Any) -> None:
    def handle_request(cmd: RequestFocusCommand) -> None:
        request_focus(app, cmd.owner)

    def handle_release(cmd: ReleaseFocusCommand) -> None:
        release_focus(app, cmd.owner)

    app.services.register("focus", "request_focus", handle_request)
    app.services.register("focus", "release_focus", handle_release)


def teardown(app: Any) -> None:
    # Focus holds no background resources; nothing to tear down.
    pass
```

- [ ] **Step 2.3.3: Run tests, format/lint/type, commit**

```bash
uv run pytest tests/integrations/test_focus.py -v
uv run black src/yoyopod/integrations/focus/ tests/integrations/test_focus.py
uv run ruff check src/yoyopod/integrations/focus/ tests/integrations/test_focus.py
uv run mypy src/yoyopod/integrations/focus/
git add -A
git commit -m "feat(integrations/focus): AudioFocus arbiter (replaces CallInterruptionPolicy)

Stateless arbiter: request_focus pre-empts current owner, emits
AudioFocusGranted + AudioFocusLost events. release_focus is idempotent.
focus.owner state entity is the single source of truth.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Diagnostics integration (event log + snapshot + responsiveness watchdog)

The diagnostics integration is the engine for LLM-assisted debugging:
1. Subscribes to every event on the bus + every state change + every command invocation, and writes structured JSONL.
2. Exposes `diagnostics.snapshot` command that dumps state + subscriptions + tick stats.
3. Measures `scheduler.drain() + bus.drain()` duration per tick and emits `ResponsivenessLagEvent` when over threshold.

**Entities:** none directly; diagnostics consumes the bus.

**Events:** `ResponsivenessLagEvent` already defined in `core/events.py`.

**Commands:** `snapshot(SnapshotCommand)`, `mark_user_activity(MarkUserActivityCommand)`.

### Subtask 3.1: Diagnostics commands + events

- [ ] **Step 3.1.1: Create `src/yoyopod/integrations/diagnostics/commands.py`**

```python
"""Typed commands for the diagnostics integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class SnapshotCommand:
    """Write a diagnostic snapshot JSON file to the log directory."""

    reason: str


@dataclass(frozen=True, slots=True)
class MarkUserActivityCommand:
    """Publish a UserActivityEvent indicating user input (keeps screen awake)."""

    action_name: str | None = None
```

### Subtask 3.2: Log writer with TDD

The log writer subscribes to the bus and appends each event to a rolling JSONL file.

- [ ] **Step 3.2.1: Create `tests/integrations/test_diagnostics_log_writer.py`**

```python
import json
from pathlib import Path

import pytest

from yoyopod.core.events import StateChangedEvent, UserActivityEvent
from yoyopod.core.testing import build_test_app
from yoyopod.integrations.diagnostics.log_writer import LogWriter


@pytest.fixture
def log_path(tmp_path) -> Path:
    return tmp_path / "events.jsonl"


def test_log_writer_appends_state_changed(app_with_log_writer):
    app, path = app_with_log_writer
    app.states.set("power.battery_percent", 80)
    app.drain()

    lines = path.read_text(encoding="utf-8").strip().splitlines()
    assert len(lines) >= 1
    last = json.loads(lines[-1])
    assert last["kind"] == "event"
    assert last["type"] == "StateChangedEvent"
    assert last["payload"]["entity"] == "power.battery_percent"


def test_log_writer_appends_user_activity(app_with_log_writer):
    app, path = app_with_log_writer
    app.bus.publish(UserActivityEvent(action_name="button_select"))
    app.drain()

    lines = path.read_text(encoding="utf-8").strip().splitlines()
    assert any(json.loads(line)["type"] == "UserActivityEvent" for line in lines)


def test_log_writer_tracks_commands(app_with_log_writer):
    app, path = app_with_log_writer
    # Commands are logged when services.call is invoked through the diagnostics-wrapped
    # services proxy; for the test we emulate a command entry directly.
    app.services.register("demo", "do", lambda _d: None)
    app.services.call("demo", "do", {"arg": 1})

    lines = path.read_text(encoding="utf-8").strip().splitlines()
    assert any(
        json.loads(line)["kind"] == "command" and json.loads(line)["domain"] == "demo"
        for line in lines
    )


def test_log_writer_rotates_on_size(tmp_path, monkeypatch):
    path = tmp_path / "events.jsonl"
    writer = LogWriter(path=path, max_bytes=200, max_files=3)

    from yoyopod.integrations.diagnostics.log_writer import LogEntryRecord

    for i in range(100):
        writer.append(LogEntryRecord(kind="event", type="X", payload={"i": i}))

    # Should have produced at least 1 rotated file alongside current.
    rotated = list(tmp_path.glob("events.jsonl.*"))
    assert len(rotated) >= 1


@pytest.fixture
def app_with_log_writer(tmp_path):
    from yoyopod.integrations.diagnostics import setup as setup_diag, teardown as teardown_diag

    path = tmp_path / "events.jsonl"
    app = build_test_app()
    app.config = type("C", (), {
        "diagnostics": type("DC", (), {
            "log_path": str(path),
            "max_bytes": 1_000_000,
            "max_files": 3,
            "responsiveness_threshold_ms": 100.0,
            "responsiveness_watchdog_enabled": False,
        })(),
    })()
    app.register_integration(
        "diagnostics",
        setup=lambda a: setup_diag(a),
        teardown=lambda a: teardown_diag(a),
    )
    app.setup()
    yield app, path
    app.stop()
```

- [ ] **Step 3.2.2: Implement `src/yoyopod/integrations/diagnostics/log_writer.py`**

```python
"""JSONL event log writer.

Subscribes to every event on the bus plus every command invocation; writes
structured JSONL to a rolling file. See docs/superpowers/specs/2026-04-21-
phase-a-spine-rewrite-design.md §8.
"""

from __future__ import annotations

import json
import os
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal


LogKind = Literal["event", "command", "error", "lifecycle"]


@dataclass(frozen=True, slots=True)
class LogEntryRecord:
    """Wire format for one log line; not to be confused with core.LogEntry (in-memory buffer)."""

    kind: LogKind
    type: str = ""
    payload: dict | None = None
    domain: str = ""
    service: str = ""
    data: dict | None = None
    handler: str = ""
    exc: str = ""


class LogWriter:
    """Thread-safe append-only JSONL writer with size-based rotation."""

    def __init__(self, path: Path, max_bytes: int = 5 * 1024 * 1024, max_files: int = 5) -> None:
        self._path = Path(path)
        self._max_bytes = int(max_bytes)
        self._max_files = int(max_files)
        self._lock = threading.Lock()
        self._path.parent.mkdir(parents=True, exist_ok=True)

    def append(self, record: LogEntryRecord) -> None:
        """Append one JSONL line; rotate files when over size."""
        line = self._to_line(record)
        with self._lock:
            if self._path.exists() and self._path.stat().st_size + len(line) > self._max_bytes:
                self._rotate()
            with open(self._path, "a", encoding="utf-8") as f:
                f.write(line)

    def _rotate(self) -> None:
        """Rotate files: events.jsonl -> events.jsonl.1 -> .2 -> ..., drop beyond max_files."""
        for i in range(self._max_files - 1, 0, -1):
            src = self._path.with_suffix(self._path.suffix + f".{i}")
            dst = self._path.with_suffix(self._path.suffix + f".{i + 1}")
            if src.exists():
                if dst.exists():
                    dst.unlink()
                src.rename(dst)
        first_rotated = self._path.with_suffix(self._path.suffix + ".1")
        if first_rotated.exists():
            first_rotated.unlink()
        if self._path.exists():
            self._path.rename(first_rotated)

    def _to_line(self, record: LogEntryRecord) -> str:
        obj: dict[str, Any] = {
            "ts": _iso_now(),
            "kind": record.kind,
        }
        if record.kind == "event":
            obj["type"] = record.type
            obj["payload"] = record.payload or {}
        elif record.kind == "command":
            obj["domain"] = record.domain
            obj["service"] = record.service
            obj["data"] = record.data or {}
        elif record.kind == "error":
            obj["handler"] = record.handler
            obj["exc"] = record.exc
        elif record.kind == "lifecycle":
            obj["integration"] = record.payload.get("integration", "") if record.payload else ""
            obj["phase"] = record.payload.get("phase", "") if record.payload else ""
        return json.dumps(obj, default=str) + os.linesep


def _iso_now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%S", time.gmtime()) + f".{int(time.time() * 1000) % 1000:03d}Z"
```

### Subtask 3.3: Snapshot implementation

- [ ] **Step 3.3.1: Create `src/yoyopod/integrations/diagnostics/snapshot.py`**

```python
"""Diagnostic snapshot writer."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any


def write_snapshot(app: Any, *, directory: Path, reason: str) -> Path:
    """Write a JSON snapshot of current app state to `directory`."""
    directory.mkdir(parents=True, exist_ok=True)
    ts_compact = time.strftime("%Y%m%dT%H%M%S", time.gmtime())
    path = directory / f"snapshot-{ts_compact}-{reason}.json"

    states_snapshot = {
        entity: {
            "value": _jsonable(sv.value),
            "attrs": _jsonable_dict(sv.attrs),
            "last_changed_at": sv.last_changed_at,
        }
        for entity, sv in app.states.all().items()
    }

    snapshot = {
        "ts": time.time(),
        "reason": reason,
        "states": states_snapshot,
        "services": [f"{d}.{s}" for d, s in app.services.registered()],
        "recent_events_count": len(app.recent_events(10_000)),
    }

    path.write_text(json.dumps(snapshot, indent=2, default=str), encoding="utf-8")
    return path


def _jsonable(value: Any) -> Any:
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if hasattr(value, "__dict__"):
        return {k: _jsonable(v) for k, v in vars(value).items()}
    return str(value)


def _jsonable_dict(d: dict) -> dict:
    return {k: _jsonable(v) for k, v in d.items()}
```

### Subtask 3.4: Responsiveness watchdog

- [ ] **Step 3.4.1: Create `src/yoyopod/integrations/diagnostics/watchdog.py`**

```python
"""Responsiveness watchdog for the main loop."""

from __future__ import annotations

import time
from typing import Any

from yoyopod.core.events import ResponsivenessLagEvent


class ResponsivenessWatchdog:
    """Measures drain duration per tick; emits ResponsivenessLagEvent when slow."""

    def __init__(self, app: Any, threshold_ms: float = 100.0) -> None:
        self._app = app
        self._threshold = float(threshold_ms)

    def tick_start(self) -> float:
        return time.monotonic()

    def tick_end(self, start_mono: float, context: str = "drain") -> None:
        elapsed_ms = (time.monotonic() - start_mono) * 1000.0
        if elapsed_ms > self._threshold:
            self._app.bus.publish(
                ResponsivenessLagEvent(duration_ms=elapsed_ms, context=context)
            )
```

### Subtask 3.5: Integration setup

- [ ] **Step 3.5.1: Create `src/yoyopod/integrations/diagnostics/__init__.py`**

```python
"""Diagnostics integration: event log, snapshot, responsiveness watchdog."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from loguru import logger

from yoyopod.core.events import (
    LifecycleEvent,
    ResponsivenessLagEvent,
    StateChangedEvent,
    UserActivityEvent,
)
from yoyopod.integrations.diagnostics.commands import (
    MarkUserActivityCommand,
    SnapshotCommand,
)
from yoyopod.integrations.diagnostics.log_writer import (
    LogEntryRecord,
    LogWriter,
)
from yoyopod.integrations.diagnostics.snapshot import write_snapshot
from yoyopod.integrations.diagnostics.watchdog import ResponsivenessWatchdog

_STATE_KEY = "_diagnostics_integration"


def setup(app: Any) -> None:
    cfg = app.config.diagnostics
    log_path = Path(cfg.log_path)
    writer = LogWriter(
        path=log_path,
        max_bytes=int(cfg.max_bytes),
        max_files=int(cfg.max_files),
    )
    watchdog = ResponsivenessWatchdog(app, threshold_ms=float(cfg.responsiveness_threshold_ms))

    # Log every StateChangedEvent.
    def log_state_changed(ev: StateChangedEvent) -> None:
        writer.append(
            LogEntryRecord(
                kind="event",
                type="StateChangedEvent",
                payload={
                    "entity": ev.entity,
                    "old": _extract(ev.old) if ev.old else None,
                    "new": _extract(ev.new),
                },
            )
        )

    # Log every UserActivityEvent.
    def log_user_activity(ev: UserActivityEvent) -> None:
        writer.append(
            LogEntryRecord(
                kind="event",
                type="UserActivityEvent",
                payload={"action_name": ev.action_name},
            )
        )

    # Log lifecycle events.
    def log_lifecycle(ev: LifecycleEvent) -> None:
        writer.append(
            LogEntryRecord(
                kind="lifecycle",
                payload={"integration": ev.integration, "phase": ev.phase},
            )
        )

    # Log responsiveness lags.
    def log_lag(ev: ResponsivenessLagEvent) -> None:
        writer.append(
            LogEntryRecord(
                kind="error",
                handler="responsiveness",
                exc=f"lag {ev.duration_ms:.1f}ms context={ev.context}",
            )
        )

    app.bus.subscribe(StateChangedEvent, log_state_changed)
    app.bus.subscribe(UserActivityEvent, log_user_activity)
    app.bus.subscribe(LifecycleEvent, log_lifecycle)
    app.bus.subscribe(ResponsivenessLagEvent, log_lag)

    # Wrap services.call so commands are logged.
    original_call = app.services.call

    def logged_call(domain: str, service: str, data: Any = None) -> Any:
        writer.append(
            LogEntryRecord(
                kind="command",
                domain=domain,
                service=service,
                data=_extract_data(data),
            )
        )
        try:
            return original_call(domain, service, data)
        except Exception as exc:
            writer.append(
                LogEntryRecord(
                    kind="error",
                    handler=f"{domain}.{service}",
                    exc=f"{exc.__class__.__name__}: {exc}",
                )
            )
            raise

    app.services.call = logged_call  # type: ignore[assignment]

    # Commands.
    def handle_snapshot(cmd: SnapshotCommand) -> str:
        snapshot_path = write_snapshot(
            app,
            directory=log_path.parent,
            reason=cmd.reason,
        )
        logger.info("Diagnostics.snapshot written to {}", snapshot_path)
        return str(snapshot_path)

    def handle_mark_user_activity(cmd: MarkUserActivityCommand) -> None:
        app.bus.publish(UserActivityEvent(action_name=cmd.action_name))

    app.services.register("diagnostics", "snapshot", handle_snapshot)
    app.services.register("diagnostics", "mark_user_activity", handle_mark_user_activity)

    setattr(
        app,
        _STATE_KEY,
        {"writer": writer, "watchdog": watchdog, "original_services_call": original_call},
    )


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    # Restore services.call to avoid surprises in later tests that share a process.
    try:
        app.services.call = state["original_services_call"]
    except Exception:
        pass
    delattr(app, _STATE_KEY)


def _extract(sv: Any) -> dict:
    return {
        "value": _jsonable(sv.value),
        "attrs": {k: _jsonable(v) for k, v in (sv.attrs or {}).items()},
    }


def _extract_data(data: Any) -> dict:
    if data is None:
        return {}
    if hasattr(data, "__dict__"):
        return {k: _jsonable(v) for k, v in vars(data).items()}
    return {"value": _jsonable(data)}


def _jsonable(value: Any) -> Any:
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if hasattr(value, "__dict__"):
        return {k: _jsonable(v) for k, v in vars(value).items()}
    return str(value)
```

### Subtask 3.6: Run tests, commit

- [ ] **Step 3.6.1: Run**

```bash
uv run pytest tests/integrations/test_diagnostics_log_writer.py -v
uv run black src/yoyopod/integrations/diagnostics/ tests/integrations/test_diagnostics_log_writer.py
uv run ruff check src/yoyopod/integrations/diagnostics/ tests/integrations/test_diagnostics_log_writer.py
uv run mypy src/yoyopod/integrations/diagnostics/
git add -A
git commit -m "feat(integrations/diagnostics): event log writer, snapshot, responsiveness watchdog

Subscribes to StateChangedEvent, UserActivityEvent, LifecycleEvent,
ResponsivenessLagEvent and appends to rolling JSONL. Wraps services.call
to log every command invocation (and failure). Snapshot command writes
state + services registry to JSON. Watchdog measures main-loop drain time.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Screen integration

Folds `ScreenPowerService` into an integration. Owns screen wake/sleep state, brightness, idle timeout. Registers a UI tick callback into `YoyoPodApp`.

**Entities:** `screen.awake` (bool), `screen.brightness_percent` (int 0–100), `screen.idle_timeout_seconds` (int).

**Commands:** `wake`, `sleep`, `set_brightness(WakeCommand)`, `set_idle_timeout`.

Wakes the screen on `UserActivityEvent` reception.

### Subtask 4.1: Commands and handlers

- [ ] **Step 4.1.1: Create `src/yoyopod/integrations/screen/commands.py`**

```python
from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class WakeCommand:
    reason: str = "user_activity"


@dataclass(frozen=True, slots=True)
class SleepCommand:
    reason: str = "idle_timeout"


@dataclass(frozen=True, slots=True)
class SetBrightnessCommand:
    percent: int


@dataclass(frozen=True, slots=True)
class SetIdleTimeoutCommand:
    seconds: int
```

- [ ] **Step 4.1.2: Create `src/yoyopod/integrations/screen/handlers.py`**

```python
"""Screen integration state handlers."""

from __future__ import annotations

import time
from typing import Any


def wake(app: Any, reason: str = "user_activity") -> None:
    app.states.set("screen.awake", True, attrs={"reason": reason, "woke_at": time.time()})


def sleep(app: Any, reason: str = "idle_timeout") -> None:
    app.states.set("screen.awake", False, attrs={"reason": reason, "slept_at": time.time()})


def set_brightness(app: Any, percent: int) -> None:
    clamped = max(0, min(100, int(percent)))
    app.states.set("screen.brightness_percent", clamped)


def set_idle_timeout(app: Any, seconds: int) -> None:
    app.states.set("screen.idle_timeout_seconds", max(0, int(seconds)))


def check_idle_timeout(app: Any, now: float | None = None) -> None:
    if not app.states.get_value("screen.awake"):
        return
    timeout = app.states.get_value("screen.idle_timeout_seconds", 0)
    if timeout <= 0:
        return
    sv = app.states.get("screen.last_activity_at")
    if sv is None:
        return
    last = float(sv.value or 0.0)
    current = now if now is not None else time.time()
    if current - last >= timeout:
        sleep(app, reason="idle_timeout")
```

- [ ] **Step 4.1.3: Create `src/yoyopod/integrations/screen/__init__.py`**

```python
"""Screen integration: wake/sleep, brightness, idle timeout."""

from __future__ import annotations

import time
from typing import Any

from yoyopod.core.events import UserActivityEvent
from yoyopod.integrations.screen.commands import (
    SetBrightnessCommand,
    SetIdleTimeoutCommand,
    SleepCommand,
    WakeCommand,
)
from yoyopod.integrations.screen.handlers import (
    check_idle_timeout,
    set_brightness,
    set_idle_timeout,
    sleep,
    wake,
)


_STATE_KEY = "_screen_integration"


def setup(app: Any) -> None:
    cfg = app.config.screen

    app.states.set("screen.brightness_percent", int(cfg.default_brightness_percent))
    app.states.set("screen.idle_timeout_seconds", int(cfg.idle_timeout_seconds))
    app.states.set("screen.awake", True, attrs={"reason": "boot"})
    app.states.set("screen.last_activity_at", time.time())

    def on_user_activity(_ev: UserActivityEvent) -> None:
        app.states.set("screen.last_activity_at", time.time())
        if not app.states.get_value("screen.awake"):
            wake(app, reason="user_activity")

    app.bus.subscribe(UserActivityEvent, on_user_activity)

    def handle_wake(cmd: WakeCommand) -> None:
        wake(app, reason=cmd.reason)

    def handle_sleep(cmd: SleepCommand) -> None:
        sleep(app, reason=cmd.reason)

    def handle_set_brightness(cmd: SetBrightnessCommand) -> None:
        set_brightness(app, cmd.percent)

    def handle_set_idle_timeout(cmd: SetIdleTimeoutCommand) -> None:
        set_idle_timeout(app, cmd.seconds)

    app.services.register("screen", "wake", handle_wake)
    app.services.register("screen", "sleep", handle_sleep)
    app.services.register("screen", "set_brightness", handle_set_brightness)
    app.services.register("screen", "set_idle_timeout", handle_set_idle_timeout)

    # UI tick: idle-timeout check + optional display driver pump.
    def ui_tick() -> None:
        check_idle_timeout(app)

    app._ui_tick_callback = ui_tick

    setattr(app, _STATE_KEY, {})


def teardown(app: Any) -> None:
    if hasattr(app, "_ui_tick_callback"):
        delattr(app, "_ui_tick_callback")
    if hasattr(app, _STATE_KEY):
        delattr(app, _STATE_KEY)
```

- [ ] **Step 4.1.4: Create `tests/integrations/test_screen.py`**

```python
import time

import pytest

from yoyopod.core.events import UserActivityEvent
from yoyopod.core.testing import build_test_app
from yoyopod.integrations.screen import setup as setup_screen, teardown as teardown_screen
from yoyopod.integrations.screen.commands import (
    SetBrightnessCommand,
    SetIdleTimeoutCommand,
    SleepCommand,
    WakeCommand,
)


@pytest.fixture
def app_with_screen():
    app = build_test_app()
    app.config = type("C", (), {
        "screen": type("SC", (), {
            "default_brightness_percent": 75,
            "idle_timeout_seconds": 30,
        })(),
    })()
    app.register_integration(
        "screen",
        setup=lambda a: setup_screen(a),
        teardown=lambda a: teardown_screen(a),
    )
    app.setup()
    yield app
    app.stop()


def test_initial_state(app_with_screen):
    app = app_with_screen
    assert app.states.get_value("screen.awake") is True
    assert app.states.get_value("screen.brightness_percent") == 75
    assert app.states.get_value("screen.idle_timeout_seconds") == 30


def test_sleep_and_wake_via_commands(app_with_screen):
    app = app_with_screen
    app.services.call("screen", "sleep", SleepCommand(reason="test"))
    assert app.states.get_value("screen.awake") is False

    app.services.call("screen", "wake", WakeCommand(reason="test"))
    assert app.states.get_value("screen.awake") is True


def test_user_activity_wakes_screen(app_with_screen):
    app = app_with_screen
    app.services.call("screen", "sleep", SleepCommand())
    assert app.states.get_value("screen.awake") is False

    app.bus.publish(UserActivityEvent(action_name="button"))
    app.drain()

    assert app.states.get_value("screen.awake") is True


def test_set_brightness_clamps_to_0_100(app_with_screen):
    app = app_with_screen
    app.services.call("screen", "set_brightness", SetBrightnessCommand(percent=150))
    assert app.states.get_value("screen.brightness_percent") == 100
    app.services.call("screen", "set_brightness", SetBrightnessCommand(percent=-10))
    assert app.states.get_value("screen.brightness_percent") == 0


def test_set_idle_timeout(app_with_screen):
    app = app_with_screen
    app.services.call("screen", "set_idle_timeout", SetIdleTimeoutCommand(seconds=60))
    assert app.states.get_value("screen.idle_timeout_seconds") == 60


def test_idle_timeout_sleeps_screen_via_ui_tick(app_with_screen, monkeypatch):
    app = app_with_screen
    # simulate that no activity for a long time
    app.states.set("screen.last_activity_at", time.time() - 1000.0)
    app._ui_tick_callback()
    assert app.states.get_value("screen.awake") is False
```

- [ ] **Step 4.1.5: Run, commit**

```bash
uv run pytest tests/integrations/test_screen.py -v
uv run black src/yoyopod/integrations/screen/ tests/integrations/test_screen.py
uv run ruff check src/yoyopod/integrations/screen/ tests/integrations/test_screen.py
uv run mypy src/yoyopod/integrations/screen/
git add -A
git commit -m "feat(integrations/screen): wake/sleep/brightness/idle-timeout + UI tick callback

Subscribes to UserActivityEvent to wake; registers a ui_tick callback
that the main loop invokes for idle-timeout checks. Folds the old
ScreenPowerService responsibilities into the new integration shape.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Voice integration

Rehomes `VoiceRuntimeCoordinator` + STT/TTS backends. Models the voice pipeline as idle → listening → thinking → responding state machine expressed as `voice.state`.

**Entities:** `voice.state` (`"idle" | "listening" | "thinking" | "responding"`; attrs: `transcript`, `response`).

**Events:** `VoiceTranscriptReadyEvent(transcript)`, `VoiceResponseCompletedEvent(text)`.

**Commands:** `start_listening`, `stop_listening`, `say`.

### Subtask 5.1: Relocate voice backends

- [ ] **Step 5.1.1: Move STT/TTS engines under `backends/voice/`**

```bash
mkdir -p src/yoyopod/backends/voice
```

Examine `src/yoyopod/voice/` and relocate STT/TTS engine files (the names depend on the current structure; examples: `stt_vosk.py`, `tts_*.py`):

```bash
git mv src/yoyopod/voice/<stt_file>.py src/yoyopod/backends/voice/stt.py
git mv src/yoyopod/voice/<tts_file>.py src/yoyopod/backends/voice/tts.py
```

Update imports inside moved files. Create `src/yoyopod/backends/voice/__init__.py`:

```python
"""STT and TTS backend adapters."""

from __future__ import annotations

from yoyopod.backends.voice.stt import SttBackend
from yoyopod.backends.voice.tts import TtsBackend

__all__ = ["SttBackend", "TtsBackend"]
```

### Subtask 5.2: Voice events + commands

- [ ] **Step 5.2.1: Create `src/yoyopod/integrations/voice/events.py`**

```python
from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class VoiceTranscriptReadyEvent:
    transcript: str


@dataclass(frozen=True, slots=True)
class VoiceResponseCompletedEvent:
    text: str
```

- [ ] **Step 5.2.2: Create `src/yoyopod/integrations/voice/commands.py`**

```python
from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class StartListeningCommand:
    """Begin STT capture; transitions voice.state to 'listening'."""


@dataclass(frozen=True, slots=True)
class StopListeningCommand:
    """End STT capture; transitions voice.state to 'thinking'."""


@dataclass(frozen=True, slots=True)
class SayCommand:
    """Speak the given text via TTS; transitions voice.state to 'responding' then 'idle'."""

    text: str
```

### Subtask 5.3: Voice pipeline

- [ ] **Step 5.3.1: Create `src/yoyopod/integrations/voice/pipeline.py`**

```python
"""Voice pipeline state transitions driven by STT/TTS callbacks."""

from __future__ import annotations

from typing import Any

from yoyopod.integrations.focus.arbiter import release_focus, request_focus
from yoyopod.integrations.voice.events import (
    VoiceResponseCompletedEvent,
    VoiceTranscriptReadyEvent,
)


def begin_listening(app: Any, stt: Any) -> None:
    request_focus(app, "voice")
    app.states.set("voice.state", "listening")
    stt.start()


def end_listening(app: Any, stt: Any) -> None:
    app.states.set("voice.state", "thinking")

    def on_transcript(text: str) -> None:
        app.scheduler.run_on_main(
            lambda t=text: _on_transcript_ready(app, t)
        )

    stt.stop(callback=on_transcript)


def _on_transcript_ready(app: Any, transcript: str) -> None:
    app.states.set("voice.state", "thinking", attrs={"transcript": transcript})
    app.bus.publish(VoiceTranscriptReadyEvent(transcript=transcript))


def say(app: Any, tts: Any, text: str) -> None:
    app.states.set("voice.state", "responding", attrs={"response": text})

    def on_finished() -> None:
        app.scheduler.run_on_main(lambda: _on_response_finished(app, text))

    tts.speak(text, on_done=on_finished)


def _on_response_finished(app: Any, text: str) -> None:
    app.bus.publish(VoiceResponseCompletedEvent(text=text))
    app.states.set("voice.state", "idle")
    release_focus(app, "voice")
```

### Subtask 5.4: Voice integration setup

- [ ] **Step 5.4.1: Create `src/yoyopod/integrations/voice/__init__.py`**

```python
"""Voice integration: STT, TTS, pipeline state."""

from __future__ import annotations

from typing import Any

from yoyopod.integrations.voice.commands import (
    SayCommand,
    StartListeningCommand,
    StopListeningCommand,
)
from yoyopod.integrations.voice.pipeline import (
    begin_listening,
    end_listening,
    say,
)

_STATE_KEY = "_voice_integration"


def setup(app: Any, stt: Any | None = None, tts: Any | None = None) -> None:
    if stt is None or tts is None:
        from yoyopod.backends.voice import SttBackend, TtsBackend
        if stt is None:
            stt = SttBackend(app.config.voice)
        if tts is None:
            tts = TtsBackend(app.config.voice)

    app.states.set("voice.state", "idle")

    def handle_start(_cmd: StartListeningCommand) -> None:
        begin_listening(app, stt)

    def handle_stop(_cmd: StopListeningCommand) -> None:
        end_listening(app, stt)

    def handle_say(cmd: SayCommand) -> None:
        say(app, tts, cmd.text)

    app.services.register("voice", "start_listening", handle_start)
    app.services.register("voice", "stop_listening", handle_stop)
    app.services.register("voice", "say", handle_say)

    setattr(app, _STATE_KEY, {"stt": stt, "tts": tts})


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    try:
        close = getattr(state["stt"], "close", None)
        if callable(close):
            close()
    except Exception:
        pass
    try:
        close = getattr(state["tts"], "close", None)
        if callable(close):
            close()
    except Exception:
        pass
    delattr(app, _STATE_KEY)
```

- [ ] **Step 5.4.2: Create `tests/integrations/test_voice.py`**

```python
from dataclasses import dataclass, field
from typing import Callable

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.focus import setup as setup_focus, teardown as teardown_focus
from yoyopod.integrations.voice import setup as setup_voice, teardown as teardown_voice
from yoyopod.integrations.voice.commands import (
    SayCommand,
    StartListeningCommand,
    StopListeningCommand,
)
from yoyopod.integrations.voice.events import (
    VoiceResponseCompletedEvent,
    VoiceTranscriptReadyEvent,
)


@dataclass
class _FakeStt:
    started: bool = False
    stopped_with_callback: Callable[[str], None] | None = None

    def start(self):
        self.started = True

    def stop(self, callback):
        self.started = False
        self.stopped_with_callback = callback


@dataclass
class _FakeTts:
    spoken: list[str] = field(default_factory=list)
    on_done_cb: Callable[[], None] | None = None

    def speak(self, text, on_done):
        self.spoken.append(text)
        self.on_done_cb = on_done


@pytest.fixture
def app_with_voice():
    app = build_test_app()
    stt = _FakeStt()
    tts = _FakeTts()
    app.register_integration("focus", setup=lambda a: setup_focus(a), teardown=lambda a: teardown_focus(a))
    app.register_integration(
        "voice",
        setup=lambda a: setup_voice(a, stt=stt, tts=tts),
        teardown=lambda a: teardown_voice(a),
    )
    app.setup()
    yield app, stt, tts
    app.stop()


def test_start_listening_requests_focus(app_with_voice):
    app, stt, _ = app_with_voice
    app.services.call("voice", "start_listening", StartListeningCommand())
    app.drain()

    assert stt.started is True
    assert app.states.get_value("voice.state") == "listening"
    assert app.states.get_value("focus.owner") == "voice"


def test_stop_listening_transitions_to_thinking_and_publishes_transcript(app_with_voice):
    app, stt, _ = app_with_voice
    captured: list[VoiceTranscriptReadyEvent] = []
    app.bus.subscribe(VoiceTranscriptReadyEvent, lambda ev: captured.append(ev))

    app.services.call("voice", "start_listening", StartListeningCommand())
    app.drain()

    app.services.call("voice", "stop_listening", StopListeningCommand())
    app.drain()

    assert app.states.get_value("voice.state") == "thinking"
    assert stt.stopped_with_callback is not None

    # Simulate STT completing with a transcript.
    stt.stopped_with_callback("hello world")
    app.drain()

    assert any(ev.transcript == "hello world" for ev in captured)


def test_say_plays_tts_and_releases_focus(app_with_voice):
    app, _, tts = app_with_voice
    completed: list[VoiceResponseCompletedEvent] = []
    app.bus.subscribe(VoiceResponseCompletedEvent, lambda ev: completed.append(ev))

    # Acquire voice focus first.
    from yoyopod.integrations.focus.arbiter import request_focus
    request_focus(app, "voice")

    app.services.call("voice", "say", SayCommand(text="Hi there"))
    app.drain()

    assert tts.spoken == ["Hi there"]
    assert app.states.get_value("voice.state") == "responding"

    # Simulate TTS completion.
    tts.on_done_cb()
    app.drain()

    assert app.states.get_value("voice.state") == "idle"
    assert app.states.get_value("focus.owner") is None
    assert any(ev.text == "Hi there" for ev in completed)
```

- [ ] **Step 5.4.3: Run, commit**

```bash
uv run pytest tests/integrations/test_voice.py -v
uv run black src/yoyopod/integrations/voice/ src/yoyopod/backends/voice/ tests/integrations/test_voice.py
uv run ruff check src/yoyopod/integrations/voice/ src/yoyopod/backends/voice/ tests/integrations/test_voice.py
uv run mypy src/yoyopod/integrations/voice/ src/yoyopod/backends/voice/
git add -A
git commit -m "feat(integrations/voice): STT/TTS pipeline with focus integration

Voice integration drives STT begin/end + TTS say via a small pipeline that
transitions voice.state (idle -> listening -> thinking -> responding).
Acquires focus.owner='voice' during listening; releases on response done.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Final verification

- [ ] **Step 6.1: Confirm structure**

```bash
ls src/yoyopod/integrations/
# expected: __init__.py, power/, network/, location/, contacts/, cloud/, focus/, diagnostics/, screen/, voice/
```

- [ ] **Step 6.2: CI gate**

```bash
uv run python scripts/quality.py ci
```

Expected: all green.

- [ ] **Step 6.3: Commit count**

Expect ~10 new commits on top of Plan 3 (one per subtask).

---

## Definition of Done

- `focus`, `diagnostics`, `screen`, `voice` integrations populated.
- Event log writes JSONL to configured path.
- Snapshot command writes state + services dump.
- Screen idle-timeout works via ui_tick callback.
- Voice pipeline transitions state through listening → thinking → responding → idle, coordinating with focus integration.
- Full CI gate green.

---

## What's next (Plan 5)

`music` — local mpv playback, library, playlists, recent tracks. First integration to actually use `focus.request_focus("music")`.

---

*End of implementation plan.*
