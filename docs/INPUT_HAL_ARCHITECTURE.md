# YoyoPod Input HAL Architecture

**Last updated:** 2026-04-02
**Status:** Implemented with compatibility layer

This document describes the input abstraction layer that now exists in the UI package.

## Goals

- route semantic user actions instead of raw hardware buttons
- support multiple input sources
- decouple screen navigation from specific input devices
- keep simulation and hardware modes on the same action model

## Current Files

- `yoyopy/ui/input/input_hal.py`: `InputAction` and `InputHAL`
- `yoyopy/ui/input/input_manager.py`: action dispatcher
- `yoyopy/ui/input/input_factory.py`: adapter selection
- `yoyopy/ui/input/adapters/four_button.py`
- `yoyopy/ui/input/adapters/ptt_button.py`
- `yoyopy/ui/input/adapters/keyboard.py`

## Core Semantic Actions

- navigation: `SELECT`, `BACK`, `UP`, `DOWN`, `LEFT`, `RIGHT`, `MENU`, `HOME`
- playback: `PLAY_PAUSE`, `NEXT_TRACK`, `PREV_TRACK`, `VOLUME_UP`, `VOLUME_DOWN`
- VoIP: `CALL_ANSWER`, `CALL_REJECT`, `CALL_HANGUP`
- PTT: `PTT_PRESS`, `PTT_RELEASE`
- voice: `VOICE_COMMAND`

## Adapter Mapping

### FourButtonInputAdapter

Default mapping:

- `A -> SELECT`
- `B -> BACK`
- `X -> UP`
- `Y -> DOWN`
- long press `B -> HOME`

### PTTInputAdapter

- button press and release emit `PTT_PRESS` and `PTT_RELEASE`
- optional click patterns can emit `SELECT` and `BACK`

### KeyboardInputAdapter

Used in simulation mode:

- `Enter` or `Space -> SELECT`
- `Esc` or `Backspace -> BACK`
- `Up` or `K -> UP`
- `Down` or `J -> DOWN`

## How ScreenManager Uses It

`ScreenManager` connects semantic actions to the current screen when screens change.

This is the current call path:

```text
input adapter -> InputManager -> ScreenManager -> current screen handler
```

## Compatibility Bridge

The abstraction layer is present, but cleanup is not fully complete.

Current behavior:

- `ScreenManager` registers semantic callbacks
- `Screen` defines semantic methods like `on_select()`
- those methods currently forward to legacy `on_button_*()` handlers when a screen has not been fully migrated yet

That means the input HAL is implemented, but many concrete screens still rely on old button-named methods internally.

## Known Gaps

- full migration of concrete screens to semantic handlers is still pending
- some older demos and tests still import deleted pre-HAL input modules
- `keyboard.py` still has small naming drift between `get_supported_actions()` and the manager's `get_capabilities()` convention

## Summary

The input HAL is real and in use today. The remaining work is cleanup and removal of legacy screen handler names, not invention of the architecture itself.
