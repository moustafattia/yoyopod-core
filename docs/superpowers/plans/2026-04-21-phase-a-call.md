# Phase A — Plan 6: Call Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the biggest single piece of the spine — `VoIPManager` (618 LOC) + `CallCoordinator` (520 LOC) + `CallFSM` + `CallInterruptionPolicy` — into `integrations/call/`. This is where the 8-hop pseudo-reactive chain collapses to a 5-hop direct-call chain. After this plan lands, the maintainability pain point described in the Phase A spec §1 is gone.

**Architecture:** `LiblinphoneBackend` moves to `src/yoyopod/backends/voip/`. VoIPManager's responsibilities distribute across `integrations/call/{handlers,messaging,voice_notes,history}.py`. Call lifecycle is no longer managed by a FSM object — `call.state` entity is the source of truth, handlers transition it on backend events. Focus integration handles call-interrupts-music (no `CallInterruptionPolicy` needed).

**Tech Stack:** Python 3.12+, pytest, uv, existing Liblinphone shim + cffi binding.

**Spec reference:** spec §3.2, §5 (call entities), §7 (call commands), §9.1 (VoIPManager/CallCoordinator/CallFSM/CallInterruptionPolicy deletion), §9.2.

**Prerequisite:** Plans 1-5 executed; `focus` and `music` integrations working.

---

## File Structure

### Files to create

- `src/yoyopod/backends/voip/__init__.py`
- `src/yoyopod/backends/voip/liblinphone.py` (moved from `src/yoyopod/communication/calling/backend.py`)
- `src/yoyopod/backends/voip/binding.py` (moved from `src/yoyopod/communication/integrations/liblinphone_binding/binding.py`)
- `src/yoyopod/backends/voip/shim_native/` (moved from `src/yoyopod/communication/integrations/liblinphone_binding/native/` — keep structure)
- `src/yoyopod/integrations/call/__init__.py`
- `src/yoyopod/integrations/call/commands.py`
- `src/yoyopod/integrations/call/events.py`
- `src/yoyopod/integrations/call/handlers.py`
- `src/yoyopod/integrations/call/messaging.py` (moved from `src/yoyopod/communication/calling/messaging.py`)
- `src/yoyopod/integrations/call/voice_notes.py` (moved from `src/yoyopod/communication/calling/voice_notes.py`)
- `src/yoyopod/integrations/call/history.py` (moved from `src/yoyopod/communication/calling/history.py`)
- `src/yoyopod/integrations/call/models.py` (moved from `src/yoyopod/communication/models.py`)
- `tests/integrations/test_call.py`

### Files to delete (after migration)

- `src/yoyopod/communication/calling/manager.py` (VoIPManager)
- `src/yoyopod/communication/calling/backend.py` (moved)
- `src/yoyopod/communication/calling/messaging.py` (moved)
- `src/yoyopod/communication/calling/voice_notes.py` (moved)
- `src/yoyopod/communication/calling/history.py` (moved)
- `src/yoyopod/communication/models.py` (moved)
- `src/yoyopod/communication/calling/__init__.py`
- `src/yoyopod/communication/__init__.py`
- `src/yoyopod/communication/messaging/store.py` and `__init__.py` (likely stays or moves — check)
- `src/yoyopod/coordinators/call.py` (CallCoordinator)

---

## Task 1: Branch state verification

- [ ] **Step 1.1**

```bash
git log --oneline -25
ls src/yoyopod/integrations/
uv run pytest tests/ -q
```

Expected: 10 integrations present; all tests green.

---

## Task 2: Relocate Liblinphone backend

- [ ] **Step 2.1: Move backend files**

```bash
mkdir -p src/yoyopod/backends/voip
git mv src/yoyopod/communication/calling/backend.py src/yoyopod/backends/voip/liblinphone.py
git mv src/yoyopod/communication/integrations/liblinphone_binding/binding.py src/yoyopod/backends/voip/binding.py
```

The native shim directory may contain C source; move the whole directory:

```bash
git mv src/yoyopod/communication/integrations/liblinphone_binding/native src/yoyopod/backends/voip/shim_native
# If there's also an __init__.py at binding level:
[ -f src/yoyopod/communication/integrations/liblinphone_binding/__init__.py ] && git rm src/yoyopod/communication/integrations/liblinphone_binding/__init__.py
```

- [ ] **Step 2.2: Move shared communication models**

```bash
git mv src/yoyopod/communication/models.py src/yoyopod/integrations/call/models.py
```

- [ ] **Step 2.3: Update imports everywhere**

```bash
grep -rn "from yoyopod.communication.calling.backend\|from yoyopod.communication.integrations.liblinphone_binding\|from yoyopod.communication.models\|from yoyopod.communication import" src/ tests/
```

