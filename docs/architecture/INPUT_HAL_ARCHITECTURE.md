# YoYoPod Input HAL Architecture

**Last updated:** 2026-05-12
**Status:** Implemented

This document describes the input abstraction layer that now exists in the UI package.

## Goals

- route semantic user actions instead of raw hardware buttons
- support multiple input sources
- decouple screen navigation from specific input devices
- keep simulation and hardware modes on the same action model

## Current Files

- `device/ui/src/input/`: Rust input model and hardware event normalization
- `device/ui/src/app/input_router.rs`: typed input action to app command routing
- `device/ui/src/transport/`: input events emitted through the typed worker protocol

## Core Typed Actions

- `Advance`
- `Select`
- `Back`
- `PttPress`
- `PttRelease`

## Runtime Mapping

Whisplay single-button hardware emits click-pattern actions for navigation. When
voice-note recording owns the screen, the same button path switches to typed PTT
passthrough so press and release surface as `PttPress` and `PttRelease`.

Runtime or test commands can send `UiCommand::InputAction`. The UI transport
emits every command and hardware action back as typed `UiEvent::Input`, then the
app layer produces typed intents when a screen owns the action.

This is the current call path:

```text
button or UiCommand::InputAction -> transport -> app::input_router -> UiRuntime -> typed UiIntent
```

## Screen Contract

Current behavior:

- screen behavior is driven by typed app commands, registry focus policy, and
  registry navigation policy
- screen-specific side effects are typed `UiIntent` values from
  `device/ui/src/app/intents.rs`
- there are no legacy `on_button_*()` or Python input compatibility contracts in
  the Rust UI domain

## Summary

Screen input is typed end to end in the Rust UI domain.
