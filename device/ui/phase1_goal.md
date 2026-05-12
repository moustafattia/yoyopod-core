# Phase 1 — Divergence from the Architecture Proposal

Goal of this document: close the gap between the original UI architecture
proposal (see `ARCHITECTURE_AUDIT.html`) and what actually landed in
`device/ui/`. The refactor shipped most promises, but a handful of items
either did not land, landed shallowly, or regressed during integration.
Each item below is the work needed to declare Phase 1 complete.

Status legend: ❌ not shipped · 🟡 partial / inert · ✅ shipped (listed for context)

---

## D1. Transport reaches into render `[high]` ✅

The proposal required strict layer purity: `transport/` may only know about
`device/protocol` and serde — never LVGL, never the framebuffer.

**Closed:**
- `transport/` no longer imports `crate::render`.
- The dispatcher maps `UiCommand` into `AppEvent` only.
- Render dirty handling moved into the app layer through `UiRuntime::render_if_dirty`.

**Goal:**
- Remove `Framebuffer` and `LvglRenderer` imports from `transport/`.
- Move `render_if_dirty` into `app/core.rs` (e.g. `AppCore::after_command`).
- Have `dispatcher::dispatch_command` return a `DispatchOutcome` that the
  main loop hands back to `AppCore`; the app then decides whether to render.

**Acceptance:**
- `grep -r "use crate::render" device/ui/src/transport/` returns nothing.
- The layer-purity test in `lib.rs` is extended to assert this.

---

## D2. `app/core.rs` is the new monolith `[high]` ✅

`runtime/state_machine.rs` was 628 LOC. The first cut moved too much into
`app/core.rs`, which reached **677 LOC**. The split files existed but were
skeletal:

| File | LOC | Substantive content |
|---|---|---|
| `app/navigator.rs` | 29 | 3 helpers (`runtime_preemption`, `is_call_screen`, `is_overlay_screen`) |
| `app/focus.rs` | 52 | 4 numeric helpers |
| `app/input_router.rs` | 56 | `route()` ignores its `_state` param |
| `app/intents.rs` | 50 | 4 small action builders |

The substantive logic had stayed in `core.rs`:
- `select_focused` at `core.rs:265–330` — 60+ LOC of hardcoded
  `match UiScreen { Hub => match focus_index { 0 => …, 1 => … } }`.
- `apply_app_state_route` at `core.rs:245`, not in `navigator.rs`.
- 49 references to specific `UiScreen::*` variants in `core.rs`.

**Closed:**
- Move `apply_app_state_route` into `navigator.rs`.
- Replace `select_focused`'s match with a registry-driven dispatcher
  (see D2b below) so screen-specific logic stops touching `core.rs`.
- `core.rs` is now below 300 LOC of orchestration.

**Acceptance:**
- `core.rs` line count ≤ 300.
- `grep -c "UiScreen::" device/ui/src/app/core.rs` ≤ 5 (only orchestration
  defaults such as the initial screen).

---

## D2b. Declarative navigation graph `[high]` ✅

The proposal called for a data-driven nav graph in the screen registry.
Today only the *backward* edge is registry-driven (`NavigationPolicy::Root|Stack|Overlay|Call`).
The *forward* edges (`Hub focus 0 → Listen`, etc.) are still in match arms.

**Goal:**
Extend `presentation::registry::ScreenRegistryEntry` with declarative
selection targets:

```rust
pub enum SelectionTarget {
    PushScreen(UiScreen),
    EmitIntent(IntentTemplate),
    PushWithIntent { screen: UiScreen, intent: IntentTemplate },
    DynamicListItem { kind: ListKind },   // resolved at runtime against the snapshot
}

pub struct ScreenRegistryEntry {
    // …existing fields…
    pub select_targets: &'static [SelectionTarget],
}
```

`select_focused` becomes a ~10-line generic over the registry.

**Acceptance:**
- `select_focused` in `core.rs` no longer matches on `UiScreen`.
- Adding a new screen requires zero edits in `app/core.rs`.

---

## D3. Native backend did not shrink to ~600 LOC `[high]` ✅

Proposal target: `native_backend.rs` shrinks from 2,383 → ~600 LOC of pure
FFI. Layout/theme tables did move to RON (✅), but the backend file is still
3× the target.

