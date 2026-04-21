# Phase A — Plan 8: Dead-Code Removal + Final Sweep Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Final Phase A housekeeping. Delete all remaining legacy files whose functionality has been fully absorbed into `core/` + `integrations/`. Rewrite `src/yoyopod/app.py` as a slim ~150-LOC composition root. Archive or delete stale documentation. Pass the full Pi hardware validation suite. Mark the Phase A spec `Status: Implemented`.

**Architecture:** No new code. Deletions + `app.py` rewrite + docs hygiene.

**Spec reference:** spec §9.1 (deleted outright), §11.2 steps 12-13, §14 Definition of Done.

**Prerequisite:** Plans 1-7 executed. All 12 integrations in `integrations/`. All 17 screens use `app` constructor. `src/yoyopod/runtime/` gone. `src/yoyopod/coordinators/` reduced to `__init__.py` + any stragglers.

---

## File Structure

### Files to rewrite

- `src/yoyopod/app.py` — replace legacy shell with a ~150 LOC composition-root that constructs `YoyoPodApp`, registers integrations, runs.

### Files to delete

- `src/yoyopod/fsm.py` (MusicFSM, CallFSM, CallInterruptionPolicy — already delete-pending since Plan 2, confirm no remaining consumers)
- `src/yoyopod/event_bus.py` (replaced by `core/bus.py`)
- `src/yoyopod/events.py` (replaced by `core/events.py` + per-integration events)
- `src/yoyopod/coordinators/` (entire directory — CoordinatorRuntime, AppRuntimeState, any stragglers)
- `src/yoyopod/app_context.py` (AppContext)
- `src/yoyopod/communication/` (whole package — manager.py, calling/, integrations/, messaging/, models.py — all relocated)
- Any empty legacy packages whose files have been moved away

### Docs to archive or update

- `docs/RUNTIME_EVENT_FLOW.md` — archive (move to `docs/archive/`)
- `docs/SYSTEM_ARCHITECTURE.md` — update to reflect new architecture
- `docs/superpowers/specs/2026-04-21-phase-a-spine-rewrite-design.md` — mark `Status: Implemented`
- `CLAUDE.md` — update "Source of Truth" file list

---

## Task 1: Branch state verification

- [ ] **Step 1.1**

```bash
git log --oneline -40
ls src/yoyopod/integrations/
uv run pytest tests/ -q
```

Expected: Plans 1-7 all landed; all tests green; 12 integrations populated; `src/yoyopod/runtime/` gone.

- [ ] **Step 1.2: Inventory legacy files remaining**

```bash
ls -la src/yoyopod/fsm.py src/yoyopod/event_bus.py src/yoyopod/events.py 2>/dev/null
ls -la src/yoyopod/coordinators/ 2>/dev/null
ls -la src/yoyopod/communication/ 2>/dev/null
ls -la src/yoyopod/app_context.py 2>/dev/null
```

Record what's still present. Anything that's already gone, skip in the corresponding delete step.

---

## Task 2: Delete legacy FSMs + event bus + events + app_context

- [ ] **Step 2.1: Confirm no consumers outside already-scheduled-for-rewrite files**

```bash
grep -rn "from yoyopod.fsm\|from yoyopod.event_bus\|from yoyopod.events\|from yoyopod.app_context" src/ tests/
```

Consumers: expected only in `src/yoyopod/app.py` (legacy shell, about to be rewritten) and `src/yoyopod/main.py` (entry point, may need update). If a test file still references these, it's slated for deletion per spec §12.1 or needs migration.

- [ ] **Step 2.2: Delete legacy files**

```bash
git rm src/yoyopod/fsm.py
git rm src/yoyopod/event_bus.py
git rm src/yoyopod/events.py
git rm src/yoyopod/app_context.py
```

- [ ] **Step 2.3: Delete legacy tests that are no longer applicable**

Per spec §12.1:
```bash
git rm tests/test_fsm_runtime.py 2>/dev/null || true
# tests/test_event_bus.py was rewritten in Plan 1 — keep or delete based on current state
```

- [ ] **Step 2.4: Commit**

