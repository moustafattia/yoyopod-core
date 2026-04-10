# Hold-to-Ask Voice Command Shortcut

**Date:** 2026-04-10
**Status:** Draft
**Depends on:** `docs/ASK_SCREEN_DESIGN_SPEC.md` (unified Ask screen)

---

## Problem

Voice commands require navigating Hub → Ask → Voice Commands before speaking. For a device-control shortcut ("call mom", "volume up"), this is too many steps. The single Whisplay button has an unused gesture on the Hub: hold is mapped to BACK, which is a no-op on the root screen.

## Solution

Hold the button on the Hub to instantly enter push-to-talk voice command mode. The device listens while the button is held down, processes the command on release, shows a brief result, and auto-returns to Hub.

---

## Interaction Flow

```
User holds button on Hub
  → 800ms threshold: BACK fires → Hub pushes Ask(listening)
  → Ask starts recording immediately
  → User speaks while holding button
  → User releases button
  → PTT_RELEASE fires → Ask stops recording
  → "Thinking" state (transcribe + match)
  → Execute command
  → "Reply" state (show result for 2s)
  → Auto-pop back to Hub
```

### Navigating commands (call, play music)

When a command triggers its own navigation (call screen, now_playing), that navigation takes priority. No auto-return — the user is on the destination screen.

```
Hub hold → Ask(listening) → "call mom" → push outgoing_call
Hub hold → Ask(listening) → "play music" → push now_playing
```

### Informational commands (volume, mute, errors)

When the result is purely informational, show the reply for 2 seconds, then auto-pop.

```
Hub hold → Ask(listening) → "volume up" → "Volume is 75" (2s) → pop to Hub
Hub hold → Ask(listening) → (no speech) → "No Speech" (2s) → pop to Hub
```

---

## Convention: Hold on Root Screens

This is not a Hub-specific hack. The rule is:

> On any screen where `on_back` has nowhere to pop to (root screens), hold opens the Ask voice shortcut instead.

Today the Hub is the only root screen. If more root screens are added, they inherit this behavior.

---

## PTT Adapter Change

The PTT adapter currently fires `BACK` on button **release** after a hold. For push-to-talk, we need two events: one at the hold threshold (to start listening) and one on release (to stop listening).

### Current behavior

```
press → hold 800ms → (nothing) → release → BACK fires
```

### New behavior

```
press → hold 800ms → BACK fires immediately → release → PTT_RELEASE fires
```

### Implementation

In `PTTInputAdapter._poll_button`, add threshold-crossing detection in navigation mode (similar to the existing `raw_ptt_passthrough` path but for BACK):

```python
# In the poll loop, when button is held past threshold:
elif (
    current_state
    and previous_state
    and self.enable_navigation
    and not self._hold_back_fired      # new flag
    and self.press_start_time is not None
    and (current_time - self.press_start_time) >= self.long_press_time
):
    self._hold_back_fired = True
    self._fire_action(InputAction.BACK, {
        "method": "long_hold",
        "duration": current_time - self.press_start_time,
    })
```

In `_handle_button_release`, when BACK was already fired at threshold:

```python
if self._hold_back_fired:
    self._fire_action(InputAction.PTT_RELEASE, {
        "timestamp": current_time,
        "duration": press_duration,
        "after_hold": True,
    })
    # reset state, do NOT fire BACK again
    self._hold_back_fired = False
    self.press_start_time = None
    self.pending_single_tap_time = None
    self.double_tap_candidate = False
    return
```

### Effect on other screens

On non-root screens, BACK now fires at the hold threshold instead of on release. The user sees the screen pop immediately at 800ms rather than waiting for release. This is a minor timing improvement — the gesture is committed at 800ms either way. The subsequent `PTT_RELEASE` on release is ignored by screens that don't handle it (the base `Screen.on_ptt_release` is a no-op).

---

## Hub Screen Change

### `HubScreen.on_back`

Currently a no-op. Change to:

```python
def on_back(self, data=None) -> None:
    """Open Ask in quick-command mode (hold-to-ask shortcut)."""
    self.request_route("hold_ask")
```

### Route addition

```python
"hub": {
    # ... existing routes ...
    "hold_ask": NavigationRequest.push("ask"),
},
```

The `"hold_ask"` route pushes Ask with default payload. The Ask screen detects quick-command mode via an entry flag (see below).

---

## Ask Screen: Quick-Command Mode

The unified Ask screen (from `ASK_SCREEN_DESIGN_SPEC.md`) gains a `quick_command` entry mode in addition to its normal entry.

### Entry detection

The Ask screen's `enter()` method checks how it was entered:

```python
def enter(self) -> None:
    # ... existing setup ...
    if self._quick_command:
        # Skip idle, go straight to listening
        self._start_ptt_capture()
    else:
        self._state = "idle"
```

