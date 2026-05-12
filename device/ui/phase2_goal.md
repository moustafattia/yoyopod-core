# Phase 2 — UI Domain Enhancements

Phase 1 (`phase1_goal.md`) closes the gap against the original proposal.
Phase 2 is about the next layer of value: enhancements that the clean
architecture *unlocks* and that are worth doing for this product.

These are ordered by leverage for an embedded one-button device.
Speculative items (touch gestures, hot-reload assets, scripted screens,
multi-language, soft checkpoints) are intentionally **excluded** — they
would expand surface area without a concrete product need.

---

## E1. Make transitions actually drive pixels `[finish what was started]`

> Note: same work as Phase 1 D5. Listed again here because it is also the
> gateway to every richer animation feature (E-chain below). If D5 lands
> in Phase 1, E1 is satisfied automatically.

**Why first.** Until controllers consume interpolated values, every other
animation enhancement is theoretical.

**Shape.**
- `TransitionSampler { sample(role, prop) -> Option<f32> }` passed into
  `TypedScreenController::sync_model`.
- Each controller asks the sampler for the roles it owns; falls back to
  the static view-model value when no transition is active.
- `presentation/transitions.rs` exposes a `tick(now_ms) -> impl TransitionSampler`.

**Acceptance.** A test that dispatches `UiCommand::Animate { property: Opacity, … }`
records non-static facade calls across the animation window.

---

## E2. Declarative `select_targets` in the registry `[high leverage]`

> Note: same work as Phase 1 D2b. Listed here because, once it lands,
> several follow-up enhancements (E3, E10, E11) become straightforward.

Collapses `core.rs::select_focused`'s 60-LOC `match` into a table-driven
function. New screens stop touching `app/core.rs`.

---

## E3. Passthrough policy in the registry `[high leverage]`

> Note: same work as Phase 1 D4. Promoting to Phase 2 in case D4 ships
> as a tactical patch in Phase 1 — the registry-driven shape is the
> longer-term goal.

Removes hardcoded `active_screen == VoiceNote && phase == …` checks from
the app core. Future input-passthrough screens (e.g. a "hold to record"
on a new screen type) get the behavior for free.

---

## E4. Split `render/lvgl/backend.rs` into widget-kind files `[high]`

After Phase 1 D3, the remaining ~600 LOC of FFI should still be split:

- `widget_factory.rs` — `create_*` helpers.
- `widget_registry.rs` — registry + lookup.
- `flush.rs` — LVGL flush callback + framebuffer bridge.
- `lifecycle.rs` — display open/close, draw-buffer init.

Each ≤ 200 LOC. Makes the FFI layer reviewable.

**Acceptance.** No file under `render/lvgl/` exceeds 250 LOC except
generated assets.

## E4b. Split `render/lvgl/style_apply.rs` by style responsibility `[high]`

Phase 1 moved the hardcoded LVGL styling out of `backend.rs`, but
`style_apply.rs` is still the largest hand-authored LVGL file. Split it by
responsibility before adding new style behavior:

- role tuning / one-off LVGL widget tuning
- base style application from `WidgetStyle`
- variant application
- accent application
- icon fallback label mapping

**Acceptance.** `style_apply.rs` is split into named modules with no
hand-authored style module over 250 LOC.

---

## E5. Per-domain dirty bounding boxes → dirty-rectangle rendering `[high impact]`

Per-domain `DirtyState` already exists. The next step is per-domain
*screen regions*:

```rust
pub struct DirtyRegion { pub x: u16, pub y: u16, pub w: u16, pub h: u16 }

// in ScreenRegistryEntry, per domain that affects this screen:
pub fn dirty_region_for(domain: RuntimeSnapshotDomain) -> Option<DirtyRegion>;
```

Then in the LVGL flush callback, only push the touched rectangles down
the SPI bus. On Whisplay this saves ~134 KB of SPI traffic per frame
when only the status bar changed.

**Acceptance.** A power-only patch (battery percentage change) results in
SPI transfer ≤ 20% of a full frame.

---

## E6. `Renderer` trait + `NullRenderer` for headless tests `[high]`

The proposal tree included `render/null.rs`; it is missing. Today the
dispatcher → app → render path can only be tested with LVGL initialized,
which makes `cargo test` slow and platform-dependent.

**Shape.**
```rust
pub trait Renderer {
    fn render(&mut self, model: &ScreenModel) -> Result<RenderReport>;
    fn flush(&mut self) -> Result<()>;
}
```

- `LvglRenderer: Renderer` (move the impl).
- `NullRenderer: Renderer` (records calls into a `Vec` for assertions).

Once D1 is done, `AppCore` holds `Box<dyn Renderer>` and tests inject the
null variant.

**Acceptance.** A unit test exercises the full
`UiCommand::ApplySnapshot → AppCore::tick → Renderer::render` path with
zero LVGL calls.

