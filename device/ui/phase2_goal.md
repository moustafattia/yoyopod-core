# Phase 2 - UI Domain Enhancements

Phase 1 (`phase1_goal.md`) closed the architecture gap against the original proposal.
Phase 2 is the production-code enhancement slice unlocked by that architecture.

Current slice status: implemented with check-only verification. Test-only work is intentionally out of scope for this phase.

---

## E1. Make transitions actually drive pixels

`TransitionSampler` is passed into typed controllers and controllers consume sampled values where supported.

**Status.** Satisfied by Phase 1 transition sampler consumption.

## E2. Declarative `select_targets` in the registry

Selection behavior is table-driven from the screen registry instead of hardcoded in `app/core.rs`.

**Status.** Satisfied by registry-owned `select_targets`.

## E3. Passthrough policy in the registry

PTT passthrough policy is registry data instead of screen-specific app-core branching.

**Status.** Satisfied by registry-owned passthrough policies.

## E4. Split `render/lvgl/backend.rs` into widget-kind files

The native LVGL backend is split into focused files for backend facade, widget factory, widget registry, flush, and lifecycle.

**Status.** Implemented. No hand-authored file under `render/lvgl/` exceeds 250 LOC; `icons.rs` remains the embedded/generated asset blob.

## E4b. Split `render/lvgl/style_apply.rs` by style responsibility

`style_apply.rs` now delegates to focused modules for tuning, base style, variants, accent handling, and icon-label mapping.

**Status.** Implemented. No hand-authored style module exceeds 250 LOC.

## E5. Per-domain dirty bounding boxes -> dirty-rectangle rendering

The registry exposes dirty regions for domains that only affect bounded screen areas. `DirtyState` resolves a render region and Whisplay flushes that framebuffer region over SPI.

**Status.** Implemented for power/network status-bar patches, with full-frame rendering retained for domains that affect screen content or navigation.

## E6. `Renderer` trait + `NullRenderer`

The app render state now owns `Box<dyn Renderer>`. `LvglRenderer` implements the trait, and `render/null.rs` provides an explicit headless renderer for future harness use.

**Status.** Implemented without adding tests in this phase.

## E7. Watchdog for stale runtime manager ticks

If `ui.tick` stalls for more than 5 seconds, the worker marks the runtime link stalled, emits `UiErrorCode::RuntimeStalled`, and pushes the existing error overlay path.

**Status.** Implemented.

## E8. `ui.health` patch statistics

`UiHealth` now reports full snapshot count and patch counts by `RuntimeSnapshotDomain`.

**Status.** Implemented.

## E9. Capability matrix in `ui.ready`

`UiReady` now carries schema version and registry-derived per-screen capabilities: supported intent kinds and passthrough trigger.

**Status.** Implemented on the UI side. Runtime currently has no current per-screen intent filter to remove; it can consume the matrix when needed.

## E10. Remove transport -> render coupling

Transport remains free of render-layer imports and delegates through app-owned render state.

**Status.** Satisfied by Phase 1 and preserved in this slice.

---

## Deliberately Not Included

| Skipped item | Reason |
|---|---|
| Property/proptest coverage | Test-only scope; explicitly out for this phase. |
| Per-screen unit tests | Test-only scope; explicitly out for this phase. |
| Touch / gesture framework | No touch hardware on this device. |
| Scripted screens (RON-driven) | The registry already covers known screens; adds attack surface without a use case. |
| Hot-reload assets | Firmware update is already the deploy path; adds complexity. |
| Multi-language / i18n | Requires a product decision first; scope creep until then. |
| Soft-state checkpoints | Runtime manager already owns persistent state. Two sources of truth would be worse than today. |
| Widget pooling | LVGL handles allocation; premature without a measured churn problem. |
| Animation timelines / scripted sequences | Single transitions cover today's needs; revisit after measured need. |
| Performance metrics (full flamegraph) | Health patch stats cover the dominant question; broader profiling on demand. |