Rewrite to new paths:
- `yoyopod.communication.calling.backend` → `yoyopod.backends.voip.liblinphone`
- `yoyopod.communication.integrations.liblinphone_binding.binding` → `yoyopod.backends.voip.binding`
- `yoyopod.communication.models` → `yoyopod.integrations.call.models`
- `yoyopod.communication import CallHistoryStore, VoIPManager` → (CallHistoryStore moves later; VoIPManager is about to die — stub temporarily)

- [ ] **Step 2.4: Create `src/yoyopod/backends/voip/__init__.py`**

```python
"""Liblinphone-based VoIP backend adapter."""

from __future__ import annotations

from yoyopod.backends.voip.liblinphone import LiblinphoneBackend, VoIPBackend

__all__ = ["LiblinphoneBackend", "VoIPBackend"]
```

- [ ] **Step 2.5: Run the current VoIP tests to confirm imports still resolve**

```bash
uv run pytest tests/test_voip_backend.py -v
```

Expected: pass (test exercises the adapter through the new import path).

- [ ] **Step 2.6: Commit**

```bash
git add -A
git commit -m "refactor(voip): move Liblinphone backend + binding to backends/voip/

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Relocate messaging, voice_notes, history submodules

- [ ] **Step 3.1: Create `integrations/call/` structure and move files**

```bash
mkdir -p src/yoyopod/integrations/call
git mv src/yoyopod/communication/calling/messaging.py src/yoyopod/integrations/call/messaging.py
git mv src/yoyopod/communication/calling/voice_notes.py src/yoyopod/integrations/call/voice_notes.py
git mv src/yoyopod/communication/calling/history.py src/yoyopod/integrations/call/history.py
```

- [ ] **Step 3.2: Move the message store**

```bash
git mv src/yoyopod/communication/messaging/store.py src/yoyopod/integrations/call/message_store.py
```

- [ ] **Step 3.3: Update internal imports inside moved files**

Grep + rewrite:
```bash
grep -rn "from yoyopod.communication" src/yoyopod/integrations/call/
```

Targets:
- `from yoyopod.communication.models import …` → `from yoyopod.integrations.call.models import …`
- `from yoyopod.communication.calling.backend` → `from yoyopod.backends.voip.liblinphone`
- `from yoyopod.communication.messaging.store` → `from yoyopod.integrations.call.message_store`

- [ ] **Step 3.4: Run CI gate**

```bash
uv run python scripts/quality.py ci
```

Any import errors — fix in place. Tests that asserted exact class locations may need path updates.

- [ ] **Step 3.5: Commit**

```bash
git add -A
git commit -m "refactor(call): relocate messaging, voice_notes, history under integrations/call/

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Write call commands

- [ ] **Step 4.1: Create `src/yoyopod/integrations/call/commands.py`**

```python
"""Typed commands for the call integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class DialCommand:
    """Place an outgoing call to the given SIP address."""

    address: str
    display_name: str = ""


@dataclass(frozen=True, slots=True)
class AnswerCommand:
    """Answer the incoming call."""


@dataclass(frozen=True, slots=True)
class HangupCommand:
    """End the current call."""


@dataclass(frozen=True, slots=True)
class RejectCommand:
    """Reject an incoming call."""


@dataclass(frozen=True, slots=True)
class MuteCommand:
    """Mute the local microphone."""


@dataclass(frozen=True, slots=True)
class UnmuteCommand:
    """Unmute the local microphone."""


@dataclass(frozen=True, slots=True)
class SendMessageCommand:
    """Send a text message to the given SIP address."""

    recipient_address: str
    text: str
    display_name: str = ""


@dataclass(frozen=True, slots=True)
class StartVoiceNoteCommand:
    """Begin recording a voice note for a recipient."""

    recipient_address: str
    recipient_name: str = ""


@dataclass(frozen=True, slots=True)
class StopVoiceNoteCommand:
    """Stop recording; prepare the draft for review/send."""


@dataclass(frozen=True, slots=True)
class SendVoiceNoteCommand:
    """Send the currently active voice-note draft."""


@dataclass(frozen=True, slots=True)
class CancelVoiceNoteCommand:
    """Discard the in-progress voice-note recording."""


@dataclass(frozen=True, slots=True)
class PlayVoiceNoteCommand:
    """Play a recorded voice note back through the local speaker."""

    file_path: str


@dataclass(frozen=True, slots=True)
class MarkVoiceNotesSeenCommand:
    """Clear unread-voice-note markers for the given SIP address."""

    address: str
```

- [ ] **Step 4.2: Commit**

```bash
git add src/yoyopod/integrations/call/commands.py
git commit -m "feat(integrations/call): 13 typed commands

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Write call events

- [ ] **Step 5.1: Create `src/yoyopod/integrations/call/events.py`**

```python
"""Domain events for the call integration."""

from __future__ import annotations

from dataclasses import dataclass

from yoyopod.integrations.call.models import (
    CallState,
    RegistrationState,
    VoIPMessageRecord,
)


