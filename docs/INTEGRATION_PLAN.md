# YoyoPod VoIP + Music Integration Plan

**Last updated:** 2026-04-02
**Status:** Implemented

This document started as a plan and now serves as the completion record for the VoIP + Mopidy integration.

## What Is Implemented

- unified `YoyoPodApp` coordinator in `yoyopy/app.py`
- integrated state machine in `yoyopy/state_machine.py`
- music auto-pause on incoming call
- optional music auto-resume after call end
- screen stack transitions for incoming, outgoing, and active calls
- periodic now-playing refresh for progress updates
- simulation mode using the web server and keyboard input

## Current Implementation Map

### Coordinator

- `yoyopy/app.py`

Responsibilities:

- load config
- initialize display/input/screen infrastructure
- start VoIP and Mopidy managers
- register callbacks
- coordinate call interruption and resume behavior

### Music Layer

- `yoyopy/audio/mopidy_client.py`

Responsibilities:

- playback control
- playlist discovery and loading
- track/state polling callbacks

### VoIP Layer

- `yoyopy/connectivity/voip_manager.py`

Responsibilities:

- `linphonec` lifecycle
- registration state tracking
- call state parsing
- caller lookup and callback dispatch

### UI Layer

- `yoyopy/ui/display/`
- `yoyopy/ui/input/`
- `yoyopy/ui/screens/`

The older `display.py`, `screens.py`, `screen_manager.py`, and `input_handler.py` layout is no longer current.

## Integrated States

Key states used by the running app:

- `PLAYING`
- `PAUSED`
- `CALL_IDLE`
- `CALL_INCOMING`
- `CALL_OUTGOING`
- `CALL_ACTIVE`
- `PLAYING_WITH_VOIP`
- `PAUSED_BY_CALL`
- `CALL_ACTIVE_MUSIC_PAUSED`

See `yoyopy/state_machine.py` for the authoritative transition list.

## Incoming Call Flow

1. `VoIPManager` detects an incoming call from `linphonec` output
2. `YoyoPodApp` pauses music if playback is active
3. state changes to `CALL_INCOMING`
4. `IncomingCallScreen` is pushed
5. answer/reject actions route back through `VoIPManager`
6. call end clears call screens and optionally resumes music

## Outgoing Call Flow

1. user navigates to contacts
2. `VoIPManager.make_call()` sends the SIP call command
3. outgoing and in-call screens are pushed as call state changes arrive
4. call end pops call screens and returns the app to the prior playback state

## What Changed After The Original Plan

The original integration work predated the later UI refactor. The current code now uses:

- `InputManager` instead of `InputHandler`
- `Display` HAL instead of a single hardcoded display module
- split screen modules instead of one `screens.py`

The integration behavior is still real and current; the old file references were not.

## Remaining Cleanup Outside Integration

The integration itself is done. Remaining repository work is separate:

- update stale docs and packaging metadata
- migrate older demos and tests to current UI imports
- finish semantic input migration inside screen implementations
- reduce hardcoded hardware assumptions
