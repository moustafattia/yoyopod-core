# Phase 3 — Clean Restructure of `presentation/` and `render/`

Phase 1 and Phase 2 closed the architecture proposal and added the
production-code enhancements unlocked by it. Phase 3 is purely
**structural**: reshape the two largest layers (`presentation/` and
`render/`) so the file tree reflects the data flow and the boundary
between them is self-policing.

This phase is **file movement + import-path rewriting only**. No
behavioral changes. No new features. No new tests except the architecture
guards that make the boundary enforceable.

---

## Why this phase exists

`render/lvgl/` currently has 45 files across 4 nested levels with mixed
vocabulary (`backend`, `facade`, `scene`, `controllers`, `style_apply`).
A reader cannot infer the data flow from the file tree. The layering is
correct underneath; the discoverability is not.

`presentation/` is fine internally but has a few residual seams
(`screens/models.rs` mixing view-model types with screen builders) and
no enforcement that it stays free of render imports.

The boundary between the two is the most important boundary in the UI
worker — it is the line between "what to show" (pure, render-agnostic,
testable) and "how to draw it" (LVGL, framebuffer, pixels). Today the
boundary holds by convention; after this phase it holds by test.

---

## The boundary in one sentence

> `presentation/` says **what** to show. `render/` says **how** to draw it.

- `presentation/` is pure data + pure functions. No LVGL, no framebuffer,
  no FFI. Could be serialized to JSON and rendered by a different process.
- `render/` is the bridge between that data and pixels. Knows LVGL, knows
  widgets, knows styling, knows the framebuffer.

Exactly three types cross the boundary:
- `ScreenModel` (typed view-model enum)
- `TransitionSampler` (read-only access to active transitions)
- `DirtyRegion` (optional bounded redraw area)

These already define the `Renderer` trait signature today. The Phase 3
work makes that contract structural.

---

## Scope of `presentation/`

After Phase 3, `presentation/` contains exactly five kinds of code:

| Concern | Location |
|---|---|
| Screen identity | `presentation/registry.rs` (re-exports `UiScreen` from protocol) |
| Screen view-model types | `presentation/view_models.rs` |
| Per-screen view-model builders | `presentation/screens/*.rs` |
| Navigation policy data | `presentation/registry.rs` (`SelectionTarget`, `PassthroughPolicy`, `BackPolicy`, `DirtyRegion`, capabilities) |
| Transitions (decisions, not pixels) | `presentation/transitions.rs` |

Everything in `presentation/` must be:
- Free of LVGL imports.
- Free of `crate::render::*` imports.
- Free of framebuffer / FFI / hardware imports.
- Compilable and unit-testable without a display.

### Proposed layout

```
presentation/
├── mod.rs                       Re-exports the public surface
│
├── view_models.rs               HubViewModel, ListScreenModel,
│                                NowPlayingViewModel, AskViewModel,
│                                CallViewModel, …, ScreenModel enum,
│                                ChromeModel, StatusBarModel.
│                                (was: screens/models.rs)
│
├── screens/                     One file per screen — pure view-model builders
│   ├── mod.rs                     Shared helpers
│   ├── chrome.rs                  Status-bar / chrome extractor
│   ├── hub.rs                     fn hub_model(snapshot, focus) -> HubViewModel
│   ├── listen.rs
│   ├── music.rs                   (playlists, recent tracks, now_playing)
│   ├── ask.rs
│   ├── call.rs
│   ├── talk.rs
│   ├── power.rs
│   └── overlay.rs
│
├── registry.rs                  ScreenRegistryEntry, SelectionTarget,
│                                PassthroughPolicy, BackPolicy, DirtyRegion,
│                                ScreenCapabilities, dirty_region_for,
│                                screen_capabilities().
│
└── transitions.rs               Transition, TransitionSampler, Easing,
                                 TransitionTarget, TransitionProperty.
```

### File moves required

| From | To |
|---|---|
| `presentation/screens/models.rs` | `presentation/view_models.rs` |
| `presentation/screens/{hub,listen,music,ask,call,talk,power,overlay}.rs` | unchanged |
| `presentation/screens/chrome.rs` | unchanged |
| `presentation/registry.rs` | unchanged |
| `presentation/transitions.rs` | unchanged |
| `presentation/mod.rs` | updated re-exports |

