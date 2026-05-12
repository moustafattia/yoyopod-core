# YoYoPod UI Worker Domain Architecture

**Last updated:** 2026-05-12
**Status:** Current implementation

This document describes the Rust UI worker domain owned by `device/ui/`.
The UI worker is a supervised Rust host process. It owns device UI state,
single-button input interpretation, screen model creation, LVGL scene
controllers, native LVGL rendering, and display flushes.

## Ownership

```text
yoyopod-runtime
  -> yoyopod-ui-host
     -> UiRuntime
     -> ScreenModel
     -> Rust LVGL scene controllers
     -> NativeLvglFacade
     -> upstream LVGL 9.5 C library
     -> RGB565 framebuffer
     -> DisplayDevice flush
```

- `yoyopod-runtime` owns process supervision and app-domain state.
- `yoyopod-ui-host` owns UI-domain state, input interpretation, rendering, and
  hardware display/button access.
- Python remains CLI/deploy/validation tooling only. It is not in the UI
  runtime path.

## Worker Protocol

The UI host communicates over line-delimited JSON worker envelopes:

- stdout is protocol-owned.
- stderr is log-owned.
- the outer `WorkerEnvelope` stays the shared transport container.
- UI command/event payloads are decoded through `yoyopod_protocol::ui`
  (`UiCommand`, `UiEvent`, `UiIntent`, and `RuntimeSnapshot`).
- accepted commands are `ui.runtime_snapshot`, `ui.input_action`, `ui.tick`,
  `ui.poll_input`, `ui.set_backlight`, `ui.health`, `ui.animate`,
  `ui.runtime_patch`, `ui.shutdown`, and `worker.stop`.
- emitted events are `ui.ready`, `ui.input`, `ui.intent`, `ui.screen_changed`,
  `ui.health`, `ui.error`, and `ui.shutdown_complete`.

Runtime snapshots are shared protocol DTOs, not UI-local copies. Local button
or command input is interpreted as typed `InputAction`. Domain actions leave
the UI worker as typed `UiIntent` variants for the runtime to route. Unknown or
malformed UI commands produce a typed `ui.error` event instead of being ignored.

## Render Contract

The app UI has one renderer: Rust-owned LVGL scenes backed by upstream LVGL.
The worker does not negotiate renderer modes and does not fall back to a second
software app renderer.

```text
RuntimeSnapshot
  -> UiRuntime navigation/preemption/focus
  -> ScreenModel
  -> NativeSceneRenderer
  -> RustSceneBridge
  -> screen controller
  -> NativeLvglFacade
  -> LVGL object tree
  -> RGB565 framebuffer
  -> DisplayDevice::flush_full_frame
```

The low-level `Framebuffer` type remains part of the render transport. It is
the RGB565 target that LVGL flush callbacks write into before the display
driver sends pixels to hardware or mock display surfaces. It is not a second
app layout engine.

## Native Boundary

The only C dependency in this domain is upstream LVGL 9.5, built from the
pinned source through `device/ui/native/lvgl`.

- YoYoPod screen state and scene controllers are Rust.
- YoYoPod LVGL FFI declarations are in `device/ui/src/lvgl/sys.rs`.
- YoYoPod LVGL facade and unsafe native calls are contained in
  `device/ui/src/lvgl/native_backend.rs`.
- YoYoPod-owned C scene or shim code is not part of the current UI domain.

## Invariants

- Missing native LVGL is a startup/render failure, not a signal to degrade.
- App screen layout belongs in Rust screen models and Rust LVGL controllers.
- Raw LVGL calls stay inside `device/ui/src/lvgl/`.
- Hardware access stays behind `DisplayDevice` and `ButtonDevice`.
- UI navigation/focus/preemption belongs in `UiRuntime`, not in controllers.
- Controllers sync an already-selected `ScreenModel`; they do not route app
  state or emit cross-domain commands.