@dataclass(frozen=True, slots=True)
class CallBackendStateEvent:
    """Raw backend-reported call state — internal to the call integration."""

    state: CallState
    caller_address: str = ""
    reason: str = ""


@dataclass(frozen=True, slots=True)
class CallIncomingEvent:
    """An incoming call has arrived and ringing should start."""

    caller_address: str
    caller_name: str


@dataclass(frozen=True, slots=True)
class MessageReceivedEvent:
    """An inbound text message or media payload arrived."""

    record: VoIPMessageRecord


@dataclass(frozen=True, slots=True)
class MessageDeliveryChangedEvent:
    """Outbound message delivery state updated."""

    record: VoIPMessageRecord


@dataclass(frozen=True, slots=True)
class VoiceNoteCompletedEvent:
    """A voice-note recording finished and is ready for review/send."""

    draft_path: str
    recipient_address: str


@dataclass(frozen=True, slots=True)
class RegistrationChangedEvent:
    """SIP registration state updated."""

    state: RegistrationState
    reason: str = ""
```

- [ ] **Step 5.2: Commit**

```bash
git add src/yoyopod/integrations/call/events.py
git commit -m "feat(integrations/call): domain events

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Write call handlers

This is the meat — the logic that was spread across `VoIPManager._handle_backend_event`, `CallCoordinator.handle_call_state_change`, `handle_call_ended`, and the pause/resume shuttle.

- [ ] **Step 6.1: Create `src/yoyopod/integrations/call/handlers.py`**

```python
"""Call-lifecycle handlers.

Consumes CallBackendStateEvent from the backend and mutates `call.state` /
`call.caller` / `call.registration` entities. Requests audio focus on incoming
or outgoing call start; releases focus when call ends. Auto-resume of music is
handled by the music integration via its AudioFocusLostEvent subscriber +
re-request on our release (the focus arbiter fires events naturally).
"""

from __future__ import annotations

from typing import Any

from loguru import logger

from yoyopod.integrations.call.events import (
    CallBackendStateEvent,
    CallIncomingEvent,
    RegistrationChangedEvent,
)
from yoyopod.integrations.call.models import CallState, RegistrationState
from yoyopod.integrations.focus.arbiter import release_focus, request_focus


def handle_backend_state(app: Any, event: CallBackendStateEvent) -> None:
    """Main dispatcher for backend-reported call state changes."""
    state = event.state
    logger.info("Call backend state: {}", state.value)

    if state == CallState.INCOMING:
        _enter_incoming(app, event.caller_address)
        return

    if state in (
        CallState.OUTGOING,
        CallState.OUTGOING_PROGRESS,
        CallState.OUTGOING_RINGING,
        CallState.OUTGOING_EARLY_MEDIA,
    ):
        _enter_outgoing(app, event.caller_address)
        return

    if state in (CallState.CONNECTED, CallState.STREAMS_RUNNING):
        _enter_active(app)
        return

    if state in (CallState.RELEASED, CallState.END, CallState.ERROR):
        _enter_idle(app, reason=state.value)
        return


def _enter_incoming(app: Any, caller_address: str) -> None:
    name = _resolve_contact_name(app, caller_address)
    app.states.set(
        "call.state",
        "incoming",
        attrs={"caller_address": caller_address, "caller_name": name},
    )
    app.states.set("call.caller", {"address": caller_address, "display_name": name})
    request_focus(app, "call")
    app.bus.publish(CallIncomingEvent(caller_address=caller_address, caller_name=name))


def _enter_outgoing(app: Any, callee_address: str) -> None:
    name = _resolve_contact_name(app, callee_address)
    app.states.set(
        "call.state",
        "outgoing",
        attrs={"callee_address": callee_address, "callee_name": name},
    )
    app.states.set("call.caller", {"address": callee_address, "display_name": name})
    request_focus(app, "call")


def _enter_active(app: Any) -> None:
    current = app.states.get("call.state")
    attrs = dict(current.attrs) if current else {}
    app.states.set("call.state", "active", attrs=attrs)


def _enter_idle(app: Any, reason: str = "") -> None:
    app.states.set("call.state", "idle", attrs={"reason": reason})
    app.states.set("call.caller", None)
    app.states.set("call.muted", False)
    release_focus(app, "call")


def handle_registration_changed(app: Any, event: RegistrationChangedEvent) -> None:
    logger.info("SIP registration: {}", event.state.value)
    app.states.set(
        "call.registration",
        event.state.value,
        attrs={"reason": event.reason} if event.reason else None,
    )


def _resolve_contact_name(app: Any, address: str) -> str:
    """Look up a display name for an address via the contacts integration."""
    try:
        from yoyopod.integrations.contacts.commands import LookupByAddressCommand
        contact = app.services.call(
            "contacts", "lookup_by_address", LookupByAddressCommand(address=address)
        )
        if contact is not None:
            return str(getattr(contact, "display_name", address))
    except Exception as exc:
        logger.debug("Contact lookup failed for {}: {}", address, exc)
    return _extract_username(address)


def _extract_username(address: str | None) -> str:
    if not address:
        return "Unknown"
    if "@" in address:
        username_part = address.split("@", 1)[0]
        if ":" in username_part:
            return username_part.split(":")[-1]
        return username_part
    return address
```