The substantive change is one file move plus an architecture test. The
rest of `presentation/` already has the right shape.

---

## Scope of `render/`

After Phase 3, `render/` is organised by **role in the pipeline**, not
by implementation pattern. Five folders, each answering one question:

| Folder | Answers |
|---|---|
| `render/pipeline/` | How does a render call flow? |
| `render/screens/` | How does each screen draw itself? |
| `render/widgets/` | What primitive operations exist on a widget? |
| `render/styling/` | How do widgets get their look? |
| `render/lvgl/` | How do we talk to LVGL C? |

The five top-level concerns (`mod.rs` for the `Renderer` trait, `null.rs`,
`framebuffer.rs`, `assets.rs`) stay at the root of `render/`.

### Proposed layout

```
render/
├── mod.rs                       Renderer trait, RenderReport, re-exports
├── null.rs                      NullRenderer (headless tests / harness)
├── framebuffer.rs               Framebuffer struct
├── assets.rs                    RON loader (layouts.ron, theme.ron)
│
├── pipeline/                    L1 + L2: render call flow
│   ├── mod.rs                     LvglRenderer (impl Renderer)
│   │                              (was: render/lvgl/mod.rs)
│   ├── scene.rs                   Scene lifecycle / change detection
│   │                              (was: render/lvgl/scene.rs +
│   │                                    render/lvgl/scene/controller.rs)
│   └── list_view.rs               List-screen render helper
│                                  (was: render/lvgl/scene/list_model.rs)
│
├── screens/                     L3: one renderer per screen — recipes
│   ├── mod.rs                     ScreenRenderer trait, factory
│   │                              (was: render/lvgl/controllers/mod.rs)
│   ├── shared.rs                  Helpers shared by several screens
│   ├── status_bar.rs              Chrome strip rendering
│   ├── hub.rs   listen.rs   playlist.rs   now_playing.rs
│   ├── talk.rs  talk_actions.rs    ask.rs   call.rs
│   ├── list.rs  power.rs           overlay.rs
│   └── talk_actions/layout.rs     Kept if the screen needs sub-files
│
├── widgets/                     L4: widget vocabulary
│   ├── mod.rs                     Facade trait
│   │                              (was: render/lvgl/facade.rs)
│   ├── registry.rs                WidgetId → object map
│   │                              (was: render/lvgl/widget_registry.rs)
│   ├── factory.rs                 Widget creation helpers
│   │                              (was: render/lvgl/widget_factory.rs)
│   ├── roles.rs                   WidgetRole IDs
│   │                              (was: render/lvgl/roles.rs)
│   └── primitives.rs              WidgetId newtype
│                                  (was: render/lvgl/primitives.rs)
│
├── styling/                     Side stack: declarative look-and-feel
│   ├── mod.rs                     apply_role entry point
│   │                              (was: render/lvgl/style_apply.rs)
│   ├── layout.rs                  Rect resolver (consumes layouts.ron)
│   │                              (was: render/lvgl/layout.rs)
│   ├── theme.rs                   Color/font resolver (consumes theme.ron)
│   │                              (was: render/lvgl/theme.rs)
│   ├── style.rs                   Small wrapper (kept for now)
│   ├── tuning/                    Per-role base values
│   │   ├── mod.rs                   (was: style_apply/tuning.rs)
│   │   ├── base.rs                  (was: tuning/base_roles.rs)
│   │   ├── text.rs                  (was: tuning/text_roles.rs)
│   │   ├── list.rs                  (was: tuning/list_roles.rs)
│   │   └── communication.rs         (was: tuning/communication_roles.rs)
│   ├── variants/                  Per-screen style overrides
│   │   ├── mod.rs                   (was: style_apply/variant.rs)
│   │   ├── ask.rs                   (was: variant/ask.rs)
│   │   ├── now_playing.rs           (was: variant/now_playing.rs)
│   │   └── communication.rs         (was: variant/communication.rs)
│   ├── base.rs                    Apply base style for a role
│   │                              (was: style_apply/base.rs)
│   ├── accent.rs                  Apply accent / selection highlight
│   │                              (was: style_apply/accent.rs)
│   └── icons.rs                   Apply icon assets
│                                  (was: style_apply/icons.rs)
│
└── lvgl/                        L5: the actual LVGL C boundary
    ├── mod.rs                     Concrete Facade impl
    │                              (was: render/lvgl/backend.rs +
    │                                    render/lvgl/backend/facade_impl.rs)
    ├── ffi.rs                     Raw extern "C" bindings
    │                              (the only file allowed to import LVGL C)
    ├── lifecycle.rs               lv_init / lv_display_create / teardown
    ├── flush.rs                   Framebuffer → SPI callback
    └── icons.rs                   Embedded bitmap assets (the 853-LOC blob)
```

