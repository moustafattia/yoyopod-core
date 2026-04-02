# YoyoPod Display HAL Architecture

**Last updated:** 2026-04-02
**Status:** Implemented

This document describes the display abstraction layer that is now present in the codebase.

## Goals

- support multiple display backends behind one API
- keep screen code hardware-agnostic
- preserve a backward-compatible `Display` facade
- allow simulation without physical hardware

## Current Files

- `yoyopy/ui/display/display_hal.py`: HAL interface
- `yoyopy/ui/display/display_manager.py`: `Display` facade
- `yoyopy/ui/display/display_factory.py`: adapter selection and auto-detection
- `yoyopy/ui/display/adapters/pimoroni.py`
- `yoyopy/ui/display/adapters/whisplay.py`
- `yoyopy/ui/display/adapters/simulation.py`

## Architecture

```text
Display
  -> get_display(...)
     -> DisplayHAL implementation
        -> PimoroniDisplayAdapter
        -> WhisplayDisplayAdapter
        -> SimulationDisplayAdapter
```

## Supported Adapters

### PimoroniDisplayAdapter

- 320x240
- landscape
- Display HAT Mini

### WhisplayDisplayAdapter

- 240x280
- portrait
- PiSugar Whisplay HAT

### SimulationDisplayAdapter

- 240x280
- portrait
- browser rendering through `yoyopy/ui/web_server.py`

## Backward Compatibility Contract

Concrete screens and app code still create `Display(...)`, not hardware-specific adapters.

The facade exposes:

- dimensions and orientation
- shared color constants
- drawing primitives
- status bar rendering
- backlight control
- text measurement
- cleanup

## Selection Rules

`display_factory.py` currently chooses hardware using:

1. explicit `display.hardware` config
2. `YOYOPOD_DISPLAY`
3. Whisplay driver path detection
4. `displayhatmini` import success
5. simulation fallback

## Known Gaps

- Whisplay discovery still depends on a hardcoded driver path
- simulation is portrait-first even when mimicking other display classes
- there is no pluggable adapter registration mechanism yet

## Summary

The display HAL is no longer a proposal. It exists and is the current display architecture used by `YoyoPodApp`.