- [ ] **Step 6.2: Commit**

```bash
git add src/yoyopod/integrations/call/handlers.py
git commit -m "feat(integrations/call): backend-state handlers + registration handler

Implements the call lifecycle state machine as state-entity transitions
plus focus requests. No more CallFSM, CallCoordinator, or
CallInterruptionPolicy — the focus arbiter does all the cross-domain
coordination by publishing AudioFocusLostEvent.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Write call integration `setup()` / `teardown()`

- [ ] **Step 7.1: Create `src/yoyopod/integrations/call/__init__.py`**

```python
"""Call integration: SIP calls, messages, voice notes, history, registration."""

from __future__ import annotations

from typing import Any

from loguru import logger

from yoyopod.integrations.call.commands import (
    AnswerCommand,
    CancelVoiceNoteCommand,
    DialCommand,
    HangupCommand,
    MarkVoiceNotesSeenCommand,
    MuteCommand,
    PlayVoiceNoteCommand,
    RejectCommand,
    SendMessageCommand,
    SendVoiceNoteCommand,
    StartVoiceNoteCommand,
    StopVoiceNoteCommand,
    UnmuteCommand,
)
from yoyopod.integrations.call.events import (
    CallBackendStateEvent,
    MessageDeliveryChangedEvent,
    MessageReceivedEvent,
    RegistrationChangedEvent,
    VoiceNoteCompletedEvent,
)
from yoyopod.integrations.call.handlers import (
    handle_backend_state,
    handle_registration_changed,
)
from yoyopod.integrations.call.models import (
    CallState,
    IncomingCallDetected,
    CallStateChanged,
    RegistrationStateChanged,
    MessageReceived,
    MessageDeliveryChanged,
    MessageFailed,
    BackendStopped,
)

_STATE_KEY = "_call_integration"