---

## E7. Watchdog for stale runtime manager `[med]`

If `ui.tick` has not arrived for N seconds, the worker should:
1. Stop trusting its snapshot.
2. Emit `UiEvent::Error { code: UiErrorCode::RuntimeStalled, … }`.
3. Push an overlay screen ("lost link") using existing overlay precedence.

This is one small timer in `app/core.rs` plus one variant of
`UiErrorCode` (which is already typed).

**Acceptance.** Disconnecting the runtime manager for > 5 s results in
the lost-link overlay being visible.

---

## E8. `ui.health` patch statistics `[low cost, high signal]`

`RuntimeSnapshotPatch` exists, but there is no visibility into whether
the runtime manager *uses* it instead of always sending `Full`. Extend
`UiEvent::Health`:

```rust
pub struct HealthReport {
    // …existing…
    pub full_snapshots: u64,
    pub patches_per_domain: HashMap<RuntimeSnapshotDomain, u64>,
}
```

**Acceptance.** Health output shows non-zero patch counts in normal
operation; ratio > 80% patches for steady-state.

---

## E9. Property test for registry × snapshot × input `[low cost, high signal]`

Now that types are centralized, one proptest can:

1. Generate a random `RuntimeSnapshot` (using domain `Arbitrary` impls).
2. Generate a random sequence of `InputAction`s.
3. Run them through `AppCore`.
4. Assert: never panics; `active_screen` is always in the registry;
   `focus_index` is always in range; intents are decodable.

Catches whole classes of bugs (e.g. focus drifting past list bounds when
the snapshot shrinks, or PTT being emitted on a screen that does not
support it).

**Acceptance.** A `proptest!` block runs in CI; coverage of `core.rs`
substantially increases.

---

## E10. Capability matrix in `ui.ready` `[med]`

The runtime manager already filters intents per screen on its side
(rules duplicated from the worker). Once E2 lands, the registry knows
the capability set of every screen. Emit it once at startup:

```rust
pub struct ScreenCapabilities {
    pub screen: UiScreen,
    pub supported_intents: &'static [IntentKind],
    pub passthrough: Option<InputAction>,
}

UiEvent::Ready { display, schema_version, screens: Vec<ScreenCapabilities> }
```

The runtime manager stops maintaining its own copy.

**Acceptance.** The runtime manager's per-screen intent filter is removed
and replaced by `ui.ready` consumption; integration tests still pass.

---

## E11. Per-screen unit tests `[follow-on from E2]`

Once `select_targets` is data in the registry, each entry can be tested
independently against a stub snapshot:

```rust
#[test]
fn hub_focus_0_selects_listen() {
    let entry = screen_entry(UiScreen::Hub);
    assert_eq!(entry.select_targets[0], SelectionTarget::PushScreen(UiScreen::Listen));
}
```

Replaces today's monolithic `core.rs` test module with focused per-screen
tests.

---

## E12. Remove transport→render coupling `[Phase 1 carry-over]`

> Same as Phase 1 D1. Listed for completeness because Phase 2 work
> (especially E6 NullRenderer) depends on it.

---

## What was deliberately not included

The earlier exploration suggested 15 enhancements. The following are
intentionally **not** in Phase 2:

| Skipped item | Reason |
|---|---|
| Touch / gesture framework | No touch hardware on this device. |
| Scripted screens (RON-driven) | The registry already covers known screens; adds attack surface without a use case. |
| Hot-reload assets | Firmware update is already the deploy path; adds complexity. |
| Multi-language / i18n | Requires a product decision first; scope creep until then. |
| Soft-state checkpoints | Runtime manager already owns persistent state. Two sources of truth would be worse than today. |
| Widget pooling | LVGL handles allocation; premature without a measured churn problem. |
| Animation timelines / scripted sequences | Single transitions (E1) cover today's needs; revisit after E1 ships. |
| State-machine debug API | The proptest in E9 is a cheaper way to get the same confidence. |
| Input recording / replay | E9 (proptest) gives the same regression coverage at less cost. |
| Performance metrics (full flamegraph) | E8 patch-stats covers the dominant question; broader profiling on demand. |

---

## Suggested ordering for Phase 2

1. **Foundation** (must precede everything else):
   E12 (= P1 D1), E6 (NullRenderer), E9 (proptest).
2. **Finish the architecture:** E1 (= P1 D5), E2 (= P1 D2b), E3 (= P1 D4), E4 (split backend).
3. **Performance & resilience:** E5 (dirty-rect SPI), E7 (watchdog).
4. **Cross-crate cleanup:** E10 (capability matrix), E8 (patch stats), E11 (per-screen tests).

Items 1–2 are also Phase 1 closeout work, so the natural sequence is:
finish Phase 1, then proceed directly into Phase 2 items 3 and 4 on a
clean foundation.
