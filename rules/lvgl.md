# LVGL Display Pipeline

Applies to: `device/ui/**`

## Overview

LVGL 9.5.0 is the standard rendering layer for all hardware displays. The architecture is display-hardware-agnostic -- LVGL renders to its internal buffer, then flush callbacks route pixels to whatever SPI display is connected.

For the Figma-to-Whisplay implementation workflow, screen extraction order, and hardware validation loop, also follow `rules/design-fidelity.md`.

## Production Contract

- Non-simulated Whisplay runs are a production LVGL path.
- Whisplay hardware uses LVGL as the only app UI renderer; there is no renderer selection knob.
- If the Whisplay driver, hardware init, or LVGL backend is unavailable, startup must fail loudly instead of silently degrading to another renderer.
- The Rust runtime software preview reuses the Whisplay LVGL render contract and browser preview transport; it is not a separate PIL renderer.
- Simulation also requires native LVGL. If native LVGL is missing, the correct fix is to build it, not to fall back to PIL.

## Rendering Pipeline

```
LVGL object tree
  -> partial render (40-line draw buffer)
  -> flush callback (RGB565_SWAPPED)
  -> Rust display bridge in `device/ui/src/renderer/lvgl/`
  -> hardware SPI + RGB565 framebuffer/browser preview
```

## Native Boundary

Rust owns the scene controllers and calls upstream LVGL directly through
`device/ui/src/renderer/lvgl/ffi.rs`. The only C dependency in this path is
upstream LVGL itself, built from the pinned 9.5.0 source using
`device/ui/native/lvgl`.

## lv_conf.h

Minimal config enabling only what YoYoPod uses:
- `LV_COLOR_DEPTH 16` (RGB565)
- `LV_USE_SNAPSHOT 1` (for screenshot readback)
- Montserrat fonts 12-40
- Flex layout, list, label, button, image widgets

## Building

LVGL native build commands moved out of the CLI in the Round 0 rebuild
and have not yet been ported back. For now, build LVGL via cmake directly
or rely on the CI-built Rust artifacts that already bundle a working
`liblvgl.a`:

```bash
# CI route (preferred): yoyopod target deploy fetches the
# yoyopod-rust-device-arm64-<sha> bundle which already contains the
# linked LVGL.

# Local route (rare; required only for simulation preview):
cmake -S device/ui/native/lvgl -B device/ui/native/lvgl/build \
    -DCMAKE_BUILD_TYPE=Release \
    -DLVGL_SOURCE_DIR=.cache/lvgl/lvgl-9.5.0 \
    -DCONFIG_LV_BUILD_EXAMPLES=OFF -DCONFIG_LV_BUILD_DEMOS=OFF
cmake --build device/ui/native/lvgl/build --parallel 2
```

Do not rebuild LVGL on the Pi for normal dev/prod validation. Use
CI-built Rust artifacts for the exact commit under test.

## Screenshot Support

- `yoyopod target screenshot` defaults to the shadow-first path via `SIGUSR2`.
- `yoyopod target screenshot --readback` requests LVGL readback via `SIGUSR1`.
- The CLI clears the previous remote PNG before capture and waits for a fresh file.
- Both screenshot signals also append freeze diagnostics to `logs/yoyopod_errors.log`:
  - an all-thread backtrace dump
  - a structured runtime snapshot logged before the screenshot is queued
- To confirm which path actually succeeded, check the app log:
  - `Saved screenshot via LVGL readback` means native LVGL snapshotting succeeded.
  - `Saved screenshot via shadow buffer` means the RGB565 framebuffer path was used instead.