def setup(
    app: Any,
    backend: Any | None = None,
    message_store: Any | None = None,
    call_history: Any | None = None,
) -> None:
    if backend is None:
        from yoyopod.backends.voip import LiblinphoneBackend
        backend = LiblinphoneBackend(app.config.voip)

    if message_store is None:
        from yoyopod.integrations.call.message_store import VoIPMessageStore
        from pathlib import Path
        store_dir = Path(app.config.voip.message_store_dir)
        store_dir.mkdir(parents=True, exist_ok=True)
        message_store = VoIPMessageStore(store_dir)

    if call_history is None:
        from yoyopod.integrations.call.history import CallHistoryStore
        call_history = CallHistoryStore(app.config.voip)
        call_history.load()

    # Initial state.
    app.states.set("call.state", "idle")
    app.states.set("call.muted", False)
    app.states.set("call.registration", "none")
    app.states.set("call.history_unread_count", call_history.missed_count())

    # Route backend events into typed CallBackendStateEvent + others on the bus.
    def route_backend_event(ev: Any) -> None:
        if isinstance(ev, CallStateChanged):
            app.scheduler.run_on_main(
                lambda: app.bus.publish(
                    CallBackendStateEvent(state=ev.state, caller_address="")
                )
            )
        elif isinstance(ev, IncomingCallDetected):
            app.scheduler.run_on_main(
                lambda: app.bus.publish(
                    CallBackendStateEvent(state=CallState.INCOMING, caller_address=ev.caller_address)
                )
            )
        elif isinstance(ev, RegistrationStateChanged):
            app.scheduler.run_on_main(
                lambda: app.bus.publish(
                    RegistrationChangedEvent(state=ev.state)
                )
            )
        elif isinstance(ev, MessageReceived):
            app.scheduler.run_on_main(
                lambda msg=ev.message: _on_message_received(app, message_store, msg)
            )
        elif isinstance(ev, MessageDeliveryChanged):
            app.scheduler.run_on_main(
                lambda msg=ev.message: _on_message_delivery_changed(app, message_store, msg)
            )
        elif isinstance(ev, MessageFailed):
            app.scheduler.run_on_main(
                lambda reason=ev.reason, mid=ev.message_id: logger.error(
                    "Message {} failed: {}", mid, reason
                )
            )
        elif isinstance(ev, BackendStopped):
            app.scheduler.run_on_main(
                lambda reason=ev.reason: logger.warning("Liblinphone backend stopped: {}", reason)
            )

    backend.on_event(route_backend_event)

    # Subscribe to typed bus events — dispatch to handlers.
    app.bus.subscribe(
        CallBackendStateEvent,
        lambda ev: handle_backend_state(app, ev),
    )
    app.bus.subscribe(
        RegistrationChangedEvent,
        lambda ev: handle_registration_changed(app, ev),
    )

    # Commands.
    def handle_dial(cmd: DialCommand) -> None:
        backend.make_call(cmd.address, display_name=cmd.display_name)

    def handle_answer(_cmd: AnswerCommand) -> None:
        backend.answer_call()

    def handle_hangup(_cmd: HangupCommand) -> None:
        backend.hangup()

    def handle_reject(_cmd: RejectCommand) -> None:
        backend.reject_call()

    def handle_mute(_cmd: MuteCommand) -> None:
        if backend.mute():
            app.states.set("call.muted", True)

    def handle_unmute(_cmd: UnmuteCommand) -> None:
        if backend.unmute():
            app.states.set("call.muted", False)

    def handle_send_message(cmd: SendMessageCommand) -> None:
        from yoyopod.integrations.call.messaging import MessagingService
        service = MessagingService(
            config=app.config.voip,
            backend=backend,
            message_store=message_store,
            lookup_contact_name=lambda addr: _lookup(app, addr),
        )
        service.send_text_message(cmd.recipient_address, cmd.text, cmd.display_name)

    def handle_start_voice_note(cmd: StartVoiceNoteCommand) -> None:
        from yoyopod.integrations.call.voice_notes import VoiceNoteService
        service = VoiceNoteService(
            config=app.config.voip,
            backend=backend,
            message_store=message_store,
            lookup_contact_name=lambda addr: _lookup(app, addr),
            notify_message_summary_change=lambda: None,
        )
        service.start_voice_note_recording(cmd.recipient_address, cmd.recipient_name)
        setattr(app, _STATE_KEY + "_voice_note_service", service)

    def handle_stop_voice_note(_cmd: StopVoiceNoteCommand) -> None:
        service = getattr(app, _STATE_KEY + "_voice_note_service", None)
        if service is None:
            return
        draft = service.stop_voice_note_recording()
        if draft is not None:
            app.bus.publish(
                VoiceNoteCompletedEvent(
                    draft_path=draft.file_path,
                    recipient_address=draft.recipient_address,
                )
            )

    def handle_send_voice_note(_cmd: SendVoiceNoteCommand) -> None:
        service = getattr(app, _STATE_KEY + "_voice_note_service", None)
        if service is not None:
            service.send_active_voice_note()

    def handle_cancel_voice_note(_cmd: CancelVoiceNoteCommand) -> None:
        service = getattr(app, _STATE_KEY + "_voice_note_service", None)
        if service is not None:
            service.cancel_voice_note_recording()

    def handle_play_voice_note(cmd: PlayVoiceNoteCommand) -> None:
        service = getattr(app, _STATE_KEY + "_voice_note_service", None)
        if service is not None:
            service.play_voice_note(cmd.file_path)

    def handle_mark_seen(cmd: MarkVoiceNotesSeenCommand) -> None:
        call_history.mark_voice_notes_seen(cmd.address)
        app.states.set("call.history_unread_count", call_history.missed_count())

    app.services.register("call", "dial", handle_dial)
    app.services.register("call", "answer", handle_answer)
    app.services.register("call", "hangup", handle_hangup)
    app.services.register("call", "reject", handle_reject)
    app.services.register("call", "mute", handle_mute)
    app.services.register("call", "unmute", handle_unmute)
    app.services.register("call", "send_message", handle_send_message)
    app.services.register("call", "start_voice_note", handle_start_voice_note)
    app.services.register("call", "stop_voice_note", handle_stop_voice_note)
    app.services.register("call", "send_voice_note", handle_send_voice_note)
    app.services.register("call", "cancel_voice_note", handle_cancel_voice_note)
    app.services.register("call", "play_voice_note", handle_play_voice_note)
    app.services.register("call", "mark_voice_notes_seen", handle_mark_seen)

    backend.start()

    setattr(app, _STATE_KEY, {
        "backend": backend,
        "message_store": message_store,
        "call_history": call_history,
    })


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    try:
        state["backend"].stop()
    except Exception as exc:
        logger.error("VoIP backend stop: {}", exc)
    delattr(app, _STATE_KEY)
    if hasattr(app, _STATE_KEY + "_voice_note_service"):
        delattr(app, _STATE_KEY + "_voice_note_service")


def _on_message_received(app: Any, message_store: Any, record: Any) -> None:
    message_store.upsert(record)
    app.bus.publish(MessageReceivedEvent(record=record))


def _on_message_delivery_changed(app: Any, message_store: Any, record: Any) -> None:
    message_store.upsert(record)
    app.bus.publish(MessageDeliveryChangedEvent(record=record))