```bash
git add -A
git commit -m "refactor: delete legacy fsm.py, event_bus.py, events.py, app_context.py

All functionality absorbed into src/yoyopod/core/.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Delete coordinators/ directory

- [ ] **Step 3.1: Confirm no consumers**

```bash
grep -rn "from yoyopod.coordinators\|CoordinatorRuntime\|AppRuntimeState" src/ tests/
```

Expected: matches only in docs/ (spec/plans) and possibly `app.py` (to be rewritten).

- [ ] **Step 3.2: Delete**

```bash
git rm -r src/yoyopod/coordinators/
```

- [ ] **Step 3.3: Commit**

```bash
git add -A
git commit -m "refactor: delete src/yoyopod/coordinators/ entirely

CallCoordinator, PlaybackCoordinator, ScreenCoordinator, PowerCoordinator,
CoordinatorRuntime, AppRuntimeState all obsolete. Replaced by integrations +
state store + focus arbiter.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Delete communication/ directory

- [ ] **Step 4.1: Confirm all files relocated**

```bash
ls src/yoyopod/communication/
```

Expected: `__init__.py`, maybe empty `calling/`, `integrations/`, `messaging/` directories. Anything non-empty here means relocation in Plan 6 missed a file — fix now.

- [ ] **Step 4.2: Delete**

```bash
git rm -r src/yoyopod/communication/
```

- [ ] **Step 4.3: Commit**

```bash
git add -A
git commit -m "refactor: delete src/yoyopod/communication/ (all code relocated)

VoIPManager -> integrations/call/
LiblinphoneBackend -> backends/voip/
Models -> integrations/call/models.py

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Rewrite `src/yoyopod/app.py`

Replace the legacy ~685 LOC shell with a slim composition root that constructs `YoyoPodApp`, loads config, registers integrations, and runs.

- [ ] **Step 5.1: Write the new `src/yoyopod/app.py`**

```python
"""YoyoPod application entry point.

Constructs a YoyoPodApp with config, registers integrations in dependency
order, runs the main loop. See docs/superpowers/specs/2026-04-21-phase-a-
spine-rewrite-design.md §3, §11.2 step 13.
"""

from __future__ import annotations

import signal
import sys
from typing import Any

from loguru import logger

from yoyopod.config import load_config
from yoyopod.core import YoyoPodApp


# Integration registration order matters:
# 1. diagnostics first so the event log captures later lifecycle events.
# 2. recovery next so later integrations can register retry handlers.
# 3. focus before music/call/voice (arbiter must exist first).
# 4. contacts before call (call looks up names).
# 5. network -> location (location shares modem serial).
# 6. cloud, power at any time once bus/states/services exist.
# 7. screen needs its ui_tick_callback set before run().
# 8. music before call (call acquires focus which pre-empts music).
INTEGRATION_MODULES = [
    "yoyopod.integrations.diagnostics",
    "yoyopod.integrations.recovery",
    "yoyopod.integrations.focus",
    "yoyopod.integrations.contacts",
    "yoyopod.integrations.network",
    "yoyopod.integrations.location",
    "yoyopod.integrations.cloud",
    "yoyopod.integrations.power",
    "yoyopod.integrations.screen",
    "yoyopod.integrations.voice",
    "yoyopod.integrations.music",
    "yoyopod.integrations.call",
]


def build_app(config_dir: str = "config", simulate: bool = False) -> YoyoPodApp:
    """Construct and wire the app. Does not start the main loop."""
    config = load_config(config_dir=config_dir, simulate=simulate)

    app = YoyoPodApp(log_capacity=500)
    app.config = config

    for dotted in INTEGRATION_MODULES:
        module = _import(dotted)
        name = dotted.rsplit(".", 1)[-1]
        setup = getattr(module, "setup")
        teardown = getattr(module, "teardown", None)
        app.register_integration(name, setup=setup, teardown=teardown)

    return app


def run(config_dir: str = "config", simulate: bool = False) -> None:
    """Construct, setup, and run until SIGINT/SIGTERM."""
    logger.info("=" * 60)
    logger.info("YoyoPod starting (simulate={})", simulate)
    logger.info("=" * 60)

    app = build_app(config_dir=config_dir, simulate=simulate)

    def handle_signal(signum: int, _frame: Any) -> None:
        logger.info("Signal {} received; stopping", signum)
        app.stop()

    signal.signal(signal.SIGINT, handle_signal)
    signal.signal(signal.SIGTERM, handle_signal)

    try:
        app.setup()
        app.run(tick_interval_seconds=0.005)
    finally:
        app.stop()

    logger.info("YoyoPod stopped")


