# Architecture

```text
yoyopod.py / yoyopy/main.py  (entry points)
        |
    YoyoPodApp (app.py) -- central coordinator
    |- MusicFSM + CallFSM (fsm.py) -- composed playback and call state machines
    |- CoordinatorRuntime (coordinators/runtime.py) -- derived app state
    |- AppContext (app_context.py) -- shared state
    |- MopidyClient (audio/mopidy_client.py) -- Mopidy JSON-RPC
    |- VoIPManager (voip/manager.py) -- linphonec subprocess
    |- Display HAL (ui/display/) -- factory pattern, 3 adapters
    |- Input HAL (ui/input/) -- semantic actions, 3 adapters
    `- ScreenManager (ui/screens/manager.py) -- stack-based navigation
```

## Display HAL

`ui/display/`: `DisplayHAL` interface -> factory -> adapters (pimoroni, whisplay, simulation). Facade via `Display` in `manager.py`.

## Input HAL

`ui/input/`: Semantic actions (SELECT, BACK, UP, DOWN). Adapters: `four_button.py`, `ptt_button.py`, `keyboard.py`. Manager dispatches actions to active screen.

## Screen System

`ui/screens/`: Base class in `base.py`, stack-based manager in `manager.py`. Feature screens organized in `navigation/`, `music/`, `voip/` subdirectories.

## State Orchestration

`MusicFSM` and `CallFSM` stay independent, while `CoordinatorRuntime` derives combined runtime states such as `PLAYING_WITH_VOIP`, `PAUSED_BY_CALL`, and `CALL_ACTIVE_MUSIC_PAUSED`. Auto-pauses music on incoming calls and auto-resumes after call ends (configurable).

## Key Patterns

- Ring tone generated via `speaker-test` subprocess (800Hz on `plughw:1`)
- Screen stack: incoming call pushes screens, `_pop_call_screens()` pops all call screens on hangup to prevent stack overflow
- VoIP monitor thread reads linphonec output continuously; callbacks fire on the coordinator thread for UI updates
- EventBus serializes typed app events on the coordinator thread