def _lookup(app: Any, address: str) -> str:
    try:
        from yoyopod.integrations.contacts.commands import LookupByAddressCommand
        contact = app.services.call(
            "contacts", "lookup_by_address", LookupByAddressCommand(address=address)
        )
        if contact is not None:
            return str(getattr(contact, "display_name", address))
    except Exception:
        pass
    if "@" in address:
        return address.split("@", 1)[0]
    return address
```

- [ ] **Step 7.2: Create `tests/integrations/test_call.py`**

```python
from dataclasses import dataclass, field
from typing import Callable

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.call import setup as setup_call, teardown as teardown_call
from yoyopod.integrations.call.commands import (
    AnswerCommand,
    DialCommand,
    HangupCommand,
    MuteCommand,
    UnmuteCommand,
)
from yoyopod.integrations.call.events import CallIncomingEvent
from yoyopod.integrations.call.models import (
    CallState,
    CallStateChanged,
    IncomingCallDetected,
    RegistrationState,
    RegistrationStateChanged,
)
from yoyopod.integrations.focus import setup as setup_focus, teardown as teardown_focus


@dataclass
class _FakeVoipBackend:
    event_cb: Callable | None = None
    started: bool = False
    stopped: bool = False
    calls_made: list[str] = field(default_factory=list)
    answered: int = 0
    hung_up: int = 0
    rejected: int = 0
    muted: bool = False

    def on_event(self, cb):
        self.event_cb = cb

    def start(self):
        self.started = True
        return True

    def stop(self):
        self.stopped = True

    def make_call(self, address, display_name=""):
        self.calls_made.append(address)

    def answer_call(self):
        self.answered += 1

    def hangup(self):
        self.hung_up += 1

    def reject_call(self):
        self.rejected += 1

    def mute(self):
        self.muted = True
        return True

    def unmute(self):
        self.muted = False
        return True

    def simulate_incoming(self, caller="sip:bob@x"):
        if self.event_cb:
            self.event_cb(IncomingCallDetected(caller_address=caller))

    def simulate_state(self, state: CallState):
        if self.event_cb:
            self.event_cb(CallStateChanged(state=state))

    def simulate_registration(self, state: RegistrationState):
        if self.event_cb:
            self.event_cb(RegistrationStateChanged(state=state))


class _FakeMessageStore:
    def upsert(self, record):
        pass


class _FakeCallHistory:
    def __init__(self):
        self._unread = 0

    def load(self):
        pass

    def missed_count(self):
        return self._unread

    def mark_voice_notes_seen(self, address):
        pass


@pytest.fixture
def app_with_call():
    app = build_test_app()
    backend = _FakeVoipBackend()
    message_store = _FakeMessageStore()
    call_history = _FakeCallHistory()
    app.config = type("C", (), {"voip": type("VC", (), {"message_store_dir": "/tmp"})()})()

    app.register_integration("focus", setup=lambda a: setup_focus(a), teardown=lambda a: teardown_focus(a))
    app.register_integration(
        "call",
        setup=lambda a: setup_call(
            a, backend=backend, message_store=message_store, call_history=call_history
        ),
        teardown=lambda a: teardown_call(a),
    )
    app.setup()
    yield app, backend
    app.stop()


def test_setup_initial_state(app_with_call):
    app, _ = app_with_call
    assert app.states.get_value("call.state") == "idle"
    assert app.states.get_value("call.muted") is False
    assert app.states.get_value("call.registration") == "none"


def test_incoming_call_updates_state_and_acquires_focus(app_with_call):
    app, backend = app_with_call
    captured: list[CallIncomingEvent] = []
    app.bus.subscribe(CallIncomingEvent, lambda ev: captured.append(ev))

    backend.simulate_incoming(caller="sip:alice@x")
    app.drain()

    assert app.states.get_value("call.state") == "incoming"
    assert app.states.get_value("focus.owner") == "call"
    assert len(captured) == 1
    assert captured[0].caller_address == "sip:alice@x"


def test_outgoing_call_acquires_focus(app_with_call):
    app, backend = app_with_call
    app.services.call("call", "dial", DialCommand(address="sip:bob@x"))
    backend.simulate_state(CallState.OUTGOING)
    app.drain()

    assert app.states.get_value("call.state") == "outgoing"
    assert app.states.get_value("focus.owner") == "call"
    assert backend.calls_made == ["sip:bob@x"]


def test_connected_transitions_to_active(app_with_call):
    app, backend = app_with_call
    backend.simulate_incoming()
    app.drain()
    backend.simulate_state(CallState.CONNECTED)
    app.drain()

    assert app.states.get_value("call.state") == "active"


def test_released_transitions_to_idle_releases_focus(app_with_call):
    app, backend = app_with_call
    backend.simulate_incoming()
    app.drain()
    backend.simulate_state(CallState.CONNECTED)
    app.drain()
    backend.simulate_state(CallState.RELEASED)
    app.drain()

    assert app.states.get_value("call.state") == "idle"
    assert app.states.get_value("focus.owner") is None


