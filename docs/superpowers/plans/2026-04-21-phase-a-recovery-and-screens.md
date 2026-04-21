# Phase A — Plan 7: Recovery Integration + 17-Screen Touch-Up Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-home `RecoverySupervisor` as an integration and migrate all 17 screens to consume `app.states` and `app.services` instead of direct manager references. This is the UI-side work that makes Phase A feel coherent end-to-end.

**Architecture:** Recovery subscribes to `BackendStoppedEvent`, retries the owning integration's backend on a backoff, publishes `RecoveryAttemptedEvent`. Screens change constructor signature from `(manager1, manager2, ...)` to `(app: YoyoPodApp)`. Read paths become `app.states.get_value(...)`; action paths become `app.services.call(...)`. Screens that need periodic refresh subscribe to `StateChangedEvent` with an entity prefix filter.

**Tech Stack:** Python 3.12+, pytest, uv, existing PIL/LVGL rendering (unchanged — only the data-access seam changes).

**Spec reference:** spec §10 (screen touch-up), §11.2 step 10-11.

**Prerequisite:** Plans 1-6 executed. All 10 integrations working; `VoIPManager`/`CallCoordinator` deleted.

---

## File Structure

### Files to create

- `src/yoyopod/integrations/recovery/__init__.py`
- `src/yoyopod/integrations/recovery/commands.py`
- `src/yoyopod/integrations/recovery/events.py` (optional — `RecoveryAttemptedEvent`)
- `src/yoyopod/integrations/recovery/supervisor.py` (logic relocated from `src/yoyopod/runtime/recovery.py`)
- `tests/integrations/test_recovery.py`

### Files to modify (17 screen files + ScreenManager)

Under `src/yoyopod/ui/screens/`:
- `home.py`
- `hub.py`
- `menu.py`
- `navigation/listen.py`
- `navigation/ask.py`
- `system/power.py`
- `music/now_playing.py`
- `music/playlist.py`
- `music/recent.py`
- `voip/call.py`
- `voip/talk_contact.py`
- `voip/call_history.py`
- `voip/contact_list.py`
- `voip/voice_note.py`
- `voip/quick_call.py`
- `voip/incoming_call.py` (or wherever the incoming-call screen lives)
- `voip/outgoing_call.py`
- `voip/in_call.py`
- `manager.py` (ScreenManager — takes `app` in constructor, uses it for screen-instance resolution)
- `router.py` (if it needs `app`)

### Files to delete

