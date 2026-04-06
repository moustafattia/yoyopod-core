# LVGL Display Pipeline

Applies to: `yoyopy/ui/lvgl_binding/**`, `yoyopy/ui/display/adapters/whisplay.py`

## Overview

LVGL 9.5.0 is the standard rendering layer for all hardware displays. The architecture is display-hardware-agnostic -- LVGL renders to its internal buffer, then flush callbacks route pixels to whatever SPI display is connected.

## Rendering Pipeline

```
LVGL object tree
  -> partial render (40-line draw buffer)
  -> flush callback (RGB565_SWAPPED)
  -> LvglDisplayBackend._flush_callback() [Python]
  -> Rgb565FlushTarget.draw_rgb565_region() [adapter]
  -> hardware SPI + PIL shadow buffer (dual-write)
```

## C Shim (`native/lvgl_shim.c`)

The C shim bridges Python (via cffi) and the LVGL C library:
- `yoyopy_lvgl_init/shutdown` -- lifecycle
- `yoyopy_lvgl_register_display` -- sets up flush callback, RGB565_SWAPPED format
- Scene functions (`hub_build/sync/destroy`, `listen_*`, `playlist_*`, etc.) -- each screen type
- `yoyopy_lvgl_snapshot` -- captures active screen via `lv_snapshot_take()`
- `yoyopy_lvgl_force_refresh` -- invalidates and redraws immediately

## Python Binding (`binding.py`)

cffi ABI-mode wrapper. Mirrors the C shim API 1:1. Key patterns:
- Strings encoded to `char[]` via `ffi.new()`
- Colors packed as `(r, g, b)` tuples -> 24-bit RGB via `_pack_rgb()`
- `to_bytes()` converts cffi pixel buffers to Python bytes

## lv_conf.h

Minimal config enabling only what YoyoPod uses:
- `LV_COLOR_DEPTH 16` (RGB565)
- `LV_USE_SNAPSHOT 1` (for screenshot readback)
- Montserrat fonts 12-40
- Flex layout, list, label, button, image widgets

## Building

```bash
python scripts/lvgl_build.py   # clones LVGL 9.5.0, compiles shim
```

Must rebuild on Pi after changing `lv_conf.h` or `lvgl_shim.c`.

## Screenshot Support

Default screenshot capture should use LVGL readback first via `SIGUSR1`.
If readback is unavailable, it may fall back to the adapter screenshot path.
`SIGUSR2` is reserved as a legacy shadow-first debug path only.
