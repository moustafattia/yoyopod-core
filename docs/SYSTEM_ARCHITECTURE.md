# YoyoPod System Architecture

**Last updated:** 2026-04-02
**Status:** Current implementation

This document describes the architecture that exists on `main` after the UI HAL refactor.

## Overview

YoyoPod runs as a single Python application that coordinates:

- display rendering
- semantic input handling
- screen navigation
- Mopidy music playback
- SIP calling through `linphonec`
- state transitions between playback and call flows

The production entrypoint is `yoyopod.py`, which delegates to `YoyoPodApp` in `yoyopy/app.py`.

## Runtime Topology

```text
yoyopod.py / yoyopy.main
  -> YoyoPodApp
     -> Display facade
        -> Display factory
           -> PimoroniDisplayAdapter | WhisplayDisplayAdapter | SimulationDisplayAdapter
     -> InputManager
        -> FourButtonInputAdapter | PTTInputAdapter | KeyboardInputAdapter
     -> ScreenManager
        -> navigation screens
        -> music screens
        -> voip screens
     -> MusicFSM / CallFSM / CallInterruptionPolicy
     -> CoordinatorRuntime
     -> AppContext
     -> MopidyClient
        -> Mopidy JSON-RPC over HTTP
     -> VoIPManager
        -> linphonec subprocess
```

## Package Structure

### Application Layer

- `yoyopy/app.py`: main coordinator
- `yoyopy/main.py`: package entry point
- `yoyopy/fsm.py`: split music and call state models
- `yoyopy/coordinators/runtime.py`: derived app runtime state
- `yoyopy/app_context.py`: shared app state

### Audio and VoIP

- `yoyopy/audio/mopidy_client.py`: playlist loading, playback control, polling callbacks
- `yoyopy/connectivity/voip_manager.py`: registration, call state parsing, call control

### UI Layer

- `yoyopy/ui/display/`: display HAL and adapters
- `yoyopy/ui/input/`: input HAL, semantic actions, adapters
- `yoyopy/ui/screens/`: screen classes split by feature
- `yoyopy/ui/web_server.py`: simulation browser server

## Display Architecture

`Display` in `yoyopy/ui/display/display_manager.py` is a facade over the HAL interface in `yoyopy/ui/display/display_hal.py`.

Supported adapters:

- `PimoroniDisplayAdapter`: 320x240 landscape
- `WhisplayDisplayAdapter`: 240x280 portrait
- `SimulationDisplayAdapter`: browser-rendered portrait simulation

Selection happens in `yoyopy/ui/display/display_factory.py` using:

1. explicit `display.hardware` config
2. `YOYOPOD_DISPLAY` environment variable
3. auto-detection
4. simulation fallback

## Input Architecture

`InputManager` dispatches semantic actions, not hardware button names.

Core semantic actions:

- navigation: `SELECT`, `BACK`, `UP`, `DOWN`, `LEFT`, `RIGHT`, `MENU`, `HOME`
- playback: `PLAY_PAUSE`, `NEXT_TRACK`, `PREV_TRACK`
- VoIP: `CALL_ANSWER`, `CALL_REJECT`, `CALL_HANGUP`
- PTT: `PTT_PRESS`, `PTT_RELEASE`

Current adapters:

- `FourButtonInputAdapter`: Pimoroni A/B/X/Y mapping
- `PTTInputAdapter`: Whisplay single-button mapping
- `KeyboardInputAdapter`: simulation keyboard controls

## Screen Architecture

`ScreenManager` owns a stack of screen instances and reconnects input handlers when the active screen changes.

Screen groups:

- `yoyopy/ui/screens/navigation/`
- `yoyopy/ui/screens/music/`
- `yoyopy/ui/screens/voip/`

There is still a compatibility bridge in `Screen`:

- semantic handlers like `on_select()` exist
- most concrete screens still implement legacy `on_button_*()` methods
- `Screen` currently forwards semantic actions to those legacy methods

That bridge is intentional but temporary.

## State Coordination

Playback and call orchestration now use composed models:

- `MusicFSM` in `yoyopy/fsm.py`
- `CallFSM` in `yoyopy/fsm.py`
- `CallInterruptionPolicy` in `yoyopy/fsm.py`
- `CoordinatorRuntime` in `yoyopy/coordinators/runtime.py`

`CoordinatorRuntime` derives the current application status from those models, including:

- `PLAYING_WITH_VOIP`
- `PAUSED_BY_CALL`
- `CALL_ACTIVE_MUSIC_PAUSED`

`YoyoPodApp` listens to:

- Mopidy playback changes
- VoIP registration changes
- VoIP call state changes

and updates:

- screen stack
- music pause/resume behavior
- state machine state

## Event Flows

### Incoming Call

1. `VoIPManager` reads `linphonec` output on a monitor thread
2. caller address and name are extracted
3. `YoyoPodApp._handle_incoming_call()` pauses music if needed
4. state transitions to `CALL_INCOMING`
5. `IncomingCallScreen` is pushed
6. ringing starts through `speaker-test`

### Music Playback

1. screen action triggers Mopidy RPC
2. `MopidyClient` polls track and playback state
3. callbacks refresh `NowPlayingScreen`
4. the derived runtime state stays synchronized with actual playback state

### Simulation Mode

1. `Display` chooses `SimulationDisplayAdapter`
2. `web_server.py` starts a Flask-SocketIO server
3. browser receives base64 PNG display updates
4. keyboard and web buttons feed `InputManager`

## Raspberry Pi Assumptions

The current code still includes a few environment-specific assumptions:

- Whisplay driver path: `/home/tifo/Whisplay/Driver/WhisPlay.py`
- audio device for Linphone and ringing: `plughw:1`
- simulation server defaults to port `5000`

These are known implementation constraints, not architecture goals.

## Source Of Truth

For current behavior, trust these files over older notes or demos:

- `yoyopy/app.py`
- `yoyopy/fsm.py`
- `yoyopy/coordinators/runtime.py`
- `yoyopy/ui/display/`
- `yoyopy/ui/input/`
- `yoyopy/ui/screens/`