- `src/yoyopod/runtime/recovery.py` (relocated to `integrations/recovery/supervisor.py`)
- `src/yoyopod/runtime/event_wiring.py` (replaced by integrations' own setup())
- `src/yoyopod/runtime/loop.py` / similar (run() now on YoyoPodApp — per Plan 2)
- `src/yoyopod/runtime/screen_power.py` (folded into integrations/screen/ in Plan 4)
- `src/yoyopod/runtime/shutdown.py` (teardown() handled by app_shell)
- `src/yoyopod/runtime/voice.py` (folded into integrations/voice/ in Plan 4)

---

## Task 1: Branch state verification

- [ ] **Step 1.1**

```bash
git log --oneline -30
uv run pytest tests/ -q
```

Expected: Plans 1-6 all landed; tests green.

---

## Task 2: Recovery integration

**Events:** `RecoveryAttemptedEvent(domain, success, reason)`.

**Commands:** `request_recovery(RequestRecoveryCommand(domain))` — manual trigger.

Recovery subscribes to `BackendStoppedEvent` and retries the named integration's backend on an exponential backoff.

### Subtask 2.1: Events + commands

- [ ] **Step 2.1.1: Create `src/yoyopod/integrations/recovery/events.py`**

```python
"""Recovery integration events."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class RecoveryAttemptedEvent:
    """Published after a backend-recovery attempt completes."""

    domain: str
    success: bool
    reason: str = ""
```

- [ ] **Step 2.1.2: Create `src/yoyopod/integrations/recovery/commands.py`**

```python
"""Recovery integration commands."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class RequestRecoveryCommand:
    """Manually trigger a recovery attempt for the given domain."""

    domain: str
```

### Subtask 2.2: Supervisor implementation

- [ ] **Step 2.2.1: Create `src/yoyopod/integrations/recovery/supervisor.py`**

```python
"""Recovery supervisor.

Subscribes to BackendStoppedEvent and schedules retry attempts via the
main-thread scheduler (with exponential backoff per domain). Publishes
RecoveryAttemptedEvent after each attempt.
"""

from __future__ import annotations

import threading
import time
from dataclasses import dataclass, field
from typing import Any, Callable

from loguru import logger

from yoyopod.core.events import BackendStoppedEvent
from yoyopod.integrations.recovery.events import RecoveryAttemptedEvent


@dataclass
class _DomainState:
    attempt_count: int = 0
    next_delay_seconds: float = 1.0
    scheduled: bool = False


MAX_DELAY_SECONDS = 30.0


class RecoverySupervisor:
    def __init__(
        self,
        app: Any,
        retry_handlers: dict[str, Callable[[], bool]] | None = None,
    ) -> None:
        self._app = app
        self._retry_handlers: dict[str, Callable[[], bool]] = retry_handlers or {}
        self._domains: dict[str, _DomainState] = {}
        self._lock = threading.Lock()

    def register_retry_handler(self, domain: str, handler: Callable[[], bool]) -> None:
        """Integrations call this in their setup() to opt in to recovery."""
        self._retry_handlers[domain] = handler

    def on_backend_stopped(self, event: BackendStoppedEvent) -> None:
        """Schedule a retry for the domain whose backend stopped."""
        self._schedule_retry(event.domain, reason=event.reason)

    def request_recovery(self, domain: str) -> None:
        """Manual retry request (from command)."""
        self._schedule_retry(domain, reason="manual")

    def _schedule_retry(self, domain: str, reason: str) -> None:
        with self._lock:
            state = self._domains.setdefault(domain, _DomainState())
            if state.scheduled:
                return
            state.scheduled = True
            delay = state.next_delay_seconds

        def run_retry() -> None:
            time.sleep(delay)
            self._app.scheduler.run_on_main(lambda: self._attempt(domain, reason))

        threading.Thread(target=run_retry, daemon=True, name=f"recovery-{domain}").start()

    def _attempt(self, domain: str, reason: str) -> None:
        handler = self._retry_handlers.get(domain)
        with self._lock:
            state = self._domains.setdefault(domain, _DomainState())
            state.scheduled = False
            state.attempt_count += 1

        success = False
        if handler is None:
            logger.warning("No recovery handler for domain {}", domain)
        else:
            try:
                success = bool(handler())
            except Exception as exc:
                logger.error("Recovery handler {} raised: {}", domain, exc)

        self._app.bus.publish(
            RecoveryAttemptedEvent(domain=domain, success=success, reason=reason)
        )

        with self._lock:
            if success:
                state.attempt_count = 0
                state.next_delay_seconds = 1.0
            else:
                state.next_delay_seconds = min(state.next_delay_seconds * 2, MAX_DELAY_SECONDS)

        if not success:
            # Schedule another retry.
            self._schedule_retry(domain, reason=f"retry_{state.attempt_count}")
```

### Subtask 2.3: Integration setup

- [ ] **Step 2.3.1: Create `src/yoyopod/integrations/recovery/__init__.py`**

```python
"""Recovery integration."""

from __future__ import annotations

from typing import Any

from yoyopod.core.events import BackendStoppedEvent
from yoyopod.integrations.recovery.commands import RequestRecoveryCommand
from yoyopod.integrations.recovery.supervisor import RecoverySupervisor


_STATE_KEY = "_recovery_integration"


def setup(app: Any) -> None:
    supervisor = RecoverySupervisor(app=app)

    app.bus.subscribe(
        BackendStoppedEvent,
        lambda ev: supervisor.on_backend_stopped(ev),
    )

    def handle_request_recovery(cmd: RequestRecoveryCommand) -> None:
        supervisor.request_recovery(cmd.domain)

    app.services.register("recovery", "request_recovery", handle_request_recovery)

    # Expose supervisor so other integrations can register their retry handlers.
    setattr(app, "recovery_supervisor", supervisor)
    setattr(app, _STATE_KEY, {"supervisor": supervisor})


def teardown(app: Any) -> None:
    if hasattr(app, _STATE_KEY):
        delattr(app, _STATE_KEY)
    if hasattr(app, "recovery_supervisor"):
        delattr(app, "recovery_supervisor")
```

### Subtask 2.4: Tests

- [ ] **Step 2.4.1: Create `tests/integrations/test_recovery.py`**

```python
import time

import pytest

from yoyopod.core.events import BackendStoppedEvent
from yoyopod.core.testing import build_test_app
from yoyopod.integrations.recovery import setup as setup_recovery, teardown as teardown_recovery
from yoyopod.integrations.recovery.commands import RequestRecoveryCommand
from yoyopod.integrations.recovery.events import RecoveryAttemptedEvent


@pytest.fixture
def app_with_recovery():
    app = build_test_app()
    app.register_integration(
        "recovery",
        setup=lambda a: setup_recovery(a),
        teardown=lambda a: teardown_recovery(a),
    )
    app.setup()
    yield app
    app.stop()


def test_backend_stopped_schedules_retry_handler(app_with_recovery):
    app = app_with_recovery
    attempts = []
    def handler():
        attempts.append("call")
        return True

    app.recovery_supervisor.register_retry_handler("call", handler)

    captured: list[RecoveryAttemptedEvent] = []
    app.bus.subscribe(RecoveryAttemptedEvent, lambda ev: captured.append(ev))

    app.bus.publish(BackendStoppedEvent(domain="call", reason="test"))
    app.drain()

    # Backoff in tests is 1s + scheduling overhead; poll briefly.
    deadline = time.monotonic() + 3.0
    while time.monotonic() < deadline and not attempts:
        time.sleep(0.1)
        app.drain()

    assert attempts == ["call"]
    assert captured, "RecoveryAttemptedEvent not published"
    assert captured[-1].success is True


def test_request_recovery_command_triggers_handler(app_with_recovery):
    app = app_with_recovery
    attempts = []
    app.recovery_supervisor.register_retry_handler("music", lambda: (attempts.append("music"), True)[1])

    app.services.call("recovery", "request_recovery", RequestRecoveryCommand(domain="music"))

    deadline = time.monotonic() + 3.0
    while time.monotonic() < deadline and not attempts:
        time.sleep(0.1)
        app.drain()

    assert attempts == ["music"]


def test_failing_handler_schedules_retry(app_with_recovery):
    app = app_with_recovery
    count = {"n": 0}

    def fail_twice_then_succeed():
        count["n"] += 1
        return count["n"] >= 3

    app.recovery_supervisor.register_retry_handler("net", fail_twice_then_succeed)

    app.services.call("recovery", "request_recovery", RequestRecoveryCommand(domain="net"))

    deadline = time.monotonic() + 10.0  # backoff 1+2+4 = 7s max
    while time.monotonic() < deadline and count["n"] < 3:
        time.sleep(0.2)
        app.drain()

    assert count["n"] >= 3
```

- [ ] **Step 2.4.2: Run, commit**

```bash
uv run pytest tests/integrations/test_recovery.py -v
uv run black src/yoyopod/integrations/recovery/ tests/integrations/test_recovery.py
uv run ruff check src/yoyopod/integrations/recovery/ tests/integrations/test_recovery.py
uv run mypy src/yoyopod/integrations/recovery/
git add -A
git commit -m "feat(integrations/recovery): relocate RecoverySupervisor under integrations

Subscribes to BackendStoppedEvent; per-domain exponential-backoff retries;
publishes RecoveryAttemptedEvent. Integrations register retry handlers in
their own setup().

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Subtask 2.5: Delete legacy recovery module

- [ ] **Step 2.5.1: Delete + CI + commit**

```bash
git rm src/yoyopod/runtime/recovery.py
uv run python scripts/quality.py ci
git add -A
git commit -m "refactor: delete legacy src/yoyopod/runtime/recovery.py

Replaced by integrations/recovery/.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Screen touch-up — the migration pattern

Every screen currently takes multiple manager references in its constructor. After migration, every screen takes exactly one dependency: `app: YoyoPodApp`. Reads of domain data go through `app.states.get_value(...)` or `app.states.get(...)`. Actions go through `app.services.call(...)`.

**Canonical migration pattern (apply to every screen):**

1. Change the constructor: replace all manager/coordinator arguments with a single `app: YoyoPodApp`.
2. Subscribe to `StateChangedEvent` (filtered by the entity prefix the screen cares about) to set `self.dirty = True` or similar render trigger. If the screen does not re-render on state change (static screen), skip subscription.
3. Rewrite every data read:
   - `self.voip_manager.is_registered()` → `app.states.get_value("call.registration") == "ok"`
   - `self.power_manager.get_snapshot().battery.percent` → `app.states.get_value("power.battery_percent")`
   - etc.
4. Rewrite every action:
   - `self.voip_manager.make_call(addr)` → `app.services.call("call", "dial", DialCommand(address=addr))`
   - `self.music_backend.play(uri)` → `app.services.call("music", "play", PlayCommand(track_uri=uri))`
   - etc.
5. Update or add tests that construct the screen with a test app built via `build_test_app()` and exercise interaction through state + services.

The following subtasks apply this pattern to each screen.

---

## Task 4: Migrate home, hub, menu, navigation screens (simple cases)

These screens are mostly passive (show info, no commands) or make limited service calls. Handle them in one task since the pattern is uniform.

- [ ] **Step 4.1: Migrate `home.py`**

Open `src/yoyopod/ui/screens/home.py`. Change constructor to `def __init__(self, app): self.app = app`. Replace reads of manager attributes with `app.states.get_value(...)`. If the home screen shows the current track or call status, use entity reads. If the home screen triggers navigation only, no service calls are needed.

Update corresponding test file (`tests/test_home_screen.py` if it exists) to build the app via `build_test_app()` and set relevant states.

- [ ] **Step 4.2: Migrate `hub.py`, `menu.py`, `navigation/listen.py`, `navigation/ask.py`, `system/power.py`**

Same mechanical migration. For `system/power.py`, the screen reads `app.states.get_value("power.battery_percent")`, `power.charging`, `power.external_power` etc. For `navigation/ask.py`, it issues `voice.start_listening`/`voice.stop_listening` via `app.services.call`.

- [ ] **Step 4.3: Run affected screen tests**

```bash
uv run pytest tests/ -k "home_screen or hub_screen or menu_screen or listen_screen or ask_screen or power_screen" -v
```

- [ ] **Step 4.4: Commit**

```bash
git add -A
git commit -m "refactor(ui): migrate home/hub/menu/navigation/power screens to app-based constructor

Screens now take a single app argument; reads use app.states, actions use
app.services. No behaviour change.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Migrate music screens

- [ ] **Step 5.1: Migrate `music/now_playing.py`, `music/playlist.py`, `music/recent.py`**

Critical reads:
- `app.states.get_value("music.state")` — "idle" | "playing" | "paused"
- `app.states.get("music.track")` — returns `StateValue` with `.attrs["title"]`, `.attrs["artist"]`, etc.
- `app.states.get_value("music.volume_percent")`

Actions:
- `app.services.call("music", "play", PlayCommand(track_uri=uri))`
- `app.services.call("music", "pause", PauseCommand())`
- `app.services.call("music", "resume", ResumeCommand())`
- `app.services.call("music", "next", NextCommand())`
- `app.services.call("music", "prev", PrevCommand())`
- `app.services.call("music", "seek", SeekCommand(position_seconds=pos))`
- `app.services.call("music", "set_volume", SetVolumeCommand(percent=p))`

For position/progress display on now_playing — position is NOT in the state store (ephemeral, polls too fast). Keep the existing direct `backend.get_position()` via `app.states.get_value("_music_integration")["backend"]` — or expose a small getter on the app: `app.music_position_seconds()`. Simpler: add a helper to the music integration exported as `app.get_music_position()`:

In `src/yoyopod/integrations/music/__init__.py` `setup()` append:

```python
def get_music_position() -> float:
    try:
        return float(backend.get_position())
    except Exception:
        return 0.0

app.get_music_position = get_music_position
```

In `teardown()` append `if hasattr(app, "get_music_position"): delattr(app, "get_music_position")`.

Now `now_playing.py` reads the live position via `app.get_music_position()`.

Subscribe to `StateChangedEvent` filtered on `music.*` to mark dirty. On each `render()`, fetch position.

- [ ] **Step 5.2: Run music screen tests**

```bash
uv run pytest tests/ -k "music_screen or now_playing or playlist_screen or recent" -v
```

- [ ] **Step 5.3: Commit**

```bash
git add -A
git commit -m "refactor(ui): migrate music screens to app-based constructor

Now-playing uses app.get_music_position() for ephemeral position reads
that shouldn't live in the state store.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Migrate call screens

These are the most complex screens — they subscribe to multiple state changes and issue many commands.

- [ ] **Step 6.1: Migrate `voip/call.py` (status screen)**

Reads:
- `app.states.get_value("call.registration")` → show registration status
- `app.states.get_value("call.state")` → show current call state

Actions: none (display only).

- [ ] **Step 6.2: Migrate `voip/quick_call.py`, `voip/talk_contact.py`, `voip/contact_list.py`**

Actions:
- `app.services.call("call", "dial", DialCommand(address=...))`
- `app.services.call("call", "send_message", SendMessageCommand(...))` (if applicable)
- `app.services.call("call", "start_voice_note", StartVoiceNoteCommand(...))`
- `app.services.call("contacts", "lookup_by_address", LookupByAddressCommand(...))`

- [ ] **Step 6.3: Migrate `voip/call_history.py`**

Read the call history via the call integration's store. Expose a helper:

In `src/yoyopod/integrations/call/__init__.py` `setup()` append:

```python
app.get_call_history = call_history
```

In `teardown` — `delattr(app, "get_call_history")`.

Then the history screen reads entries via `app.get_call_history.recent_preview()` or similar existing API.

- [ ] **Step 6.4: Migrate `voip/voice_note.py`**

Actions:
- `app.services.call("call", "start_voice_note", StartVoiceNoteCommand(...))`
- `app.services.call("call", "stop_voice_note", StopVoiceNoteCommand())`
- `app.services.call("call", "send_voice_note", SendVoiceNoteCommand())`
- `app.services.call("call", "cancel_voice_note", CancelVoiceNoteCommand())`
- `app.services.call("call", "play_voice_note", PlayVoiceNoteCommand(file_path=...))`

Subscribes to `VoiceNoteCompletedEvent` to refresh the review screen.

- [ ] **Step 6.5: Migrate `voip/incoming_call.py`, `voip/outgoing_call.py`, `voip/in_call.py`**

Incoming:
- Read `app.states.get("call.caller")` for address + name.
- Action: `app.services.call("call", "answer", AnswerCommand())` or `"reject"`.
- Subscribe to `StateChangedEvent` filtered on `call.state` — pop screen when state leaves `"incoming"`.

Outgoing:
- Read `app.states.get("call.caller")` for callee info.
- Action: `app.services.call("call", "hangup", HangupCommand())`.

In-call:
- Reads: `app.states.get("call.caller")`, `app.states.get_value("call.muted")`.
- Actions: `mute`/`unmute`/`hangup`.
- Duration read: expose helper on app:

In `src/yoyopod/integrations/call/__init__.py` setup() append:
```python
def get_call_duration_seconds() -> int:
    return int(backend.get_call_duration())
app.get_call_duration_seconds = get_call_duration_seconds
```

- [ ] **Step 6.6: Run all voip-screen tests**

```bash
uv run pytest tests/ -k "call_screen or talk or voice_note or contact_list or incoming_call or outgoing_call or in_call" -v
```

- [ ] **Step 6.7: Commit**

```bash
git add -A
git commit -m "refactor(ui): migrate all VoIP screens to app-based constructor

Call/Quick-Call/Talk/Contact-List/Call-History/Voice-Note/Incoming/
Outgoing/In-Call all take a single app arg. Helpers get_call_history,
get_call_duration_seconds exposed on app during call setup().

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Migrate ScreenManager + router

- [ ] **Step 7.1: Update `src/yoyopod/ui/screens/manager.py`**

`ScreenManager` constructs screen instances. Change construction to pass `app` only.

Replace:
```python
screen = CallScreen(voip_manager=vm, config_manager=cm, ...)
```

with:
```python
screen = CallScreen(app=app)
```

`ScreenManager.__init__` itself takes `app` and uses it to resolve integrations' helpers (`app.get_call_history`, etc.).

- [ ] **Step 7.2: Update `src/yoyopod/ui/screens/router.py`**

If router decorates or dispatches screen creation, update its hook points to accept `app`.

- [ ] **Step 7.3: Run screen manager + routing tests**

```bash
uv run pytest tests/test_screen_routing.py tests/test_call_screen.py -v
```

- [ ] **Step 7.4: Commit**

```bash
git add -A
git commit -m "refactor(ui): ScreenManager constructs screens with app only

Router and manager take app; individual screens no longer need manager
references. AppContext dependency removed from constructor chains.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Exhaustive grep for stragglers

- [ ] **Step 8.1: Verify no screen references removed classes**

```bash
grep -rn "voip_manager\|music_backend\|power_manager\|network_manager\|people_directory\|cloud_manager\|local_music_service" src/yoyopod/ui/
```

Expected: no matches (or only matches in legacy-named helpers that the touch-up already replaced).

- [ ] **Step 8.2: CI gate**

```bash
uv run python scripts/quality.py ci
```

Expected: all green.

- [ ] **Step 8.3: Commit any last-mile cleanup**

If the grep finds leftover references, fix them in place and commit.

```bash
git add -A
git commit -m "refactor(ui): last-mile screen touch-up stragglers

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Delete legacy runtime files

With every runtime responsibility absorbed into the core app shell + integrations, the `src/yoyopod/runtime/` package can be emptied.

- [ ] **Step 9.1: Delete leftover runtime files**

```bash
git rm -r src/yoyopod/runtime/
```

Update any `__init__.py` re-exports that referenced the deleted runtime files.

- [ ] **Step 9.2: CI gate**

```bash
uv run python scripts/quality.py ci
```

- [ ] **Step 9.3: Commit**

```bash
git add -A
git commit -m "refactor: delete src/yoyopod/runtime/ (all services folded into integrations)

RuntimeBootService, RuntimeLoopService, ShutdownLifecycleService,
RuntimeEventWiring, PowerRuntimeService, ScreenPowerService, and
VoiceRuntimeCoordinator have all been absorbed into YoyoPodApp.run()
(Plan 2) + the appropriate integrations.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 10: Final verification

- [ ] **Step 10.1: Structure**

```bash
ls src/yoyopod/integrations/
# expected: __init__.py, call/, cloud/, contacts/, diagnostics/, focus/, location/, music/, network/, power/, recovery/, screen/, voice/

ls src/yoyopod/
# expected: app.py (legacy shell, rewritten in Plan 8), main.py, __init__.py, app_context.py (deleted in Plan 8), core/, backends/, integrations/, ui/, cli/, config/, audio/, device/
```

- [ ] **Step 10.2: All 11 integrations + recovery present**

```bash
ls src/yoyopod/integrations/ | wc -l
```

Expected: 12 entries (`__init__.py` + 11 integration directories). Counting: call, cloud, contacts, diagnostics, focus, location, music, network, power, recovery, screen, voice = 12 directories + `__init__.py`.

- [ ] **Step 10.3: CI + Pi validate (if hardware available)**

```bash
uv run python scripts/quality.py ci
```

If hardware available, run:
```bash
yoyopod pi validate deploy
yoyopod pi validate smoke
```

- [ ] **Step 10.4: Branch history**

Expected ~15 new commits on top of Plan 6.

---

## Definition of Done

- `integrations/recovery/` populated and tested.
- All 17 screens (home, hub, menu, navigation/*, system/power, music/*, voip/*) migrated to single-`app` constructor.
- `ScreenManager` constructs screens with `app` only.
- `src/yoyopod/runtime/` deleted entirely.
- `uv run python scripts/quality.py ci` green.

---

## What's next (Plan 8)

Dead-code removal + final sweep — delete `fsm.py`, `app_context.py`, old `event_bus.py`, old `events.py`, `coordinators/` entirely, shrink `app.py` to a ~150 LOC composition-root shim, archive `docs/RUNTIME_EVENT_FLOW.md`, mark spec `Status: Implemented`, run full Pi validate.

---

*End of implementation plan.*