def test_answer_command_invokes_backend(app_with_call):
    app, backend = app_with_call
    app.services.call("call", "answer", AnswerCommand())
    assert backend.answered == 1


def test_hangup_command_invokes_backend(app_with_call):
    app, backend = app_with_call
    app.services.call("call", "hangup", HangupCommand())
    assert backend.hung_up == 1


def test_mute_unmute_cycle(app_with_call):
    app, backend = app_with_call
    app.services.call("call", "mute", MuteCommand())
    assert app.states.get_value("call.muted") is True
    assert backend.muted is True

    app.services.call("call", "unmute", UnmuteCommand())
    assert app.states.get_value("call.muted") is False
    assert backend.muted is False


def test_registration_change_updates_state(app_with_call):
    app, backend = app_with_call
    backend.simulate_registration(RegistrationState.OK)
    app.drain()

    assert app.states.get_value("call.registration") == "ok"
```

- [ ] **Step 7.3: Run, commit**

```bash
uv run pytest tests/integrations/test_call.py -v
uv run black src/yoyopod/integrations/call/ tests/integrations/test_call.py
uv run ruff check src/yoyopod/integrations/call/ tests/integrations/test_call.py
uv run mypy src/yoyopod/integrations/call/
git add -A
git commit -m "feat(integrations/call): setup/teardown + 13 commands + event-driven lifecycle

VoIPManager's 618 LOC dissolved. CallCoordinator's 520 LOC gone.
CallFSM and CallInterruptionPolicy gone. Call state lives in call.state
entity; focus integration handles call-pre-empts-music.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Delete legacy VoIPManager, CallCoordinator

- [ ] **Step 8.1: Enumerate**

```bash
grep -rn "VoIPManager\|CallCoordinator\|CallFSM\|CallInterruptionPolicy" src/ tests/
```

- [ ] **Step 8.2: Delete**

```bash
git rm src/yoyopod/communication/calling/manager.py
git rm src/yoyopod/communication/calling/__init__.py
git rm src/yoyopod/coordinators/call.py
```

Update `src/yoyopod/coordinators/__init__.py` (remove CallCoordinator re-export).

Stub `src/yoyopod/app.py` references (Plan 8 does the full rewrite).

- [ ] **Step 8.3: CI gate**

```bash
uv run python scripts/quality.py ci
```

- [ ] **Step 8.4: Commit**

```bash
git add -A
git commit -m "refactor(call): delete VoIPManager, CallCoordinator, CallFSM, CallInterruptionPolicy

All logic migrated to integrations/call/ and integrations/focus/.
~1,200 LOC removed from the spine.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Cross-integration test — incoming call auto-pauses music

This is the canonical proof that the rewrite achieves its primary goal.

- [ ] **Step 9.1: Create `tests/e2e/test_call_pauses_music.py`**

```bash
mkdir -p tests/e2e
touch tests/e2e/__init__.py
```

Create `tests/e2e/test_call_pauses_music.py`:

```python
from dataclasses import dataclass, field
from typing import Callable

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.call import setup as setup_call, teardown as teardown_call
from yoyopod.integrations.call.commands import HangupCommand
from yoyopod.integrations.call.models import (
    CallState,
    CallStateChanged,
    IncomingCallDetected,
    RegistrationState,
    RegistrationStateChanged,
)
from yoyopod.integrations.focus import setup as setup_focus, teardown as teardown_focus
from yoyopod.integrations.music import setup as setup_music, teardown as teardown_music
from yoyopod.integrations.music.commands import PlayCommand


@dataclass
class _FakeMpv:
    playback_cb: Callable | None = None
    track_cb: Callable | None = None
    avail_cb: Callable | None = None
    state: str = "idle"
    play_calls: list[str] = field(default_factory=list)
    pause_calls: int = 0

    def start(self):
        return True
    def stop(self):
        self.state = "idle"
        if self.playback_cb: self.playback_cb("idle")
    def shutdown(self):
        pass
    def play(self, uri, start_position=0.0):
        self.play_calls.append(uri)
        self.state = "playing"
        if self.playback_cb: self.playback_cb("playing")
    def pause(self):
        self.pause_calls += 1
        self.state = "paused"
        if self.playback_cb: self.playback_cb("paused")
    def resume(self):
        self.state = "playing"
        if self.playback_cb: self.playback_cb("playing")
    def next_track(self): pass
    def prev_track(self): pass
    def seek(self, pos): pass
    def set_volume(self, p): pass
    def on_playback_state_change(self, cb): self.playback_cb = cb
    def on_track_change(self, cb): self.track_cb = cb
    def on_availability_change(self, cb): self.avail_cb = cb