### What goes away

- The `backend/` subdirectory inside `lvgl/`. `backend.rs` and
  `backend/facade_impl.rs` collapse into `lvgl/mod.rs`.
- The `scene/` subdirectory inside `lvgl/`. `scene.rs`,
  `scene/controller.rs`, `scene/list_model.rs` flatten into `pipeline/`.
- The `_roles` suffix on tuning files. Parent folder already
  disambiguates: `tuning/base.rs` not `tuning/base_roles.rs`.
- The `_apply` suffix on the styling root. `styling/mod.rs` not
  `style_apply.rs`.

### What stays untouched

- `assets/layouts.ron` and `assets/theme.ron` (the data files).
- The `Renderer` trait signature.
- The `Facade` trait signature.
- The architecture tests in `lib.rs:14` (FFI containment) — they remain
  correct, with `render/lvgl/` as the FFI-allowed boundary.

---

## Naming distinction: `presentation/screens/` vs `render/screens/`

Both folders have a file per screen with the same screen name (e.g.
`hub.rs`). This is intentional, not a duplication smell.

| File | Role |
|---|---|
| `presentation/screens/hub.rs` | Pure view-model builder: `(snapshot, focus) -> HubViewModel`. No LVGL. |
| `render/screens/hub.rs` | Render recipe: consumes `HubViewModel`, emits widget facade calls. No snapshot access. |

The same word means the same *thing* (the Hub screen) across both
layers; each file answers a different *question* (data vs drawing). The
two audiences are different people at different times: product engineers
edit the presentation file when changing what Hub shows; graphics
engineers edit the render file when changing how it looks.

---

## What flows across the boundary

Exactly three types, all already defined:

```rust
pub trait Renderer {
    fn render(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,                       // from presentation/
        transitions: &TransitionSampler<'_>,       // from presentation/
        dirty_region: Option<DirtyRegion>,         // from presentation/
    ) -> Result<RenderReport>;
}
```

If a fourth type wants to cross, push back: it is almost certainly
either an app-level concern (belongs in `app/`) or a render-internal
concern (belongs deeper in `render/`).

---

## Architecture-test additions

Phase 3 ships **two new tests** in `src/lib.rs::architecture_tests`:

```rust
#[test]
fn presentation_does_not_import_render() {
    // No file under src/presentation/ may contain "crate::render".
}

#[test]
fn render_does_not_import_app_or_transport() {
    // No file under src/render/ may contain "crate::app" or "crate::transport".
}
```

Combined with the existing tests (FFI containment, transport→render,
old-module names, scene/controller legacy), the test count grows from 5
to 7. Boundaries become self-policing.

---

## Acceptance criteria

Phase 3 is complete when **all** of the following hold:

- [x] `presentation/screens/models.rs` no longer exists; its contents
      live in `presentation/view_models.rs`.
- [x] `render/lvgl/backend/` directory no longer exists.
- [x] `render/lvgl/scene/` directory no longer exists.
- [x] `render/lvgl/style_apply/` and `render/lvgl/style_apply.rs` no
      longer exist; their contents live under `render/styling/`.
- [x] `render/lvgl/controllers/` no longer exists; its contents live
      under `render/screens/`.
- [x] `render/lvgl/facade.rs`, `widget_factory.rs`, `widget_registry.rs`,
      `roles.rs`, `primitives.rs` no longer exist; their contents live
      under `render/widgets/`.
