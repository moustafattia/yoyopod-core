# Rust LVGL Scene Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Rust UI host render the same Whisplay LVGL scenes as the Python/C path, then fill the Rust runtime snapshot fields required by those scenes.

**Architecture:** Treat `yoyopod/ui/lvgl_binding/native/lvgl_shim.h` as the scene parity contract. The Rust UI host links to `yoyopod_lvgl_shim`, maps typed `ScreenModel` values into the same scene-specific `build/sync/destroy` calls, and keeps framebuffer flushes on the Rust hardware path. The existing generic Rust LVGL widget facade remains as test scaffolding until the C scene internals are ported into Rust modules.

**Tech Stack:** Rust 1.82 workspace, LVGL 9.5 native C shim, Cargo/CMake build integration, existing Rust UI host tests, Python `uv` quality gates.

---

### Task 1: Scene Contract Test And Build Link

**Files:**
- Modify: `yoyopod_rs/ui-host/build.rs`
- Modify: `yoyopod_rs/ui-host/src/lvgl/sys.rs`
- Test: `yoyopod_rs/ui-host/tests/render_lvgl.rs`

- [ ] Add a failing Rust test that requires each `UiScreen` with an old Python/C LVGL scene to map to the correct retained C scene key: `hub`, `listen`, `playlist`, `now_playing`, `talk`, `talk_actions`, `incoming_call`, `outgoing_call`, `in_call`, `ask`, `power`.
- [ ] Run `cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-ui-host --locked scene_key` and confirm the new parity assertion fails.
- [ ] Update `build.rs` so `native-lvgl` builds and links `yoyopod_lvgl_shim` in addition to `lvgl`.
- [ ] Extend `sys.rs` with the `yoyopod_lvgl_*` FFI declarations from `lvgl_shim.h`.
- [ ] Re-run the focused test and the UI-host unit tests.

### Task 2: Native Shim Scene Renderer

**Files:**
- Create: `yoyopod_rs/ui-host/src/lvgl/scene_backend.rs`
- Modify: `yoyopod_rs/ui-host/src/lvgl/mod.rs`
- Modify: `yoyopod_rs/ui-host/src/render/lvgl.rs`
- Test: `yoyopod_rs/ui-host/tests/render_lvgl.rs`
- Test: `yoyopod_rs/ui-host/tests/lvgl_runtime.rs`

- [ ] Add failing fake-backend tests that prove screen changes call scene-specific `build`, `sync`, and `destroy` operations instead of generic label/container operations.
- [ ] Implement a Rust `NativeSceneBackend` that owns shim lifecycle, display registration, flush callback, scene switching, and `ScreenModel` to shim payload conversion.
- [ ] Route production `render::LvglRenderer` through the scene backend when `native-lvgl` is enabled.
- [ ] Preserve the existing generic `LvglRenderer<F>` tests as semantic scaffolding, but stop using that path for hardware rendering.
- [ ] Run focused Rust tests.

### Task 3: Complete Current Screen Payload Parity

**Files:**
- Modify: `yoyopod_rs/ui-host/src/screens/*.rs`
- Modify: `yoyopod_rs/ui-host/src/runtime/state_machine.rs`
- Test: `yoyopod_rs/ui-host/tests/runtime_state_machine.rs`
- Test: `yoyopod_rs/ui-host/tests/render_lvgl.rs`

- [ ] Port old Python LVGL payload details into Rust models: footer copy, selected visible index behavior, accent selection, empty-state copy, status values, and call footer variants.
- [ ] Make `app_state` select the current screen when present in a runtime snapshot, with overlay and call preemption still taking priority.
- [ ] Add missing route coverage for old registered screens that Rust does not currently represent, starting with `talk_contact` if enough state exists to render it.
- [ ] Run Rust UI-host state-machine/render tests.

### Task 4: Fill Rust Runtime Snapshot Data

**Files:**
- Modify: `yoyopod_rs/runtime/src/state.rs`
- Modify: `yoyopod_rs/runtime/tests/state.rs`
- Optional Modify: worker payload modules if source data exists.

- [ ] Add failing tests showing `ui_snapshot_payload()` emits contacts, voice, power, network/GPS, and richer hub subtitles when state contains that data.
- [ ] Extend `RuntimeState` only where worker data already exists; do not invent unavailable hardware signals.
- [ ] Keep default values aligned with `yoyopod/ui/rust_host/snapshot.py`.
- [ ] Run Rust runtime tests.

### Task 5: Local And Hardware Verification

**Files:**
- Modify tests/docs only if verification discovers a stale command or missing contract.

- [ ] Run `cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-ui-host --locked`.
- [ ] Run `cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-runtime --locked`.
- [ ] Run `uv run python scripts/quality.py gate`.
- [ ] Run `uv run pytest -q`.
- [ ] Commit and push.
- [ ] Use the GitHub Actions Rust UI artifact for the exact commit on Pi hardware; do not build Rust on the Pi Zero 2W.
- [ ] Validate with `yoyopod remote validate --branch <branch> --sha <commit> --with-rust-ui-host` and screenshot/photo comparison.