@dataclass
class _FakeVoip:
    event_cb: Callable | None = None
    def on_event(self, cb): self.event_cb = cb
    def start(self): return True
    def stop(self): pass
    def make_call(self, address, display_name=""): pass
    def answer_call(self): pass
    def hangup(self): pass
    def reject_call(self): pass
    def mute(self): return True
    def unmute(self): return True
    def simulate_incoming(self, caller="sip:bob@x"):
        if self.event_cb: self.event_cb(IncomingCallDetected(caller_address=caller))
    def simulate_state(self, state: CallState):
        if self.event_cb: self.event_cb(CallStateChanged(state=state))


class _FakeLibrary:
    def record_recent_track(self, t): pass


class _FakeMsgStore:
    def upsert(self, r): pass


class _FakeCallHistory:
    def load(self): pass
    def missed_count(self): return 0
    def mark_voice_notes_seen(self, a): pass


@pytest.fixture
def app_full():
    app = build_test_app()
    mpv = _FakeMpv()
    voip = _FakeVoip()
    app.config = type("C", (), {
        "audio": type("AC", (), {"default_volume": 70})(),
        "voip": type("VC", (), {"message_store_dir": "/tmp"})(),
    })()
    app.register_integration("focus", setup=lambda a: setup_focus(a), teardown=lambda a: teardown_focus(a))
    app.register_integration(
        "music",
        setup=lambda a: setup_music(a, backend=mpv, library=_FakeLibrary()),
        teardown=lambda a: teardown_music(a),
    )
    app.register_integration(
        "call",
        setup=lambda a: setup_call(
            a, backend=voip, message_store=_FakeMsgStore(), call_history=_FakeCallHistory()
        ),
        teardown=lambda a: teardown_call(a),
    )
    app.setup()
    yield app, voip, mpv
    app.stop()


def test_incoming_call_auto_pauses_playing_music(app_full):
    app, voip, mpv = app_full
    app.services.call("music", "play", PlayCommand(track_uri="local:track.mp3"))
    app.drain()
    assert app.states.get_value("music.state") == "playing"
    assert app.states.get_value("focus.owner") == "music"

    voip.simulate_incoming(caller="sip:bob@x")
    app.drain()

    assert app.states.get_value("call.state") == "incoming"
    assert app.states.get_value("focus.owner") == "call"
    assert mpv.pause_calls == 1
    assert app.states.get_value("music.state") == "paused"


def test_call_released_leaves_music_paused_until_user_resumes(app_full):
    app, voip, mpv = app_full
    app.services.call("music", "play", PlayCommand(track_uri="local:x.mp3"))
    app.drain()

    voip.simulate_incoming()
    app.drain()
    voip.simulate_state(CallState.CONNECTED)
    app.drain()
    voip.simulate_state(CallState.RELEASED)
    app.drain()

    assert app.states.get_value("call.state") == "idle"
    assert app.states.get_value("focus.owner") is None
    # Music did NOT auto-resume — the user decides via explicit command.
    assert app.states.get_value("music.state") == "paused"
```

- [ ] **Step 9.2: Run**

```bash
uv run pytest tests/e2e/test_call_pauses_music.py -v
```

Expected: both passing.

- [ ] **Step 9.3: Commit**

```bash
git add -A
git commit -m "test(e2e): incoming call auto-pauses music via focus arbiter

Demonstrates the primary goal of the Phase A rewrite: cross-domain
coordination happens via focus integration with zero direct dependency
between music and call.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 10: Final verification

- [ ] **Step 10.1: Structure**

```bash
ls src/yoyopod/integrations/call/
ls src/yoyopod/backends/voip/
```

Expected call: `__init__.py`, `commands.py`, `events.py`, `handlers.py`, `messaging.py`, `voice_notes.py`, `history.py`, `message_store.py`, `models.py`.
Backend: `__init__.py`, `liblinphone.py`, `binding.py`, `shim_native/`.

- [ ] **Step 10.2: No legacy classes linger**

```bash
git grep -l "VoIPManager\|CallCoordinator\|CallFSM\|CallInterruptionPolicy"
```

Expected: matches only in docs/ (spec/plan references).

- [ ] **Step 10.3: CI gate**

```bash
uv run python scripts/quality.py ci
```

Expected: all green.

---

## Definition of Done

- `integrations/call/` fully populated with 13 commands, 6 event types, handlers for call-state lifecycle + registration.
- `backends/voip/` contains relocated Liblinphone adapter + binding + native shim.
- VoIPManager, CallCoordinator, CallFSM, CallInterruptionPolicy all deleted.
- End-to-end test demonstrates call-pauses-music via focus integration.
- All tests green.

---

## What's next (Plan 7)

`recovery` integration (re-home `RecoverySupervisor`), followed by touch-up of all 17 screens to consume `app.states` + `app.services` instead of manager references. This is the UI-side work that makes Phase A feel complete.

---

*End of implementation plan.*