- [x] `render/lvgl/layout.rs`, `theme.rs` no longer exist; their
      contents live under `render/styling/`.
- [x] `render/lvgl/` contains only `mod.rs`, `ffi.rs`, `lifecycle.rs`,
      `flush.rs`, `icons.rs`.
- [x] `render/pipeline/`, `render/screens/`, `render/widgets/`,
      `render/styling/` exist and own their respective files.
- [x] `presentation_does_not_import_render` test passes.
- [x] `render_does_not_import_app_or_transport` test passes.
- [x] `cargo build` passes for all targets.
- [x] All existing tests pass.

---

## Out of scope

Phase 3 is strictly structural. Explicitly out of scope:

| Item | Why deferred |
|---|---|
| Logic changes inside any moved file | Phase 3 is file moves + import rewrites only. |
| New view models / new screens | Add in a follow-up; reshape first. |
| New widget primitives | Same. |
| New tests beyond the two architecture guards | Functional test coverage is its own phase. |
| Performance work (dirty-rect coverage, pooling) | Already noted as Phase 2 follow-up; structural reshape comes first. |
| `assets/*.ron` schema changes | Data files unchanged. |
| Renaming `LvglRenderer`, `Facade`, `Renderer`, `ScreenModel` | Type names stay; only paths move. |

---

## Migration approach

Two sub-phases, each commit-clean on its own.

### Phase 3A — Reshape `render/`

1. `git mv` `render/lvgl/facade.rs` → `render/widgets/mod.rs`.
2. `git mv` `render/lvgl/widget_factory.rs` → `render/widgets/factory.rs`.
3. `git mv` `render/lvgl/widget_registry.rs` → `render/widgets/registry.rs`.
4. `git mv` `render/lvgl/roles.rs` → `render/widgets/roles.rs`.
5. `git mv` `render/lvgl/primitives.rs` → `render/widgets/primitives.rs`.
6. `git mv` `render/lvgl/layout.rs` → `render/styling/layout.rs`.
7. `git mv` `render/lvgl/theme.rs` → `render/styling/theme.rs`.
8. `git mv` `render/lvgl/style_apply/` contents → `render/styling/`
   (with `_roles` and `_apply` suffix drops).
9. `git mv` `render/lvgl/controllers/` → `render/screens/`.
10. `git mv` `render/lvgl/scene.rs` + `render/lvgl/scene/` →
    `render/pipeline/`.
11. `git mv` `render/lvgl/mod.rs` content into `render/pipeline/mod.rs`
    (the `impl Renderer` part) and `render/lvgl/mod.rs` (the concrete
    Facade impl part).
12. Collapse `render/lvgl/backend.rs` + `backend/facade_impl.rs` into
    the new `render/lvgl/mod.rs`.
13. Run `sed` to rewrite all `crate::render::lvgl::*` paths.
14. `cargo build` until quiet.
15. Add `render_does_not_import_app_or_transport` test.
16. Commit.

### Phase 3B — Tidy `presentation/`

1. `git mv` `presentation/screens/models.rs` →
   `presentation/view_models.rs`.
2. Update `presentation/screens/mod.rs` and `presentation/mod.rs`
   re-exports.
3. Run `sed` to rewrite `presentation::screens::models::*` →
   `presentation::view_models::*`.
4. `cargo build` until quiet.
5. Add `presentation_does_not_import_render` test.
6. Commit.

Estimated total effort: **half a day**, mostly mechanical. Reviewable
in two PRs of pure-move commits plus one tiny test addition each.

---

## Summary

Phase 3 takes the layered architecture that Phases 1 and 2 *built* and
makes it *visible* in the file tree. The data flow

```
presentation/ ──ScreenModel──▶ render/pipeline ──▶ render/screens ──▶ render/widgets ──▶ render/lvgl
```

becomes inferrable from the folder names alone. The boundary between
`presentation/` and `render/` becomes self-policing. The LVGL footprint
shrinks to five files. The styling stack stops dominating the page.

No behavior changes. No tests change except the two new boundary
guards. After Phase 3, a new contributor can open the project and
understand the layering in under a minute.