The route carries a payload that Ask checks. The `"hold_ask"` route uses `NavigationRequest.push("ask")` with no explicit payload. The Ask screen distinguishes quick-command entry from normal entry by checking whether the push came from the `"hold_ask"` route. The screen manager already passes the `NavigationRequest` to the screen; the Ask screen can check for a `quick_command` attribute set by the Hub before the push, or the Hub can call `ask_screen.set_quick_command(True)` directly before requesting the route. The chosen approach: Hub calls `set_quick_command(True)` on the Ask screen instance (accessed via the screen manager's registry) before emitting the route request.

### PTT capture flow

```python
def _start_ptt_capture(self) -> None:
    """Begin open-ended recording that stops on PTT_RELEASE."""
    self._state = "listening"
    self._headline = "Listening"
    self._body = "Speak now..."
    self._ptt_active = True
    self._capture_stop_event = Event()

    # Start capture with generous timeout; stop_event will end it early
    threading.Thread(
        target=self._run_ptt_capture,
        daemon=True,
    ).start()

def on_ptt_release(self, data=None) -> None:
    """Stop recording when the button is released."""
    if self._ptt_active and self._capture_stop_event is not None:
        self._ptt_active = False
        self._capture_stop_event.set()
```

### Stop vs cancel

The voice capture backend (`SubprocessAudioCaptureBackend._capture_vad`) already saves captured frames to WAV even when `cancel_event` is set — it just breaks the read loop. The discard logic is in the caller.

For PTT mode:
- `_capture_stop_event.set()` (from PTT_RELEASE) → stop recording, **process** the audio
- `_cancel_listening_cycle()` (from on_back/exit) → stop recording, **discard** the audio

The Ask screen distinguishes these by checking `self._ptt_active`: if it was cleared by `on_ptt_release`, the stop was intentional and the audio should be processed.

### Auto-return timer

After command execution in quick-command mode, if no outgoing navigation occurred:

```python
def _schedule_auto_return(self) -> None:
    """Pop back to Hub after 2 seconds."""
    if not self._quick_command:
        return
    threading.Timer(2.0, self._auto_pop).start()

def _auto_pop(self) -> None:
    """Return to the previous screen via the action scheduler."""
    scheduler = getattr(self.screen_manager, "action_scheduler", None)
    if scheduler is not None:
        scheduler(lambda: self.request_route("back"))
    else:
        self.request_route("back")
```

The auto-return is only active in quick-command mode. Normal Ask entry (from Hub select) stays on the reply screen until the user manually backs out.

---

## HintBar Text

The Hub footer should hint the hold shortcut. Current: `"Tap = Next · 2× = Open"`.

New: `"Tap = Next · 2× = Open · Hold = Ask"`

On the Ask screen in quick-command listening state, the hint shows: `"Release = Done · Hold = Cancel"` — but since the user is holding the button during this state, they may not see it. The hint is informational for completeness.

---

## What Changes (summary)

| File | Change |
|---|---|
| `yoyopy/ui/input/adapters/ptt_button.py` | Fire BACK at hold threshold (not release); fire PTT_RELEASE on release after hold |
| `yoyopy/ui/input/hal.py` | No change — PTT_RELEASE already exists in InputAction |
| `yoyopy/ui/screens/navigation/hub.py` | `on_back` pushes Ask via `hold_ask` route; update footer hint text |
| `yoyopy/ui/screens/router.py` | Add `"hold_ask"` route to `"hub"` routes |
| `yoyopy/ui/screens/navigation/ask.py` | Add `_quick_command` mode, PTT capture flow, `on_ptt_release` handler, auto-return timer |
| `yoyopy/ui/screens/base.py` | No change — `on_ptt_release` stub already exists |

### What does NOT change

- The Ask screen's visual states (idle/listening/thinking/reply) — same Figma spec
- Voice capture backend, STT backend, TTS backend
- Voice command matching and execution logic
- Non-root screen behavior (hold still means back)
- The four-button input path (Pimoroni) — unaffected

---

## Edge Cases

### User releases immediately after threshold (very short command)

The capture may contain little or no speech. The existing "No Speech" / "Not Recognized" handling covers this. Auto-return pops after 2 seconds.

### User holds for a very long time

The capture has a generous safety timeout (e.g. 30 seconds). If reached, capture stops and processes whatever was recorded.

### Multiple rapid hold-release cycles on Hub

Each hold-release is independent. BACK fires at threshold, pushes Ask. If the user releases immediately and Ask auto-pops, they're back on Hub for the next hold.

### Quick-command while music is playing

Music continues playing. Voice capture uses the mic, not the speaker. If the command is "volume up", volume changes while music plays. Same as current voice command behavior.

### Incoming call during quick-command

The call coordinator's existing interruption policy takes priority, pushing the incoming call screen on top of Ask. When the call ends, the user returns to Ask (or Hub if Ask auto-popped).

---

## Testing

### Unit tests

- PTT adapter: BACK fires at threshold, PTT_RELEASE on release, timing is correct
- Hub: `on_back` pushes Ask route
- Ask: quick-command mode skips idle, `on_ptt_release` stops capture, auto-return timer fires

### Integration tests

- Hold on Hub → Ask(listening) → release → thinking → reply → auto-pop to Hub
- Hold on Hub → "call mom" → outgoing_call screen (no auto-return)
- Hold on Hub → release immediately → "No Speech" → auto-pop

### Hardware validation

- On Pi with Whisplay: hold the physical button, speak a command, release, verify execution
- Verify timing feels natural (800ms threshold, 2s auto-return)
- Verify the attention beep plays at the right moment (threshold, not release)
