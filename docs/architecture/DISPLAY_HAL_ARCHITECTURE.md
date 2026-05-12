# YoYoPod Display HAL Architecture

**Last updated:** 2026-05-12
**Status:** Current implementation

This document describes the live Rust-owned display path used by the LVGL-only
runtime.

## Goals

- keep screen code hardware-agnostic
- use one render contract for both hardware and simulation
- avoid duplicate preview-only layout engines

## Current Files

- `device/ui/src/main.rs`: Rust UI host entrypoint
- `device/ui/src/transport/`: typed UI worker protocol loop
- `device/ui/src/render/lvgl/`: native LVGL scene backend and screen controllers
- `device/ui/src/hardware/`: hardware display/button adapters
- `device/ui/native/lvgl/`: pinned upstream LVGL C build configuration

## Architecture

```text
yoyopod-runtime
  -> yoyopod-ui-host worker
     -> Rust scene state
        -> Rust-owned LVGL scene backend
        -> hardware flush path or explicit mock display surface
```

## Supported Runtime Surfaces

### Whisplay hardware

- `240x280`
- portrait
- PiSugar Whisplay HAT
- LVGL-only production path

### Pimoroni / ST7789 hardware

- `320x240`
- landscape
- LVGL-backed adapter over ST7789 SPI plus GPIO control

### Preview

The Rust UI host can run in explicit mock hardware mode for protocol and render
smoke checks. It is not the default startup mode; callers must choose a hardware
mode and production uses Whisplay.

## Production Contract

- Non-simulated Whisplay runs use LVGL as the only app UI renderer.
- If the Whisplay driver, board init, or LVGL backend is unavailable, startup fails loudly.
- There is no supported PIL renderer or alternate production display backend in the current runtime.

## Summary

The display HAL is implemented and frozen around LVGL-backed Whisplay, Pimoroni,
and simulation adapter surfaces.
