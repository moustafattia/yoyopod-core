# YoyoPod Whisplay / Simulation Parity Contract

**Last Updated:** 2026-04-10
**Status:** Proposed

## Problem Statement

`python yoyopod.py --simulate` is now intentionally the Whisplay-sized browser preview, but it is not yet a trustworthy preview of the current Whisplay product UI.

Today the simulation path and the Whisplay path share the same screen size and general profile:

- `240x280`
- portrait orientation
- Whisplay-first product target

But they do not share the same effective rendering path. As the Whisplay LVGL implementation has moved forward, the simulation renderer has drifted in item positioning, spacing, and overall composition. That makes simulation useful for navigation smoke tests, but not good enough for day-to-day UI work when the goal is to preview what the child actually sees on the hardware.

## Goals

- make simulation the primary software-only preview of the current Whisplay UI
- treat Whisplay as the visual source of truth for the product
- remove duplicate layout behavior that can drift between hardware and browser preview
- preserve fast local and Pi-hosted browser preview for development
- keep simulation input ergonomic for developers while matching Whisplay output geometry

## Non-Goals

- emulate SPI timing, LCD refresh artifacts, or backlight latency
- reproduce panel color calibration exactly
- make Pimoroni follow the Whisplay visual profile
- keep a separate "generic simulation" UI language

## Current State

- `src/yoyopod/ui/display/adapters/whisplay.py` owns the Whisplay display profile and the production hardware path
- Whisplay now prefers the LVGL renderer in production
- `src/yoyopod/ui/display/adapters/simulation.py` is still its own PIL-based renderer with browser preview transport
- `--simulate` correctly selects simulation output plus keyboard/web-button input, but the output is still an approximation of Whisplay rather than the same render contract

That split is the root cause of the visual mismatch now visible in the browser preview.

## Contract

### 1. Whisplay is the visual source of truth

The product only has one current portrait-first target UI: Whisplay.

Simulation is a preview of that UI, not a parallel rendering product. When a Whisplay screen is updated, simulation must reflect the same layout contract without requiring a second, manually kept-in-sync positioning implementation.

### 2. Simulation keeps Whisplay output and developer-friendly input

This remains the intended split:

- simulation output: Whisplay profile
- simulation input: keyboard and web buttons
- hardware Whisplay input: one-button PTT navigation

Output parity and input ergonomics are separate concerns and should stay separate.

### 3. Browser preview is transport, not a second renderer owner

`src/yoyopod/ui/web_server.py` should remain a preview transport and browser-facing surface.

It must not become the place where Whisplay-specific layout behavior lives. Layout ownership belongs in the display/rendering contract, not in the preview server.

### 4. Intentional differences must be explicit

If a visual difference between simulation and Whisplay is intentional, it must be:

- documented
- bounded
- covered by tests

"Close enough" is not sufficient for the Whisplay-first product path.

## Required Architecture Direction

### Preferred End State

Simulation should render the same Whisplay frame contract that hardware uses, preferably through one of these approaches:

1. off-screen LVGL rendering for simulation preview
2. mirrored LVGL frame/readback pipeline reused by the browser preview
3. one shared Whisplay render model with no duplicated geometry constants between simulation and hardware

The preferred direction is to reuse the Whisplay rendering path as directly as possible rather than improving a separate approximation forever.

### Acceptable Intermediate State

If full off-screen LVGL reuse is not immediately practical, the next step should still remove the worst sources of drift:

- extract Whisplay layout constants and shared chrome into one shared module
- stop carrying separate card spacing, margins, footer offsets, and status-bar placement in simulation-only code
- add screenshot or anchor-based parity tests so drift becomes visible in CI

## Required Parity Surface

The following must be treated as parity-sensitive for simulation:

- status bar height and icon placement
- menu title block positioning
- selection-card position, size, radius, and padding
- footer copy position
- list item spacing and focus treatment
- Whisplay portrait safe-area assumptions

The first required screen set should include:

- main menu
- Listen
- Talk
- Ask
- Setup
- Voice Commands
- Talk Contact
- Now Playing

## Verification Contract

Parity must be enforced with artifacts, not visual memory.

Required verification layers:

- unit or integration tests for shared geometry/profile values
- screenshot parity checks between simulation and Whisplay references for key screens
- Pi-hosted validation using Whisplay screenshots from the live LVGL path

Small anti-aliasing or color differences can be tolerated. Geometry drift should not be.

## Suggested File Ownership

- `src/yoyopod/ui/display/factory.py`
  - owns adapter selection and preview startup
- `src/yoyopod/ui/display/adapters/whisplay.py`
  - owns Whisplay output behavior
- `src/yoyopod/ui/display/adapters/simulation.py`
  - should become a Whisplay preview surface, not an independently evolving layout engine
- `src/yoyopod/ui/lvgl_binding/`
  - candidate home for off-screen or mirrored LVGL preview support
- `tests/ui/test_display.py`
- `tests/test_remaining_lvgl_views.py`
- future screenshot parity tests under `tests/`

## Acceptance Criteria

- `python yoyopod.py --simulate` shows the same layout geometry as the current Whisplay UI for the required parity screens
- simulation no longer relies on a second drifting copy of Whisplay-first layout behavior without parity tests
- the browser preview remains usable both locally and from the Pi IP
- output parity is preserved while simulation input stays keyboard/web-button based
- UI work can use simulation as a trustworthy preview of Whisplay instead of a rough approximation

## Rollout Outline

1. capture fresh Whisplay reference screenshots for the required parity screens
2. choose the shared render seam, preferably by reusing the Whisplay LVGL path
3. remove or centralize duplicate Whisplay layout logic in simulation
4. add parity tests and make them part of the normal validation flow
5. update simulation docs to describe it as the Whisplay preview path