**Original state:**
```
render/lvgl/backend.rs   1,757 LOC   (~41 functions)
render/lvgl/facade.rs      124
render/lvgl/scene.rs       430
render/lvgl/layout.rs       59   ✅
render/lvgl/theme.rs        84   ✅
render/lvgl/style.rs        45   ✅
```

**Closed:** Split `backend.rs` into focused modules:
- `render/lvgl/widget_factory.rs` — widget creation + registration.
- `render/lvgl/widget_registry.rs` — the `HashMap<WidgetId, WidgetNode>`.
- `render/lvgl/style_apply.rs` — `ThemeRole` consumption.
- `render/lvgl/backend.rs` — FFI lifecycle + flush callback only.

**Acceptance:**
- `backend.rs` ≤ 600 LOC.
- Each new sibling module has a single, named responsibility.

---

## D4. PTT passthrough still special-cased `[med]` ✅

Five direct checks of `self.active_screen == UiScreen::VoiceNote &&
self.voice_note_phase() == "…"` at `core.rs:332, 338, 429, 442, 452`.

**Goal:** Push passthrough semantics into the registry:

```rust
pub struct PassthroughPolicy {
    pub trigger: InputAction,
    pub when: SnapshotCondition,
    pub intent: UiIntent,
}
```

Per-screen `Option<PassthroughPolicy>` in `ScreenRegistryEntry`. Removes
all five literal checks from `core.rs`.

**Acceptance:**
- `grep "VoiceNote" device/ui/src/app/core.rs` returns only `UiScreen::VoiceNote`
  in the registry-init / test code paths (not in conditionals).

---

## D5. Animation wiring exists but is inert `[high]` ✅

`presentation/transitions.rs` declares `Transition`, `Easing`,
`TransitionTarget`, `TransitionProperty`. `core.rs::advance_animations`
advances them and marks an animation dirty flag.

But **no controller reads the interpolated value**. The worked example
from the audit — "controllers stay unchanged, they just call
`facade.set_accent(widget, view_model.signal_color)` with an interpolated
value" — was not implemented.

This is the most consequential gap: the protocol surface (`ui.animate`)
exists but is silently a no-op. Consumers will trust it and not get
animation.

**Goal:**
- Add a `TransitionSampler` passed into `Controller::sync_model` that
  resolves `(WidgetRole, AnimatableProp) → Option<f32>`.
- Each controller checks the sampler for relevant roles and, when a
  transition is active, applies the sampled value (opacity / offset /
  scale) via the facade.
- `TransitionProperty::Opacity` and `TransitionProperty::SelectionOffset`
  must produce visible motion in the LVGL output.

**Acceptance:**
- A snapshot test driving an `Animate` command produces interpolated
  facade calls during `tick`s within the animation window.
- Removing `advance_animations` causes the test to fail.

---

## D6. Dispatcher is doing too much `[med]` ✅

`transport/dispatcher.rs` was 269 LOC and owned command decoding, rendering,
and error emission. The completed split is:

- `transport/dispatcher.rs` — only `UiCommand → AppEvent`.
- `app/core.rs::after_command()` — render + dirty handling.
- `transport/outbound.rs` — already owns event emission; keep it that way.

**Acceptance:**
- `dispatcher.rs` ≤ 120 LOC.
- `dispatcher.rs` has no `use crate::render::…` import.

---

## D7. Confirmed shipped (for the record) ✅

- Typed `UiCommand` / `UiEvent` / `UiIntent` enums with serde tagging.
- `ui.runtime_patch` + per-domain `DirtyState`.
- `ui.shutdown_complete` event.
- Data-driven `layouts.ron` and `theme.ron` loaded at startup.
- Typed `TypedScreenController` trait with associated `Model`.
- Tunable input timing (`input/config.rs`).
- Single chrome extractor in `presentation/screens/chrome.rs`.
- No more `serde_json::Value` payloads or `json!()` calls in `device/ui/src/`.

---

## Phase 1 completion checklist

- [x] D1 — transport stops importing render
- [x] D2 — `core.rs` ≤ 300 LOC; routing logic moved out
- [x] D2b — declarative `select_targets` in the registry
- [x] D3 — `backend.rs` ≤ 600 LOC, split into named modules
- [x] D4 — passthrough policy in the registry; no literal screen checks
- [x] D5 — transitions drive visible facade properties
- [x] D6 — `dispatcher.rs` ≤ 120 LOC, no render imports

Once these boxes are ticked, the architecture proposal is fully realized
and Phase 2 enhancements can begin from a clean baseline.