def _import(dotted: str):
    import importlib
    return importlib.import_module(dotted)


if __name__ == "__main__":
    simulate = "--simulate" in sys.argv
    run(simulate=simulate)
```

- [ ] **Step 5.2: Verify line count**

```bash
wc -l src/yoyopod/app.py
```

Expected: around 80 LOC (slimmer than the target "150 LOC" in the spec — even better).

- [ ] **Step 5.3: Format/lint/type**

```bash
uv run black src/yoyopod/app.py
uv run ruff check src/yoyopod/app.py
uv run mypy src/yoyopod/app.py
```

- [ ] **Step 5.4: Update `src/yoyopod/main.py`**

```python
"""Package entry point — delegates to yoyopod.app.run."""

from __future__ import annotations

from yoyopod.app import run


def main() -> None:
    import sys
    simulate = "--simulate" in sys.argv
    run(simulate=simulate)


if __name__ == "__main__":
    main()
```

- [ ] **Step 5.5: CI gate**

```bash
uv run python scripts/quality.py ci
```

Expected: all green.

- [ ] **Step 5.6: Commit**

```bash
git add -A
git commit -m "refactor(app): rewrite app.py as slim composition root

685 LOC -> ~80 LOC. Constructs YoyoPodApp, registers 12 integrations in
dependency order, handles SIGINT/SIGTERM, runs the main loop.
All behaviour-specific logic lives in integrations; app.py is pure wiring.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Archive stale documentation

- [ ] **Step 6.1: Move `RUNTIME_EVENT_FLOW.md` to archive**

```bash
mkdir -p docs/archive
git mv docs/RUNTIME_EVENT_FLOW.md docs/archive/RUNTIME_EVENT_FLOW.md
```

- [ ] **Step 6.2: Update `docs/SYSTEM_ARCHITECTURE.md`**

Replace the existing topology section with a new one pointing at the A+3 model:

```markdown
## Runtime architecture (post-Phase-A)

```
yoyopod.app:run()
  -> build_app(config_dir, simulate)
     -> YoyoPodApp (core/app_shell.py)
        -> Bus, States, Services, Scheduler, LogBuffer (core/)
        -> registered integrations (integrations/):
           diagnostics, recovery, focus, contacts,
           network, location, cloud, power,
           screen, voice, music, call
  -> app.setup()  (calls each integration's setup(app))
  -> app.run()    (4-line loop: drain scheduler, drain bus, tick UI, sleep)
```

Integrations consume backends from `src/yoyopod/backends/`. Cross-domain
coordination is via the focus integration's AudioFocus arbiter and
typed events on the bus (see docs/superpowers/specs/2026-04-21-phase-a-
spine-rewrite-design.md §6-7 for the event and command catalog).
```

- [ ] **Step 6.3: Update `CLAUDE.md` "Source of Truth" section**

Update the list of canonical files:
- Remove: `src/yoyopod/fsm.py`, `src/yoyopod/event_bus.py`, `src/yoyopod/events.py`, `src/yoyopod/coordinators/`, `src/yoyopod/app_context.py`.
- Add: `src/yoyopod/core/`, `src/yoyopod/integrations/`, `src/yoyopod/backends/`.

Update the "Runtime Architecture" ASCII diagram to match the new shape.

- [ ] **Step 6.4: Commit doc updates**

```bash
git add -A
git commit -m "docs: archive RUNTIME_EVENT_FLOW.md, update SYSTEM_ARCHITECTURE + CLAUDE.md

Post-Phase-A architecture documented. Source-of-truth file list points at
core/, integrations/, backends/ instead of legacy fsm/event_bus/coordinators.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Mark spec `Status: Implemented`

- [ ] **Step 7.1: Edit the Phase A design spec header**

Open `docs/superpowers/specs/2026-04-21-phase-a-spine-rewrite-design.md`. Change:

```markdown
**Status:** Awaiting review
```

to:

```markdown
**Status:** Implemented (2026-04-21 — this is the date the final Phase A PR merged; replace with actual merge date)
```

- [ ] **Step 7.2: Commit**

```bash
git add docs/superpowers/specs/2026-04-21-phase-a-spine-rewrite-design.md
git commit -m "docs: mark Phase A spec as Implemented

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Full CI + Pi hardware validation

This is the pre-merge gate from spec §12.6.

- [ ] **Step 8.1: Full local CI**

```bash
uv run python scripts/quality.py ci
```

Expected: all green. Any failure is a blocker.

- [ ] **Step 8.2: Pi deploy smoke**

```bash
yoyopod pi validate deploy
```

Expected: green. The app starts on the Pi without error.

- [ ] **Step 8.3: Pi validate suite**

```bash
yoyopod pi validate smoke
yoyopod pi validate music
yoyopod pi validate voip
yoyopod pi validate stability
```

Expected: each green. Any regression is a blocker — debug before merging.

- [ ] **Step 8.4: Manual on-Pi checklist**

Exercise on real hardware:

1. Place an outgoing call to a test SIP address; verify audio flows both ways; hang up.
2. Receive an incoming call while music is playing. Verify:
   - Music auto-pauses (focus arbiter works).
   - Incoming-call screen appears with caller ID.
   - Answer → audio flows → hang up.
3. Receive an incoming call and reject it. Verify call history records it as missed.
4. Record a voice note, review, send. Verify the recipient receives it (if the SIP peer supports SIMPLE).
5. Graceful shutdown via power command. Verify clean exit.
6. Wake from RTC alarm; verify clock sync.
7. Kill mpv on the Pi (`pkill mpv`) and verify the recovery integration detects it and restarts.
8. Kill Liblinphone activity and verify SIP re-registers.

- [ ] **Step 8.5: Event-log review**

Tail `~/.yoyopod/logs/events.jsonl` during the manual checklist. Confirm:
- Every call state transition is logged.
- Every music state change is logged.
- Every focus grant/loss is logged.
- No unexpected errors appear.
- No `ResponsivenessLagEvent` appears unless intentionally stressing the device.

---

## Task 9: Merge to main

- [ ] **Step 9.1: Rebase if main has moved**

```bash
git fetch origin
git rebase origin/main
```

Resolve conflicts if any (unlikely — main was frozen for Phase A).

- [ ] **Step 9.2: Push and open PR**

```bash
git push -u origin arch/phase-a-spine-rewrite
gh pr create --title "Phase A: spine rewrite (state store + typed bus + service registry)" --body "$(cat <<'EOF'
## Summary

Rewrites the YoyoPod spine to a Home-Assistant-style state store + typed
event bus + service registry. Replaces the pseudo-reactive event bus
(9 of 14 events pub-to-self) with direct-call integrations. Deletes
FSMs, coordinators, runtime services, AppContext, VoIPManager, old
event bus. `app.py` shrinks from ~685 to ~80 LOC.

See docs/superpowers/specs/2026-04-21-phase-a-spine-rewrite-design.md
for the full design and docs/superpowers/plans/2026-04-21-phase-a-*.md
for the 8 implementation plans that produced this branch.

## Test plan

- [ ] `uv run python scripts/quality.py ci` green
- [ ] `yoyopod pi validate deploy` green on Pi
- [ ] `yoyopod pi validate smoke` green on Pi
- [ ] `yoyopod pi validate music` green on Pi
- [ ] `yoyopod pi validate voip` green on Pi
- [ ] `yoyopod pi validate stability` green on Pi
- [ ] Manual on-Pi checklist (place/receive call, music auto-pause, voice note, shutdown, recovery)
- [ ] Event-log review during manual tests — no unexpected errors, no ResponsivenessLagEvent

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 9.3: Squash-merge when PR is green**

After review and all green checks:
```bash
gh pr merge --squash
```

---

## Definition of Done (Phase A complete)

- `src/yoyopod/core/` (primitives), `src/yoyopod/integrations/` (12 domains), `src/yoyopod/backends/` (adapters) all populated.
- Legacy files/directories deleted: `fsm.py`, `event_bus.py`, `events.py`, `app_context.py`, `coordinators/`, `runtime/`, `communication/`.
- `src/yoyopod/app.py` ≤ 150 LOC (actually ~80).
- Spec marked `Implemented`.
- Pi validation suite all green.
- Branch merged to main.

---

## What's next (Phase B)

HAL consolidation — see `docs/superpowers/specs/2026-04-21-phase-b-hal-consolidation-design.md`.

---

*End of implementation plan.*
